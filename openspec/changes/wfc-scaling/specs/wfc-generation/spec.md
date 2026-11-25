# Spec Delta

## Purpose

Defines the guarantees of the WFC map generator: which maps a seed
produces, that generation completes predictably at supported map sizes, and
how changes to generated output are managed.

## ADDED Requirements

### Requirement: Deterministic output per seed
Given the same template and seed, the generator SHALL produce the identical
map and take the identical number of steps on every run of the same version.

#### Scenario: Repeated generation with one seed
- **WHEN** a map is generated twice from the same template and seed on the same version
- **THEN** both exported maps are identical cell for cell
- **AND** both runs report the same step count

### Requirement: Seed output is versioned
Seed strings SHALL keep their encoding across versions, but the map a seed
produces MAY change between versions. Any change to the produced map SHALL
be intentional and recorded by updating the golden fixtures in the same
change.

#### Scenario: Seed-preserving change
- **WHEN** a change classified as seed-preserving is applied
- **THEN** every golden trace and golden output hash still matches without being regenerated

#### Scenario: Seed-breaking change
- **WHEN** a change classified as seed-breaking is applied
- **THEN** the golden trace and golden output hashes are regenerated in that same change
- **AND** existing seed strings still parse to the same seed

### Requirement: Golden checks cover backtracking
The test suite SHALL include golden output checks for seeds that backtrack
heavily, in addition to the fine-grained trace. Golden fixtures SHALL
record the map and its collapse events, not step counts. Checks too slow
for the regular test run SHALL be opt-in rather than removed.

#### Scenario: Regular test run
- **WHEN** the regular test suite runs
- **THEN** the fine-grained golden trace is checked
- **AND** the heavy golden checks are skipped

#### Scenario: Opt-in heavy checks
- **WHEN** the ignored tests are run explicitly
- **THEN** the exported map hash and collapse-event count of each heavy golden seed are compared against their fixtures

#### Scenario: Step granularity is not a fixture
- **WHEN** a change alters how many steps generation takes but not the produced map or its collapse events
- **THEN** every golden fixture still matches without being regenerated

#### Scenario: Partial match is a failure
- **WHEN** a run records fewer or more collapse events than the golden trace
- **THEN** the golden check fails

#### Scenario: Regular runs never rewrite fixtures
- **WHEN** the test suite runs without the explicit regenerate switch
- **THEN** no golden fixture file is modified

### Requirement: Generated maps respect the template
Every map the generator completes SHALL contain only cells whose tiles are
pairwise compatible with each neighbour according to the template,
including after any backtracking or local repair.

#### Scenario: Neighbour compatibility after repair
- **WHEN** generation completes for a seed that required backtracking or local repair
- **THEN** for every pair of neighbouring cells the chosen tiles are compatible in that direction

### Requirement: Completion at supported sizes
Every seed in the seed corpora for the supported square sizes, 30x24 (the
size the game ships) and 256x256, SHALL complete a fully collapsed map
within the default budget.

#### Scenario: Target size corpus
- **WHEN** every seed of the 256x256 seed corpus is generated with the default budget
- **THEN** each run completes with every cell collapsed

#### Scenario: Shipped size corpus
- **WHEN** every seed of the 30x24 seed corpus is generated with the default budget
- **THEN** each run completes with every cell collapsed

### Requirement: Bounded generation cost
Across a seed corpus at 256x256, the worst seed SHALL take no more than 3x
the steps of the median seed, and a release build SHALL generate each map
in about 1 second. This is a quality target measured offline over the
corpus; it is separate from the runtime collapse budget.

#### Scenario: Corpus tail
- **WHEN** the 256x256 seed corpus is run in a release build
- **THEN** the maximum step count is at most 3x the median step count
- **AND** each map is generated in about 1 second

### Requirement: Failure instead of hanging
The generator SHALL enforce a budget and stop with an error when a run
exceeds it, so degenerate runs cannot go on for hours. The budget is counted
in collapse events, never steps or wall-clock time, so the outcome is
deterministic per seed and does not depend on how work is split over steps.
A search that runs out of history to backtrack into SHALL also stop with an
error rather than panic.

#### Scenario: Exhausted budget
- **WHEN** generation exceeds its collapse budget without completing the map
- **THEN** generation stops and an error is returned to the caller

#### Scenario: Failure is reproducible
- **WHEN** a seed that exhausts its budget is generated again with the same template, budget and version
- **THEN** it fails again after the same number of collapse events

#### Scenario: Exhausted history
- **WHEN** generation needs to backtrack but no collapsed cell is left to undo
- **THEN** generation stops and an error is returned to the caller instead of panicking

### Requirement: Default budget scales with map size
Unless the caller sets a budget, the collapse budget SHALL be a fixed
multiple of the number of cells in the layout.

#### Scenario: Default budget
- **WHEN** a generator is created without an explicit budget
- **THEN** its collapse budget equals the default collapses-per-cell factor times the layout's cell count

### Requirement: Caller-configurable budget
The caller SHALL be able to set the collapse budget for a generator, to
accommodate templates that need more or fewer collapses per cell.

#### Scenario: Explicit budget overrides default
- **WHEN** a generator is created with an explicit collapse budget
- **THEN** generation stops with an error only once that budget is exceeded, regardless of the default factor

### Requirement: Generation statistics
The generator SHALL expose, for a run, the number of steps, the number of
collapse events, the number of rewinds, and the maximum backtrack depth.

#### Scenario: Statistics after a run
- **WHEN** generation of a map finishes
- **THEN** the caller can read the step count, collapse count, rewind count and maximum backtrack depth for that run
