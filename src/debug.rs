//! Game-specific BRP methods for agent-driven testing.
//!
//! Stock BRP covers generic ECS operations, but game flows (menu buttons,
//! picking-based selection) go through states and observers that generic
//! methods can't reach (0.17 extras has no mouse support and no
//! `write_message`). These methods replay the exact code paths the UI
//! would trigger. Only registered when the `brp` feature is enabled.
use crate::{input::Select, interface::InterfaceState, scene::SceneState};
use bevy::prelude::*;
use bevy::remote::{error_codes::INVALID_PARAMS, BrpResult, RemoteMethodSystemId, RemoteMethods};
use serde_json::Value;

/// Registers the `game/*` methods. Must run after `RemotePlugin` (added by
/// `BrpExtrasPlugin`) so the `RemoteMethods` resource exists.
pub struct DebugBrpPlugin;

impl Plugin for DebugBrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_debug_methods);
    }
}

fn register_debug_methods(world: &mut World) {
    let new_game = world.register_system(new_game_handler);
    let select = world.register_system(select_handler);
    let mut methods = world.resource_mut::<RemoteMethods>();
    methods.insert("game/new_game", RemoteMethodSystemId::Instant(new_game));
    methods.insert("game/select", RemoteMethodSystemId::Instant(select));
}

/// Replicates the New game menu button: back to shell UI, regenerate world.
fn new_game_handler(
    _: In<Option<Value>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
) -> BrpResult {
    next_interface_state.set(InterfaceState::Shell);
    next_scene_state.set(SceneState::Reset);
    Ok(Value::Null)
}

/// Replicates clicking an entity: `{"entity": 4294967298}`.
fn select_handler(In(params): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let entity: Entity = params
        .as_ref()
        .and_then(|p| p.get("entity"))
        .and_then(|e| serde_json::from_value(e.clone()).ok())
        .ok_or_else(|| bevy::remote::BrpError {
            code: INVALID_PARAMS,
            message: "expected {\"entity\": <u64 id>}".to_string(),
            data: None,
        })?;
    commands.trigger(Select { entity });
    Ok(Value::Null)
}
