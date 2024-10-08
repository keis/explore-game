use super::SceneState;
use crate::{actor::Members, creature::Corpse, inventory::Inventory, structure::SafeHaven};
use bevy::prelude::*;
use expl_map::MapPresence;

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct Score {
    pub survivors: u32,
    pub dead: u32,
    pub crystals: u32,
}

pub fn game_over(
    mut commands: Commands,
    mut scene_state: ResMut<NextState<SceneState>>,
    group_query: Query<&Members, With<MapPresence>>,
    safe_haven_query: Query<(&Members, &Inventory), With<SafeHaven>>,
    corpse_query: Query<&Corpse>,
) {
    if group_query.iter().any(|m| !m.is_empty()) {
        return;
    }
    let mut score = Score::default();
    for (members, inventory) in &safe_haven_query {
        score.survivors += members.len() as u32;
        score.crystals += inventory.count_item(Inventory::CRYSTAL);
    }
    score.dead = corpse_query.iter().count() as u32;
    commands.insert_resource(score);
    scene_state.set(SceneState::GameOver);
}
