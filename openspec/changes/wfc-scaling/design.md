# Design

## Context

See proposal.md - Why. The generator (`crates/expl_wfc/src/generator.rs`)
works like this today:

```
 step()
   |
   v
 next cell --select--> tile? --no--> rewind()  pop ONE collapse; restore it
   |                                           with its rejected tiles masked
   | yes                                       out; recompute its uncollapsed
   v                                           neighbours from their collapsed
 collapse, push (coord, tile, rejected)        neighbours (no mask)
   |
   v
 propagate: 6 neighbours  alts &= compat[tile][dir]      <-- one hop only
   |
   v
 next = linear scan of pending HashMap: min(num_alts), tiebreak (q, r)
```

Propagation is one hop, so a cell's domain is the intersection of its
collapsed neighbours' constraints **plus**, for a cell restored by
`rewind`, a mask of its rejected tiles. Nothing removes that mask when the
cell's history entry is discarded; only a later rewind of one of the cell's
neighbours recomputes it.

This produces a **stale-mask cascade**. Rewind restores A with its rejected
tiles masked and A is left with 0 alternatives. `step` fails on A and the
next rewind pops B, overwriting `self.rejected`; if B is not adjacent to A,
A keeps its masked 0 in `grid` and `pending`. B is re-collapsed, A is again
the minimum, `select` fails on A, and B is popped again. B's remaining tiles
are burned one by one (two steps each) regardless of whether they work,
until B is itself a stale 0. The cascade continues until a neighbour of
every stale cell has been popped. In effect today's generator does an
accidental backjump that is both expensive and unsound: tiles are rejected
because of an unrelated dead cell, so the search is incomplete.

Measurements (release build, template of 193 tiles, 4 u64 words per domain):

| Case | Result |
|---|---|
| `load_template` | 6.86 ms |
| `generate_only` (hex r10, golden seed) | 0.135 ms, 325 steps |
| 30x24, 5 random seeds | 884-1844 steps for 720 cells (1.2-2.6 per cell) |
| 60x60, 5 random seeds | 4724-6554 steps for 3600 cells (1.3-1.8 per cell), ~0.02 s |
| 120x120 `AF4HR7O3U6VZGHS5TOCQ` (bench) | 20 641 steps, 27 ms, ~1.3 us/step |
| 120x120 `AF4HR7NIWMZRATUV36FQ` | 109 153 steps, 0.92 s, ~8.4 us/step |
| 120x120 `AF4HR7JCVXJJOZQAHCYA` | 613 281 steps, 19.3 s, ~31 us/step |
| 240x240 | first seed did not finish in ~20 min |

#### Phase 0 baseline (task 2.5)

Measured with the corpus runner (`cargo run --release -p expl_wfc --example
corpus -- <sizes> --seeds N --jobs J --list-failures`) on the unchanged
generator (groundwork of sections 1-2 only, before phase 1), template of
193 tiles, budget K = 15 x cells (budget failures are counted, not hung). Seeds are a
deterministic splitmix64 list per size, the n-th seed depends only on size
and n. Steps/collapses are taken over all runs (a failed run contributes the
work done up to the budget). Stale share = stale failures / failed selects
(rewinds), summed over runs. Times are wall clock with `--jobs` runs in
parallel on a 20 thread machine, so the larger sizes are somewhat inflated.
Criterion: `load_template` 6.91 ms, `generate_only` 136 us, `generate_full`
7.06 ms, `slow/generate_large` (5 seeds at 128x128) 2.015 s.

| size | seeds | completed | failed | steps p50 | steps p90 | steps max | collapses p50 | collapses p90 | collapses max | steps/cell p50 | steps/cell max | max backtrack depth | stale share | time p50 | time max |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 30x24 | 1000 | 997 | 3 | 954 | 1364 | 21457 | 837 | 1042 | 10800 | 1.32 | 29.80 | 29 | 10.8% | 0.44 ms | 19.4 ms |
| 64x64 | 200 | 196 | 4 | 9476 | 23412 | 122448 | 6786 | 13754 | 61440 | 2.31 | 29.89 | 34 | 24.1% | 15.6 ms | 545 ms |
| 128x128 | 50 | 42 | 8 | 173954 | 489012 | 490340 | 95169 | 245760 | 245760 | 10.62 | 29.93 | 40 | 27.2% | 4.26 s | 23.8 s |
| 256x256 | 16 | 0 | 16 | 1961602 | 1964202 | 1964557 | 983040 | 983040 | 983040 | 29.93 | 29.98 | 37 | 28.8% | 136 s | 195 s |

Maxima equal to the budget (10800 / 61440 / 245760 / 983040 collapses) are
runs that hit it. Observations:

- Every size has budget failures at K = 15, including 3 of 1000 shipped
  30x24 seeds. Re-running the 30x24 corpus with K = 1000 completes one of
  them; the other two are still failing after 720 000 collapses (1.4 M
  steps), so they are not a matter of K but of a search that does not
  terminate in practice.
