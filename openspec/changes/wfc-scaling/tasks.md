# Tasks

Each task lands as its own small commit: code tasks use the `wfc:` scope,
tasks that only touch files under `openspec/` use `openspec:` (AGENTS.md).
Code tasks are tagged **[keep]**
(seed-preserving: `cargo test -p expl_wfc` and
`cargo test -p expl_wfc -- --ignored` pass with golden fixtures untouched;
step-count statistics assertions may be updated, see D2) or
**[break]** (seed-breaking: fixtures regenerated in the same commit with the
explicit regenerate switch, corpus numbers before/after recorded in the
commit message). Per D12, every task
made for performance records criterion and/or corpus numbers before and
after in its commit message, and is reverted if it shows no improvement
unless it enables a later task (then: no regression, and name that task).
Every commit passes
the AGENTS.md checklist (`cargo fmt`, `cargo clippy --workspace
--all-targets`, `cargo test --workspace`, `cargo deny check bans`).

## 1. Instrumentation and safety net

- [x] 1.1 [keep] Add run statistics to the generator readable by the caller: steps, collapses, rewinds (history entries popped), max backtrack depth (most collapses undone by one uninterrupted sequence of rewinds) and stale failures (a `select` failing on a cell that is neither the cell just restored by rewind nor a neighbour of the last collapse); add an opt-in collapse-event recorder (off by default) that logs every collapse (coord, tile, whether its propagation completed) as it happens; verify a statistics test asserts the golden seed's step and collapse counts (statistics, not golden fixtures) and the golden trace is unchanged
- [x] 1.2 [keep] Add a collapse budget (default 15 x layout cell count, caller-settable at construction) with a `WFCError` variant returned when it is exceeded, and a `WFCError` variant returned when `rewind` finds the history empty (replacing the `collapsed.pop().unwrap()` panic); adapt `step()`, the example, benches, tests and `src/map_generator/task.rs` to the new return type; verify a test with a tiny budget fails with that error at the same collapse count on two runs, a test with an explicit budget overrides the default, an unsatisfiable setup (e.g. a template whose tiles cannot neighbour each other) returns the history error instead of panicking, and the golden trace is unchanged
- [x] 1.3 [keep] Add a test helper that checks every pair of neighbouring collapsed cells is compatible per the template, and apply it in `test_collapse` and `test_fixed_seed`; verify both tests pass
- [x] 1.4 [keep] Add a test helper for the D13 progress invariant: after every step, build the log measure (one `(|rejected|, committed)` element per history depth, plus the pending retry from `self.rejected` as an uncommitted element) and assert it is strictly greater, lexicographically, than after the previous step; expose read access to the pending rejected list if needed (no behaviour change); apply it in `test_collapse` and `test_fixed_seed` (and the heavy golden checks once 2.2 lands); verify both tests pass and a unit test shows the helper rejects a non-increasing sequence

## 2. Measurement baseline (phase 0)

