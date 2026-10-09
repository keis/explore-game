# AGENTS.md

Bevy 0.17 hex-grid tactics game. Workspace: root `explore_game` binary plus
libraries in `crates/` (`expl_hexgrid`, `expl_wfc`, `expl_map`, `expl_codex`,
`expl_databinding`, `expl_hexagon`). See `HACKING.md` for architecture and
component naming

## Always run before finishing

```sh
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
cargo deny check bans
```

Keep commits small and individual with imperative messages
(e.g. `Use children! macro for role child spawns`).

## Debugging the live game via BRP MCP

The game exposes the Bevy Remote Protocol when built with the `brp` feature
(`bevy_brp_extras` 0.17.2, port 15702 or `$BRP_EXTRAS_PORT`):

```sh
cargo run --features brp
```

Useful flows:

* Find things: `world_query` / `world_find_entities_by_name`
  (e.g. count `Party` entities after a New game).
* See it: `brp_extras_screenshot` to `/tmp/*.png`, then read the image.
  The window must be visible; occluded windows capture black.
* Touch it: `world_mutate_components`, `world_trigger_event`,
  `brp_extras_send_keys`. Clicks go through stock `write_message` with
  `WindowEvent`s (0.17 extras has no mouse methods). For menu and picking
  flows, prefer the app-defined `game/new_game` and `game/select`
  (see `src/debug.rs`) — they replay the exact UI code paths.
* Follow it: `world_get_components_watch` streams changes to a log file;
  read game logs instead of blocking on the process.
* Formats: component/resource JSON shapes come from `brp_type_guide`
  (spawn/insert examples, mutation paths) — don't hand-write them.

Gotchas:

* Query/insert only works for `Reflect + Serialize/Deserialize` types that
  are `register_type`d (generics still need manual registration; see
  `src/scene/save.rs` for the pattern).
* `brp_extras/*` methods are auto-discovered; `rpc.discover` lists
  everything including app-defined methods.
* `cargo run` blocks — launch the game detached and poll, or use the MCP
  launch/log tools.