- No 256x256 seed completes (16 of 16 hit the budget after ~2 M steps).
  The tail is not seed-specific: p50 equals max.
- Stale failures are 11% of failed selects at 30x24 and 24-29% at 64 and
  above (about 28% in the 256 runs). The cascade is therefore a real but
  minority share of the work in the failing runs; the remaining ~70% of
  failed selects are on the retried cell or next to the last collapse. This
  supports the cascade as a contributor, not as the sole driver of the tail;
  the unsound pruning it causes is not visible in this number and is only
  tested by phase 1.

#### Trial-run findings (after phase 0)

Throwaway prototypes on the phase-0 code tested whether the stale-mask
cascade drives the tail. It does not; it hides the real cause.

**D13 alone regresses badly.** Scoping the mask (D13) with nothing else,
same seeds, K = 100 so the budget is not the limit:

| size | completed: today -> D13 alone | steps/cell of completed, p50: today / D13 |
|---|---|---|
| 30x24 (1000 seeds) | 998 -> 889 | 1.32 / 1.31 |
| 64x64 (200) | 198 -> 69 | 2.24 / 1.42 |
| 128x128 (50) | 48 -> 3 | 10.3 / 2.13 |

No seed completes with D13 that fails today. Raising K from 15 to 100
rescues none of D13's failures, while D13's completed runs are cheaper than
today's. Outcomes are bimodal: either cheap, or no progress at any budget,
with a roughly constant risk per cell (survival ~ exp(-cells / 3900)).

**Why: chronological backtracking through unrelated history.** Tracing
D13's failing runs (30 seeds at 64x64, 18 failures):

- Each failure is a small region (typically 4-11 cells) that cannot be
  completed given its collapsed neighbours. One-hop propagation does not
  detect it.
- The region is sealed long before it is attempted: on one seed its
  neighbours were collapsed at history depths 978-1140 and the region at
  ~2460. Min-count selection leaves such holes dormant.
- In 14 of 18 failures, 92-1600 unrelated history entries (more than 2
  cells from the region) sit above the region's newest neighbour. In the
  other 4 the region's own neighbours are spread over history with
  286-1459 unrelated entries between them.
- Chronological backtracking must exhaust every combination of those
  unrelated entries before it can change a cell that matters. Collapse
  counts grow geometrically up the stack (~10-100x per entry at the bottom
  to ~59 000x for the dead region at the top). The progress invariant
  (D13) holds throughout; the distance to the way out is just exponential.

**The cascade was an accidental backjump.** Under today's code each dead
region cell becomes a stale zero and stays dead until one of its own
neighbours is popped, so the cascade unwinds straight to the region's
boundary, burning the skipped cells' tiles at ~2 steps each: linear instead
of exponential, but expensive, unsound (tiles rejected without being
tried) and over-eager (skipped cells become stale zeros that extend the
jump).

**Graph-based backjumping (GBJ, D7) on top of D13 fixes the trap.**
Prototype on the same seeds, K = 100:

| size | completed: today / D13 / D13+GBJ | steps/cell of completed, p50: today / D13+GBJ | time of completed, p50: today / D13+GBJ |
|---|---|---|---|
| 30x24 (1000) | 998 / 889 / 986 | 1.32 / 1.30 | 0.6 / 0.7 ms |
| 64x64 (200) | 198 / 69 / 188 | 2.24 / 1.39 | 19 / 5.5 ms |
| 128x128 (50) | 48 / 3 / 47 | 10.3 / 1.41 | 3.95 s / 50 ms |
| 256x256 (16) | 0 (at K = 15) / - / 14 | - / 1.51 | - / 0.40 s (max 3.3 s) |

Same seed, both completing, GBJ / today steps: p50 0.99 at 30x24, 0.67 at
64x64, 0.15 at 128x128. Jumps on completing 64x64 runs average 1-15
entries (max 3-226). The compatibility and progress-invariant checks hold.

**What remains: hard compact regions.** Every seed GBJ fails that today
completes (13 at 30x24, 10 at 64x64) is a different failure class: a large
compact region searched exhaustively (e.g. 52 hot cells within 16 hexes;
133 cells whose conflicts reach back to the first ~100 collapses), not
unrelated entries. Today's over-eager cascade sometimes escapes these
because it wipes a large area, i.e. an accidental region reset. This
matches the known heavy-tailed behaviour of complete backtracking search
(Gomes, Selman, Kautz), for which restarts are the standard remedy.

**These failures are mostly not a budget problem.** Retrying GBJ's K = 15
failures at escalating budgets:

| size | failures at K = 15 | complete at K = 100 | at K = 1000 | at K = 10 000 | never (K = 10 000) |
|---|---|---|---|---|---|
| 30x24 | 15 | 1 | +3 (0.1-0.9 s) | +1 (5.4 s) | 10 |
| 64x64 | 15 | 3 (0.3-1.4 s) | 0 | 0 | 12 |

Today's generator: 3 of 1000 30x24 seeds fail at K = 15, 1 completes at
K = 100, 2 never complete up to K = 1000. So at 30x24, D13 + GBJ leaves
~1.0% of seeds that never complete against ~0.2% today. Today those seeds
hang the game; with the budget they return an error instead.