- [x] 2.1 [keep] Re-record the golden trace per collapse event using the 1.1 recorder instead of polling `collapsed.last()` per `step()` call: each entry is (coord, tile), with the pending snapshot only for collapses whose propagation completed without a contradiction; make the test require an exact match including the number of entries, and rewrite `res/trace.jsonl` only when explicitly requested (e.g. `WFC_BLESS=1`), never on a normal run; verify the new fixture is derived from the current generator unchanged (same collapse sequence as the old trace's collapse entries), `test_fixed_seed` passes against it, a truncated fixture makes it fail, and a normal run leaves the file untouched
- [x] 2.2 [keep] Add golden output checks for 2-3 backtracking-heavy square seeds (e.g. `AF4HR7NIWMZRATUV36FQ` at 120x120): stable, explicitly specified hash (e.g. inline FNV-1a over the dumped grid, not `DefaultHasher`) plus the collapse count from 1.1, fixtures in `res/` regenerated only via the same explicit switch as 2.1, tests `#[ignore]`d and also running the 1.3 compatibility check; verify `cargo test -p expl_wfc -- --ignored` passes and regular `cargo test` skips them
- [x] 2.3 [keep] Add a corpus runner example (deterministic seed list per size, sizes 30x24/64/128/256, optional 1024) that reports completed/failed counts, p50/p90/max steps and collapses, per-cell ratios, the stale-failure share of all failures, and p50/max time; verify it runs in release at 30x24, 64 and 128 and prints the table, with budget failures counted instead of hanging
- [x] 2.4 [keep] Replace the single-seed `generate_large` criterion bench with a fixed multi-seed 128x128 set (keep `load_template`, `generate_only`, `generate_full`); verify `cargo bench -p expl_wfc` runs all benches
- [ ] 2.5 Run the corpus at 30x24/64/128/256 and record the phase-0 baseline table in `design.md` Context, including the stale-failure share to confirm or reject the cascade hypothesis; verify the table is present with seed counts and failure counts
- [ ] 2.6 Checkpoint with the user: revisit the 256x256 target numbers and budget factor K against the baseline, record the outcome in `design.md` (D11, Goals) and, if the target numbers change, in the `wfc-generation` spec; verify `openspec validate wfc-scaling --strict` passes. Outcome: most budget failures are not budget-bound (trial-run budget table, D11), so 30x24 completion is a goal of the later search phases (2 and 4), K is decided at 3.3, and the target numbers are revisited once phase 1 gives a 256x256 baseline

## 3. Mask scoping and graph-based backjumping (phase 1)

Sections 1-2 land as a groundwork merge request first. 3.1, 3.2 and 3.4 are separate commits in the next merge request: 3.1 alone is a large regression (Context) and must not be merged without 3.2.

- [ ] 3.1 [break] Scope rejected masks to history (D13): track the masked cell and, on every rewind, recompute the previously masked cell from its collapsed neighbours before restoring the popped entry; add a test helper asserting that every uncollapsed cell except the masked one equals its recomputed domain, applied after every step in `test_collapse`, `test_fixed_seed` and the heavy golden checks; verify regenerated fixtures pass, the compatibility check holds, the stale-failure counter is 0 across the corpus, and corpus before/after is recorded (a regression is expected here, see Context)
- [ ] 3.2 [break] Add graph-based backjumping (D7): an `induced` list per history entry carried with the rejected list (through `self.rejected` on retry, dropped when jumped over), and on every failure jump to the newest history entry among the dead cell's collapsed neighbours and induced list, undoing the entries above it without rejecting their tiles, merging the rest of the conflict set into the target's induced list, then rewinding the target; a failure with no conflict returns the empty-history error; add backjump statistics (jumps, entries skipped, longest jump) to the run statistics and the corpus runner; verify regenerated fixtures pass, the compatibility, D13 domain and progress invariant checks hold, and the corpus against phase 0 and 3.1 is recorded in the commit message (trial-run prototype: 986/1000 at 30x24, 188/200 at 64x64, 47/50 at 128x128, 14/16 at 256x256 with K = 100)
- [ ] 3.3 Run the corpus, record the phase-1 table and backjump statistics in `design.md`, and checkpoint with the user: K (with the trial-run budget table in D11), the 256x256 target numbers, and whether the phase-1 merge request lands before a later phase restores 30x24 completion; verify `openspec validate wfc-scaling --strict` passes
- [ ] 3.4 [break] Set the default budget factor to the 3.3 decision (K = 100); regenerate the heavy golden fixtures whose outcome depends on the budget; verify the budget tests and golden checks pass and the corpus at the new default is recorded in the commit message

## 4. Selection heuristic exploration (phase 2)

- [ ] 4.1 On a throwaway branch from the phase-1 generator, make the cell selection key pluggable and prototype the D8 candidates (MRV baseline, MRV with proximity tie-break, surrounded-first, options per uncollapsed neighbour, fixed spiral order, enclosed-region first, seeded random tie-break), plus forward checking (6) as a second axis on the best one or two; extend the scratch corpus runner with the sealed age of dead regions; run the 30x24 (1000), 64, 128 and 256 corpora at the phase-1 K, escalate failures to K = 1000, and dump a few maps per candidate; verify every candidate passes the compatibility and progress invariant checks, and the results table (completion, never-completing failures, steps/cell p50/max, mean/max jump, sealed age, time per step, map character notes) is recorded in `design.md` D8; the prototype is not merged
- [ ] 4.2 Checkpoint with the user: choose the selection heuristic (or keep MRV) and whether forward checking is adopted with it, from the 4.1 table; record the decision and its rationale in `design.md` D8/D9; verify `openspec validate wfc-scaling --strict` passes
- [ ] 4.3 [break] (skip if 4.2 keeps MRV) Implement the chosen selection heuristic (and forward checking if 4.2 adopts it) in the generator; verify regenerated fixtures pass, the compatibility, D13 domain and progress invariant checks hold, corpus before/after against the 3.3 table is recorded, and the dumped maps were reviewed for artefacts

## 5. Seed-preserving optimisations (phase 3)

- [ ] 5.1 [keep] Replace the pending `HashMap` scan with a structure suited to the selection order chosen in 4.2 (for a count-based order: a priority queue with the same total order and stale-entry skipping, bucket queue as fallback); update test invariants and trace extraction for the new type; verify fixtures untouched and corpus time per step drops on the 128 set
- [ ] 5.2 [keep] Make the backjump keep jumping while the restored target has no alternative left after removing its rejected tiles, computing each next conflict set exactly as a separate failing step would (D5); update the step-count statistics assertion; verify golden fixtures untouched, collapse counts unchanged, and corpus step count and time drop on backtracking seeds
- [ ] 5.3 [keep] When propagation takes a neighbour to zero alternatives (with `retain` reporting "became zero" in the same pass), backjump immediately from the zeroed neighbour that selection would pick under the chosen order, instead of running selection (D5); update the step-count statistics assertion; verify golden fixtures untouched, collapse counts unchanged, and corpus step count and time drop on backtracking seeds
- [ ] 5.4 [keep] Build template compatibility from tiles materialised once as `[Item; 7]` and compute each pair once using direction symmetry; verify a test shows the compatibility sets and tile order equal those of the straightforward per-pair `compatible_with` computation, fixtures untouched, and `load_template` bench improves
- [ ] 5.5 [keep] Fix `Template::stats` to count compatible tiles (`count_ones`) instead of bitset capacity; verify a unit test on the sample template shows min < max
- [ ] 5.6 [keep] Look up neighbour constraints in `alternatives()` by the opposite direction (`(i + 3) % 6`) instead of scanning all six offsets; verify fixtures untouched and the multi-seed generate bench improves
- [ ] 5.7 Run the corpus, record the phase-3 table in `design.md`, and checkpoint with the user on target numbers and K as in 2.6; verify `openspec validate wfc-scaling --strict` passes

## 6. Search improvements (phase 4)

- [ ] 6.1 [break] (skip if 4.2 adopted it) Filter tile choice at select with a forward check (skip tiles that would leave any neighbour with zero alternatives), not stored in the domain; verify regenerated fixtures pass, the compatibility and D13 invariant checks hold, and corpus p90/max steps improve over the 5.7 table
- [ ] 6.2 STOP before region reset (D6 gate): with the latest corpus numbers, reconsider with the user whether giving up the progress invariant (D13) is worth it; weigh full CBJ (6.6) first and invariant-preserving reset variants, with the trial-run evidence that resets are what rescue hard-region seeds (D6); record the decision and chosen order in `design.md` D6/D9, and do not start 6.3-6.4 without explicit approval; verify the decision is recorded and `openspec validate wfc-scaling --strict` passes
- [ ] 6.3 [keep] (only if 6.2 approves region reset) Make collapse history support removing arbitrary cells (per-cell history index plus tombstones) while chronological rewind behaves as before; enables 6.4; verify fixtures untouched and no corpus regression
- [ ] 6.4 [break] (only if 6.2 approves region reset) After a bounded number of consecutive chronological pops, clear collapsed cells within a radius of the contradiction (using `expl_hexgrid` region iteration, minding that `ring` is 1-based), recompute the boundary domains, and grow the radius when the same area fails again; per D6 drop the active mask and re-pick `next`, and clear the rejected lists of surviving entries collapsed after the earliest removed entry; adapt the 1.4 progress helper to whatever monotone measure 6.2 chose, or restrict it to runs without resets if none; verify regenerated fixtures pass, the compatibility and D13 invariant checks hold after every reset and on all corpus output, and the 256 corpus completes without budget failures
- [ ] 6.5 Decide on full conflict-directed backjumping from the latest corpus and the backjump statistics (6.4, or 6.1 if region reset was not done): implement it as 6.6 if the 256x256 tail still misses the target, otherwise record in `design.md` D7 that it was not needed; verify the decision and numbers are recorded
- [ ] 6.6 [break] (if needed per 6.2 or 6.5) Replace GBJ's neighbour-based conflict sets with full CBJ per D7: record per rejected tile which earlier cells excluded it or killed it downstream, and build the dead cell's conflict set from those instead of all its collapsed neighbours; verify regenerated fixtures pass, the compatibility, D13 domain and progress invariant checks hold, and corpus before/after is recorded
- [ ] 6.7 Run the 30x24 and 256 corpora against the spec target and a small 1024x1024 corpus (stretch), recording the tables in `design.md`; verify every 30x24 and 256 corpus seed completes within the default budget and the 256 target, and report the 1024 outcome

## 7. Flat domain storage measurement

- [ ] 7.1 On a throwaway branch, prototype allocation-free domain storage (D5 option a or c, whichever is quicker) and run the 256 and 1024 corpora plus the multi-seed bench against the 6.7 numbers, measuring time and peak memory; verify the comparison is recorded in `design.md` D5 with a recommendation (no follow-up, small option-c follow-up change, or an `expl_hexgrid` proposal for option d), and the prototype is not merged

## 8. Integration

- [ ] 8.1 Launch the game and start a new game to confirm the 30x24 map still generates and that a budget error surfaces as a map generation error rather than a hang; verify via game logs or a BRP `Party` entity query

## Workflow follow-up

- Archive the change once phase 4 targets are met and reviewed.
- Open a separate `expl_hexgrid` proposal if profiling shows layout/offset operations are hot, or if 7.1 recommends a run-time-stride grid (D10).
