# Proposal

## Why

`expl_wfc` is fine for the current 30x24 prototype map but does not scale:
at 120x120 the same size takes anywhere from 27 ms to 19 s depending on the
seed (1.4x to 43x steps per cell), and a single 240x240 seed did not finish
in 20 minutes. Bigger maps are the next step for the game, so generation must
reliably complete at the target size of 256x256 (stretch: 1024x1024) for
every seed, not just lucky ones.

## What Changes

- Add measurement: step/rewind/backtrack-depth counters on the generator, a
  seed-corpus runner reporting p50/p90/max steps and time per size, and a
  criterion benchmark over a multi-seed set instead of one fast seed.
- Strengthen drift detection: add golden output hashes (exported grid +
  collapse-event count) for square seeds that backtrack heavily, since the
  existing radius-10 trace barely exercises rewind; make golden checks
  strict (exact length, no automatic fixture rewrite).
- Fail cleanly: a collapse budget and an error when backtracking runs out of
  history, instead of hanging or panicking.
- **BREAKING** (seed output), done first after the baseline and in one
  merge request: scope the rejected-tile mask that `rewind` leaves on a
  restored cell to that cell's history entry, and add graph-based
  backjumping. Today the mask outlives the entry, causing a cascade that
  rejects tiles without trying them (unsound) but also acts as an accidental
  backjump; a trial run showed that removing it alone traps plain
  chronological backtracking, and that a sound backjump fixes both.
- Choose the cell selection heuristic by measurement: compare several
  candidates (including today's fewest-options order) on the seed corpus
  before adopting one, since a replacement picked by intuition is as much a
  guess as the current order. **BREAKING** if a new order is adopted.
- Seed-preserving optimisations (golden trace and hashes must stay unchanged):
  priority queue for pending cells, multi-step rewind, early break out of
  propagation on contradiction, faster template construction, fix of
  `Template::stats`.
- Every performance claim is backed by benchmark or corpus measurements;
  changes that do not improve anything are reverted unless they enable a
  later improvement.
- Measure (prototype only) the gain of flat domain storage after the other
  improvements, to decide whether a follow-up or an `expl_hexgrid` proposal
  is worth it.
- **BREAKING** (seed output): search improvements that change which map a
  seed produces, aimed at the hard regions backjumping leaves: forward
  checking when selecting a tile and, after an explicit decision gate, local
  region reset and/or full conflict-directed backjumping. Each such change
  regenerates the golden trace and hashes intentionally.
- Each improvement is classified as seed-preserving or seed-breaking up front;
  a seed only has to reproduce the same map on the same version.

Out of scope: chunked generation, full arc consistency (AC-4), whole-map
restarts, sizes beyond 1024x1024, and changes to the map crate or game
(which stays at 30x24 for now).

## Capabilities

### New Capabilities
- `wfc-generation`: the WFC map generator's guarantees: deterministic output
  per seed within a version, completion for every seed at supported sizes
  with bounded steps per cell, and the seed-impact policy for changes.

### Modified Capabilities

## Impact

- Code: `crates/expl_wfc` (`generator.rs`, `cell.rs`, `template.rs`), its
  benches, example and tests (`res/trace.jsonl` plus new golden hashes).
- Public API: `Generator::pending` and `Generator::collapsed` are `pub` and
  read by the tests; their types are expected to change.
  `Generator::grid` keeps its type.
- `expl_hexgrid`, `expl_map` and the game: no changes required.
- Seeds: seed-breaking phases change the map produced by existing seed
  strings; the seed encoding itself is unchanged.
- No new runtime dependencies expected; game code is unaffected.