Causes:

1. **Chronological backtracking over unrelated history** (exponential in
   the entries between a dead region and its newest relevant neighbour),
   made worse by selection leaving sealed holes dormant. Today it is hidden
   by the unsound cascade. Fixed by D13 + GBJ.
2. **Hard compact regions** (heavy-tailed complete search). Not fixed by
   backjumping; candidates are the selection heuristic (D8, explored in
   phase 2), full CBJ (D7, reserve) and region reset or restarts (D6,
   gated).
3. **Per-step cost grows** (1.3 -> 31 us) because choosing `next` scans the
   whole pending set, which grows ragged while thrashing.

#### Phase 1 result (task 3.3)

D13 + GBJ as implemented (tasks 3.1, 3.2) reproduce the trial-run
prototype exactly. Corpus runner, completed seeds:

| size | phase 0 (K = 15) | D13 + GBJ, K = 15 | D13 + GBJ, K = 100 | steps/cell p50 | time p50 (completed) | backjumps: mean / max skip |
|---|---|---|---|---|---|---|
| 30x24 | 997/1000 | 985/1000 | 986/1000 | 1.30 | 0.67 ms | 3.5 / 225 |
| 64x64 | 196/200 | 185/200 | 188/200 | 1.40 | 6.4 ms | 7.5 / 1789 |
| 128x128 | 42/50 | 47/50 | 47/50 | 1.41 | 48 ms (phase 0: 4.26 s) | 5.7 / 2545 |
| 256x256 | 0/16 | 14/16 | 14/16 | 1.53 | 0.42 s | 3.3 / 2620 |

Against the 256x256 targets: ~1 s per map is met at p50 (slowest completed
run ~3.3 s); max/p50 steps among completed runs is ~8.9x (target 3x); 14 of
16 seeds complete. Of the pinned heavy seeds, `AF4HR7NIWMZRATUV36FQ`
completes in 18 023 collapses (61 776 in phase 0), while
`AF4HR7O3U6VZGHS5TOCQ`, the fast benchmark seed in phase 0, fails: it does
not complete at K = 1000 either (14.4M collapses), a hard compact region.

Checkpoint outcome: default K = 100 (D11); the 256x256 targets stay as they
are, measured outside regular CI (Goals); phase 1 merges after the
groundwork, accepting the 30x24 gap until phases 2 and 4 (D9).

The golden trace test (`tests/wfc_test.rs`, `res/trace.jsonl`) covers a
271-cell hex map (325 steps, ~27 rewinds), so it guards the rewind path only
lightly. It also only compares a prefix of the fixture and rewrites
`res/trace.jsonl` on every passing run, so "fixture untouched" currently
proves little.

`expl_hexgrid` needs no changes: `i32` coordinates and `usize` sizes cover
1024x1024; `Neighbours<T>` is indexable by direction and
`NEIGHBOUR_OFFSETS` pairs opposites as `(i + 3) % 6`; `HexCoord::distance`,
`ring` and `spiral` cover locality and region iteration (note `ring(c, r)`
is 1-based: `ring(c, 1)` is empty). `expl_map` and the game only consume the
exported `Grid<SquareGridLayout, _>`.

## Goals / Non-Goals

**Goals:**
- 256x256: every seed in a ~50-seed corpus completes; max/p50 steps < 3x;
  release build within ~1 s per map. These numbers are revisited at every
  phase checkpoint. They are measured with the corpus runner and the opt-in heavy checks, outside regular CI
  (`cargo test` skips ignored tests and does not run examples), since a
  256x256 corpus takes far longer than a CI run should.
- 30x24 (shipped size): every seed in the 30x24 corpus still completes
  within the default budget.
- 1024x1024 (stretch): every seed in a smaller corpus completes, in minutes.
- Seed-preserving optimisations are proven so by unchanged golden outputs.
- Every performance claim is proven by measurement (D12).
- Establish and keep the domain invariant of D13.

**Non-Goals:**
- Chunked generation, AC-4, whole-map restart (see Decisions).
- Changing tile weighting / output aesthetics beyond what search changes imply.
- Template construction beyond the cheap fixes (it is per asset load, not per map).
- Changing cell/domain storage in this change (measured only, see D5).

## Decisions

### D1. Keep one-hop propagation; get extra pruning from forward checking

Domains stay a pure function of collapsed neighbours, except for the one
cell being retried after a rewind (D13). This keeps undo order-independent
and log-free, which is what makes region reset (D6) and backjumping (D7)
cheap.

- Alternative: full arc consistency (AC-4 support counters). Rejected: needs
  a removal trail for undo, kills order-independent undo, and costs
  ~1M x 193 x 6 x 2 B ~= 2.3 GB of counters at 1024x1024.

### D2. Classify every improvement by seed impact

A seed must reproduce the same map on the same version; cross-version
stability is not required. Each improvement is labelled up front:

| # | Improvement | Attacks | Seed |
|---|---|---|---|
| 0 | Counters, seed corpus, golden hashes | measurement | keep |
| 1 | Pending priority queue | per-step cost | keep |
| 2 | Multi-step rewind | step count | keep |
| 3 | Early break in propagate + immediate rewind | step count | keep |
| 4 | Flat domain storage | per-step cost, memory | deferred: measure only (D5) |
| 5 | Template build: materialised tiles + symmetry, `stats` fix | load time | keep |
| 6 | Forward check at select | step count | break |
| 7 | Graph-based backjump (GBJ); full CBJ in reserve | thrashing | break |
| 8 | Selection heuristic, chosen by measurement (D8) | thrashing, hard regions | break |
| 9 | Local region reset + bounded backtracking | heavy tail | break |
| 10 | Rejected-mask scoping (D13), lands with 7 | soundness | break |

Seed-preserving means: the same exported map **and** the same sequence of
collapse events (coord, tile) for every seed. How that work is split over
`step()` calls is not part of the contract: multi-step rewind (2) and early
break (3) merge steps without changing the map, so step counts are a
statistic (measurement), never a golden fixture or a budget unit (D11).
Seed-preserving changes must leave every golden fixture untouched
(statistics assertions may be updated); a seed-breaking change regenerates
the fixtures in the same commit, once.

### D3. Golden hashes for backtracking-heavy seeds

Golden fixtures follow the D2 contract, so they never encode step
granularity. Collapse events are captured by an opt-in recorder in the
generator (off by default) that logs every collapse as it happens, so
events merged into one `step()` call by (2) or (3) are still seen. Tests do
not poll `collapsed.last()` after `step()`.

- **Fine-grained trace:** the existing radius-10 trace is re-recorded once,
  in phase 0, with one entry per collapse event (coord, tile) instead of
  per `step()` call. The pending snapshot is kept only for collapses whose
  propagation completed without a contradiction, since early break (3)
  intentionally stops part-way. The re-record is derived from today's
  generator, so behaviour does not change.
- **Heavy seeds:** golden checks for a few square seeds that backtrack
  heavily (e.g. `AF4HR7NIWMZRATUV36FQ` at 120x120): a stable, explicitly
  specified hash of the exported grid plus the collapse-event count from
  the generator's collapse counter. Hashing is used instead of full traces
  to keep fixtures small.
- **Strict comparison:** golden tests require an exact match, including the
  number of trace entries, and never rewrite fixtures during a normal run.
  Fixtures are regenerated only when explicitly requested (e.g.
  `WFC_BLESS=1`), so a seed-breaking commit shows the regenerated files in
  its diff and a seed-preserving one cannot touch them by accident.

The fine-grained trace keeps running in regular `cargo test`. The heavy
golden checks are `#[ignore]`d (workspace crates build at `opt-level = 1`
under test) and run explicitly with `cargo test -- --ignored`, notably
before and after every seed-preserving change.

- Alternative: full traces for large seeds. Rejected: 100k+ step fixtures.

### D4. Measurement before optimisation

Generator exposes counters: steps, collapses, rewinds (history entries
popped), max backtrack depth, and stale failures (a `select` that fails on a
cell that is neither the cell just restored by rewind nor a neighbour of
the last collapse; this counts the cascade from Context and is 0 once D13
lands), plus, from phase 1, backjump statistics (jumps that skipped at
least one entry, entries skipped in total, longest jump). A corpus runner
(example binary) generates N seeds per size (30x24/64/128/256, optionally
1024) and reports p50/p90/max steps and collapses, stale-failure share, and
time. Criterion keeps micro-benchmarks
but over a fixed multi-seed 128x128 set rather than the single lucky seed.
The corpus is the yardstick for every phase.

### D5. Seed-preserving mechanics

These run after D13 + GBJ and the selection choice (phase 3), so "identical
output" below is relative to that generator, where no stale masks exist and
every failure goes through the backjump. The mechanics below assume a
count-based order like MRV; if phase 2 picks a different order, the queue
is rebuilt for that key.

- **Pending queue (1):** `BinaryHeap` keyed by the same total order
  (min `num_alts`, then max `q`, max `r`) with stale-entry skipping (an entry
  is stale if its count no longer matches the cell). Same order, so same
  output. A bucket queue indexed by count (<= 193 buckets) is the fallback if
  stale entries pile up while thrashing.
- **Multi-step jump (2):** inside the backjump, keep jumping while the
  restored target has no alternative left after removing its rejected tiles,
  computing each next conflict set exactly as a separate failing step would.
  That failing step uses no RNG, so output is identical.
- **Early break (3):** when `retain` takes a neighbour to zero, backjump
  immediately instead of running selection (`retain` reports "became zero"
  in the same pass as intersect + count). With GBJ the dead cell matters,
  because its neighbours form the conflict set: the break must fail on the
  same cell selection would pick (under MRV, the zeroed neighbour with the
  highest (q, r)). So the 6-neighbour pass still completes; what is saved is the
  selection scan. With D13 the only zero cells are ones the last collapse
  just produced, `select` would fail without touching the RNG, and the jump
  recomputes the affected cells from scratch, so output is identical.
- **Opposite-direction lookup:** `alternatives()` indexes each collapsed
  neighbour's compatibility by the opposite direction (`(i + 3) % 6`)
  instead of scanning all 6 offsets.
- **Template (5):** materialise each tile as `[Item; 7]` once and compare
  arrays; compute each pair once using `compat(a,b,d) == compat(b,a,-d)`.
  Tile ordering is unchanged, so tile ids and seeds are unaffected.

**Possible follow-up, not implemented here: flat domain storage (4).** Today
every uncollapsed cell owns a heap-allocated `FixedBitSet` (4 words for 193
tiles): ~1M small allocations and ~90 MB at 1024x1024, alloc/free churn on
collapse and rewind, and pointer chasing in `propagate`. None of the other
improvements depend on changing this, so it is skipped and only measured
(after phase 4) to judge whether it is worth pursuing.
Options if the measurement shows a real win:

| Option | Keeps `Grid<L, Cell>` | Note |
|---|---|---|
| a. Run-time-stride buffer inside `expl_wfc` | no | conflicts with D10 |
| b. Inline `[u64; 4]` domain in `Cell` | yes | hard cap of 256 tiles |
| c. Inline small bitset (`SmallVec<[u64; 4]>`) | yes | no allocation up to 256 tiles, spills beyond |
| d. Run-time-stride grid in `expl_hexgrid` | via new type | separate hexgrid proposal (D10) |

The measurement uses a throwaway prototype (a or c is enough to bound the
gain); if the gain justifies the densest layout, the result is a proposal
for (d), otherwise (c) is a candidate for a small follow-up change.

### D6. Local repair instead of global backtracking

On a contradiction, allow at most a bounded number of chronological pops.
Past that bound, clear all collapsed cells within radius r of the
contradiction (recomputing domains on the region boundary via the one-hop
rule), remove them from history, and continue. If the same area keeps
failing, grow r. Region reset stays deterministic because it only uses
generator state and the seeded RNG.

A reset must keep the D13 invariant and the history consistent:

- The active mask is dropped: `self.rejected` is cleared, the masked cell is
  recomputed from its neighbours, and `next` is re-picked from `pending`.
- Surviving history entries collapsed after the earliest removed entry have
  their rejected lists cleared: those rejections were made against a
  history prefix that no longer exists, and keeping them would prune tiles
  that are now valid.
- A reset that empties the history simply restarts from the remaining
  state; it never calls `rewind` on an empty history (D11).

**Evidence from the trial run.** The failures left after D13 + GBJ are
hard compact regions (Context), and today's unsound cascade escapes some of
them precisely because it wipes a large area. That is direct evidence that
a reset-like move is what those seeds need, matching the standard remedy
(restarts) for heavy-tailed backtracking search. It does not settle the
invariant question below.

**Region reset gives up the progress invariant (D13).** Removing entries
from the middle of the log and clearing rejected lists can return the
search to a state it has been in before, so termination would rest only on
the collapse budget and the growing radius, and the invariant could no
longer be used to reason about correctness. That is a high price, so D6 is
gated: before any D6 work, stop and reconsider with the corpus numbers
from the phase-2 and forward-checking results in hand. Options to weigh at
that point include full CBJ (D7) first, since it keeps the invariant, and
reset variants that keep a monotone measure (e.g. a reset that counts as growing some shallower
element, or a reset count folded into the measure with its own bound).
The outcome is recorded here before D6 is implemented, or D6 is dropped.

- Alternative: whole-map restart. Rejected: at 1M cells the chance of some
  contradiction approaches 1 and each restart discards seconds of work.
- Alternative: chunking. Out of scope (proposal); region reset is its
  chunk-free counterpart and reuses the same undo mechanics.

History becomes non-chronological after a reset, so `collapsed` needs to
support removal of arbitrary entries (e.g. per-cell history index plus
tombstones); this is designed together with D6, not in phase 3.

### D7. Graph-based backjumping, lands with D13

D13 alone turns today's accidental backjump into plain chronological
backtracking, which gets trapped (Context). Graph-based backjumping (GBJ,
Dechter) replaces the accidental jump with a sound one. The two land in the
same merge request.

With one-hop constraints the constraint graph is hex adjacency: a cell's
options depend only on its collapsed neighbours, so no change to a
non-neighbour can revive a dead cell. Each history entry gets an `induced`
list next to its rejected list: the earlier cells blamed for its rejected
tiles. It has the same lifetime as the rejected list (carried with
`self.rejected` through a retry, dropped when the entry is jumped over).

When cell c has no alternative (every failure, including a neighbour just
zeroed by propagation):

1. `conflict = collapsed neighbours of c ∪ induced(c)` (empty induced for a
   freshly zeroed cell);
2. target t = the newest history entry in `conflict`; none means the
   empty-history error (D11);
3. undo every entry above t without rejecting its tile, recomputing those
   cells from their neighbours and dropping their rejected and induced
   lists, and the mask (D13);
4. `induced(t) ∪= conflict \ {t}`, then rewind t as today.

Induced lists are what make the jump sound for a cell that died through its
own rejected tiles: those failures may trace back to non-neighbours, and
the merged lists carry them along. A neighbour set is a superset of the real
reasons, so jumping to its newest member never skips a solution, and a jump
is a run of pops that ends by growing t's element, so it keeps the progress
invariant (D13). Ordinary failures next to the last collapse jump 0 entries
and behave like today's rewind.

The jump length is set by the selection order (how long a sealed region
stays dormant); GBJ makes a long jump linear instead of exponential, but the
skipped work is still redone. Shortening jumps is the selection
heuristic's job (D8, phase 2).

- **Full CBJ (Prosser) in reserve.** CBJ records conflicts per rejected tile
  (which neighbour excluded it, or which downstream conflict killed it), so
  a neighbour that excluded none of c's tiles is not blamed and jumps can be
  longer. It costs per-tile bookkeeping (up to 193 tiles x 6 neighbours per
  failure) and more code. On a hex grid nearly every collapsed neighbour
  constrains a cell, so GBJ's sets should be close; CBJ is weighed at the D6
  gate if jump statistics show GBJ blaming neighbours that did not
  contribute.
- **Dynamic backtracking** (undo only the culprit, keep later unrelated
  cells) would make ordering matter less, but reorders history and so gives
  up the progress invariant like D6. Not planned.

### D8. Cell selection heuristic: explored, not assumed

Today the next cell is the one with the fewest remaining options (MRV),
ties broken by highest (q, r). Tiles are unweighted (`Cell::select` picks
uniformly), so classic WFC's minimum-entropy order is the same order as
MRV, not an alternative. The trial run shows MRV leaving sealed regions
dormant (sealed at depth ~1140, attempted at ~2460), which sets GBJ's jump
lengths and probably feeds the hard compact regions. A replacement chosen
by intuition would be as much a guess as MRV, so phase 2 compares
candidates by measurement before adopting one.

Candidates, each a selection key prototyped on the phase-1 generator:

| | Candidate | Idea | Risk / cost |
|---|---|---|---|
| a | MRV, (q, r) tie-break | today; baseline | - |
| b | MRV, proximity tie-break | ties go to the cell nearest the last collapse, so growth and history stay compact | proximity changes every collapse, harder to queue |
| c | Surrounded-first | most collapsed neighbours first, then MRV, so a sealed region is filled while its neighbours are at the top of history | may fill holes it would have been better to leave open |
| d | Options per uncollapsed neighbour | CSP dom/deg: prefer cells that are constrained and constrain few others | ratio key, needs care to stay deterministic |
| e | Fixed spiral from the start cell | `expl_hexgrid` spiral order: no holes by construction, the frontier is a ring | ignores domain sizes; may leave directional artefacts |
| f | Enclosed-region first | when uncollapsed cells form a small component cut off by collapsed cells, fill it next | component detection cost |
| g | MRV, seeded random tie-break | cheap check of how much ties matter | none (still deterministic per seed) |

Forward checking (6) filters tile choice rather than ordering cells, so it
is measured as a second axis on the best one or two orders rather than as a
candidate of its own.

Metrics per candidate: completion over the 30x24 (1000 seeds), 64, 128 and
256 corpora; failures that never complete (escalated to K = 1000);
steps/cell p50 and max; mean and max backjump length; the sealed age of
dead regions (depth when the last neighbour collapsed versus depth when the
region is attempted); time per step, since some keys are costlier to
maintain; and map character from a few dumped maps per candidate (e.g.
directional artefacts from a fixed order).

Prototypes are throwaway, like the trial run; the results table and the
decision are recorded here, and only the chosen heuristic is implemented,
as one seed-breaking task. If no candidate beats MRV, MRV stays and the
numbers are recorded. The choice comes before the seed-preserving phase
because the pending structure (D5) is built for one particular order:
optimising MRV first would lock in fixtures and code for an order that may
be replaced.

### D9. Phasing

```
 phase 0  measure     0                          (trace re-recorded, goldens added)
 phase 1  soundness   10 (D13) + 7 (GBJ)         one merge request; goldens regenerated; corpus vs phase 0
 phase 2  selection   8: explore candidates (throwaway prototypes, D8), checkpoint,
                      adopt the winner (+ 6 if chosen); goldens regenerated
 phase 3  keep seed   1 -> 2 -> 3 -> 5 for the chosen order (golden outputs unchanged)
 phase 4  break seed  6 (if not adopted in phase 2), then STOP: D6 gate (reset vs full
                      CBJ), then the chosen one(s); corpus before/after each; goldens regenerated
 after    measure 4 (flat storage) to decide on a follow-up / hexgrid proposal
```

Merge requests: the spec change lands first, then the groundwork
(sections 1-2 of the tasks: instrumentation, budget, golden checks, corpus
runner, benches), then phase 1 (D13 + GBJ + the K = 100 default) as its own
merge request. Phase 1 merges without waiting for phases 2 and 4, accepting
that ~1.4% of 30x24 seeds return an error at K = 100 where ~0.2% hang
today.

D13 + GBJ go first because they fix unsound pruning, remove the trap that
dominates the tail at 64x64 and above, and change the behaviour every later
measurement and seed-preserving argument (D5) is based on. D13 must not
land without GBJ: alone it is a large regression. Phase 2 settles the
selection order by measurement, phase 3 optimises that order without
changing output, and phase 4 attacks what is left with a cheap, stable
baseline.

### D10. No workarounds for `expl_hexgrid`

`expl_wfc` does not work around or reimplement `expl_hexgrid` parts for
performance (e.g. its own neighbour-index tables, offset arithmetic or grid
storage). If profiling shows a hexgrid operation is hot, the improvement is
promoted to a separate proposal for the `expl_hexgrid` crate, so every user
of the crate benefits and layout logic stays in one place.

### D11. Runtime collapse budget, separate from the quality target

The 3x-median target is a corpus quality bar; the generator cannot know a
corpus median at run time, and work per cell depends on the template. The
runtime cutoff is therefore a budget counted in **collapse events**, not
steps:

- Every pop undoes an earlier collapse, so pops <= collapses and total work
  is bounded by the collapse count. Unlike steps, collapses are unaffected
  by multi-step rewind (2) and early break (3), so a seed-preserving change
  cannot move which seeds hit the budget.
- Default `budget = K x cells` with `K = 100`, set with phase 1 (the
  groundwork and the phase-0 baseline use `K = 15`). The cheap part
  of the tail is rescued (below) and failures stay bounded, but they are
  not cheap at large sizes: measured release-build cost of a failing run is
  ~0.2 s at 30x24, ~3 s at 64x64, ~13 s at 128x128 and ~93 s at 256x256
  (6.5M collapses); 1024x1024 would cap at ~100M collapses. The game only
  generates 30x24 today; the per-cell factor is revisited when larger maps
  are used, together with the stagnation-window question (Open Questions).
- Trial-run evidence (Context): most budget failures do not complete at any
  budget tried, before or after D13 + GBJ. A larger K buys a thin tail only:
  for D13 + GBJ at 30x24, K = 100 rescues 1 of 15 failures, K = 1000 another
  3 (0.1-0.9 s each) and K = 10 000 one more (5.4 s), leaving 10 that never
  complete; at 64x64 K = 100 rescues 3 of 15 and nothing more after that.
  Since K is per cell, a large K also makes every failure at large sizes
  expensive (K = 1000 at 256x256 is 65M collapses).
- Completion of the whole 30x24 corpus is therefore not something K can
  buy; it is a goal for the search phases (2 and 4).
- The caller can set the budget explicitly, so a harder template raises it
  without changing the default.
- Exceeding the budget makes generation return an error instead of
  `step()` continuing; the game's `generate_map` already returns `Result`.
- Running out of history (a rewind with nothing left to pop, today a panic
  on `collapsed.pop().unwrap()`) also returns an error, so an unsatisfiable
  start fails cleanly instead of panicking in the game's map task.

Alternatives: wall-clock timeout, rejected because the same seed could
succeed on one machine and fail on another, breaking determinism. Step
budget, rejected because step granularity is deliberately unstable (D2).
Stagnation window (no new high-water mark of collapsed cells within W
collapses), deferred; see Open Questions.

### D12. Performance claims must be measured

Any change made for performance records evidence: criterion benchmarks for
per-operation cost, and the seed corpus (D4) for step counts, tail and
end-to-end time, before and after, in the commit message. A change that
shows no improvement is reverted, unless it exists to enable a later
improvement, in which case it must show no regression and name the change
it enables (e.g. history removal for region reset).

### D13. Rejected masks belong to history

Invariant: at most one uncollapsed cell carries a rejected-tile mask: the
cell in `next` whose rejected list is held in `self.rejected`. Every other
uncollapsed cell's domain equals the intersection of its collapsed
neighbours' constraints.

Mechanism: track the masked cell. On every rewind, before restoring the
popped entry, recompute the previously masked cell (if still uncollapsed)
from its neighbours and update `pending`. Region reset (D6) and
backjumping (D7) drop the mask the same way. A test helper checks the
invariant after every step: recomputing every uncollapsed cell except the
masked one gives its stored domain.

This removes the stale-mask cascade and the unsound pruning it causes. It
is seed-breaking: fewer tiles are rejected, so different tiles are chosen.

The stale state only arises one way: rewind restores A with its rejected
tiles (including the tile just popped) masked out, and they cover all of
A's neighbour-implied domain. That domain is never empty (it is the domain
A had when first collapsed: neighbours collapsed before A are still
collapsed, later ones are already popped, and it contains A's old tile). If
the mask leaves any option, A is collapsed on the very next step and the
mask no longer matters. So D13 only changes how that one cell is reset; when
the popped cell is adjacent to A, today's neighbour recompute already does
exactly this.

After D13 the grid is a function of the log: collapsed cells come from
history, every other domain from its collapsed neighbours, and the single
mask from `self.rejected` on `next`. Today it is not, because a stale mask's
rejected list is overwritten in `self.rejected` and lost from the log.

**Progress invariant.** Search state can never repeat, and every run either
completes or exhausts the history. Measure the log as a sequence with one
element per history depth, `(|rejected|, committed)`: a history entry is
committed, a pending retry after a rewind (`self.rejected` = the popped
entry's rejected list plus its tile, for the cell in `next`) is an
uncommitted element at the popped depth. Compare sequences
lexicographically, with uncommitted < committed at equal size and an
extension greater than its prefix. Every operation strictly increases it:

- collapse of a new cell pushes a committed element (extension);
- rewind replaces the top committed `(n, _)` by an uncommitted `(n + 1, _)`
  and drops nothing shallower;
- a retry collapse turns the uncommitted element into a committed one of
  the same size;
- a failing retry pops the next entry, again growing a shallower element.

Depth is bounded by the cell count and `|rejected|` by the tile count, so
the order is finite: no state repeats and the search terminates, ending in
the empty-history error (D11) if the root is exhausted. The argument uses
only the log, so it holds today (the cascade is wasted but valid progress)
and is unchanged by D13. A test helper asserts the strict increase after
every step. Phase-3 changes keep it (they only merge steps), and so do GBJ
and CBJ (D7): a jump is a run of pops that ends by growing the target's
element. Region reset (D6) gives it up; see there.

- Alternative: keep the masks and make them sound by scoping them to the
  history prefix (a mask per entry, undone on pop). Rejected: it is
  equivalent but needs an undo log, against D1.
- The cascade is an accidental backjump, and without it chronological
  backtracking gets trapped (Context, trial run). D13 therefore lands
  together with GBJ (D7).

## Risks / Trade-offs

- [Seed-preserving changes drift output by accident] -> golden hashes on
  backtracking-heavy seeds (D3) in addition to the existing trace, with
  strict comparison and no automatic fixture rewrite.
- [Stale heap entries grow while thrashing] -> bucket-queue fallback (D5).
- [Region reset loops on an unsatisfiable area] -> growing radius plus the
  collapse budget (D11) that returns an error instead of hanging.
- [Region reset gives up the progress invariant] -> D6 is gated on an
  explicit stop-and-reconsider checkpoint; invariant-preserving options
  (CBJ, monotone reset variants) are weighed first.
- [Region reset leaves stale masks or rejections] -> reset rules in D6 and
  the D13 invariant check.
- [Removing the cascade makes the tail worse] -> shown by the trial run;
  D13 lands only together with GBJ (D7).
- [D13 + GBJ leave more failing 30x24 seeds than today (~1.4% at K = 100,
  ~1.0% that never complete, vs ~0.2%)] -> accepted: today those seeds hang
  the game, afterwards they return an
  error; the remaining hard-region failures are the target of phases 2 and
  4.
- [Fixed K too tight for a hard template] -> budget is caller-configurable.
- [Default K fails seeds at the shipped size] -> 30x24 is part of the
  corpus; the trial run shows most such failures are not budget-bound, so
  they are addressed by search changes (phases 2 and 4), not by K.
- [A new selection order or forward checking changes map character] -> they
  only change search order/pruning, not the tile rules; map dumps are part of
  the phase-2 comparison (D8), e.g. for directional artefacts from a fixed
  order.
- [The selection replacement is another guess] -> phase 2 compares several
  candidates by measurement against MRV before adopting one, and keeps MRV if
  none is better (D8).
- [Public `pending`/`collapsed` types change] -> only the tests read them;
  update those alongside.
- [Per-cell allocations limit 1024x1024] -> ~90 MB is acceptable for the
  stretch goal; measured after phase 4 (D5 follow-up).
- [Measurement noise hides small wins] -> criterion's statistics for
  micro-benchmarks; corpus comparisons use step and collapse counts
  (deterministic) where possible and time only as a secondary signal.
- [Success numbers are first estimates] -> revisit at every phase
  checkpoint.
- [The game cannot use a 1024x1024 map yet] -> accepted; `expl_map` keeps
  per-tile entities and `Grid<_, HashSet<Entity>>` presence, so game-side
  scaling is a separate future change. This change only makes generation
  ready.

## Open Questions

- Region-reset parameters (pop bound, initial radius, growth) are tuned
  from corpus results during phase 4.
- Exact corpus sizes per grid size, picked so the runner stays under a few
  minutes.
- Whether to add a stagnation-window cutoff alongside the per-cell budget,
  if a fixed budget proves too blunt (e.g. mostly easy maps with one hard
  region).
- GBJ's remaining failure rate did not grow with map area in the trial run
  (about 1.5%, 6%, 6% and 12.5% from 30x24 to 256x256, small samples at the
  large sizes), unlike D13 alone. Whether starting-area or map-edge effects
  explain this is unverified.
