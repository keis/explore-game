use super::SceneState;
use crate::{actor::Members, creature::Corpse, inventory::Inventory, structure::SafeHaven};
use bevy::prelude::*;
use expl_map::MapPresence;

#[derive(Resource, Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Resource)]
pub struct Score {
    pub survivors: u32,
    pub dead: u32,
    pub crystals: u32,
}

/// Detects game over condition and queues the transition.
pub fn game_over(
    mut scene_state: ResMut<NextState<SceneState>>,
    group_query: Query<&Members, With<MapPresence>>,
) {
    if group_query.iter().any(|m| !m.is_empty()) {
        return;
    }
    scene_state.set(SceneState::GameOver);
}

/// Tallies the score from the live world when GameOver is entered.
pub fn tally_score(
    mut commands: Commands,
    safe_haven_query: Query<(&Members, &Inventory), With<SafeHaven>>,
    corpse_query: Query<&Corpse>,
) {
    let mut score = Score::default();
    for (members, inventory) in &safe_haven_query {
        score.survivors += members.len() as u32;
        score.crystals += inventory.count_item(Inventory::CRYSTAL);
    }
    score.dead = corpse_query.iter().count() as u32;
    commands.insert_resource(score);
}

/// Removes `Score` when its state is left, so no stale score lingers in
/// unrelated states.
pub fn clear_score(mut commands: Commands) {
    commands.remove_resource::<Score>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<SceneState>();
        app
    }

    fn enter_game_over(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<SceneState>>()
            .set(SceneState::GameOver);
        app.update();
        assert_eq!(
            *app.world().resource::<State<SceneState>>().get(),
            SceneState::GameOver
        );
    }

    #[test]
    fn detector_queues_game_over_when_no_party() {
        let mut app = test_app();
        app.add_systems(Update, game_over);

        // Detection runs in Update; the queued transition applies next frame.
        app.update();
        app.update();

        assert_eq!(
            *app.world().resource::<State<SceneState>>().get(),
            SceneState::GameOver
        );
    }

    #[test]
    fn tally_scores_empty_world_as_default() {
        let mut app = test_app();
        app.add_systems(OnEnter(SceneState::GameOver), tally_score);

        enter_game_over(&mut app);

        assert_eq!(
            app.world().resource::<Score>(),
            &Score::default(),
            "direct GameOver entry tallies whatever exists"
        );
    }

    #[test]
    fn tally_counts_havens_and_corpses() {
        let mut app = test_app();
        app.add_systems(OnEnter(SceneState::GameOver), tally_score);
        let mut inventory = Inventory::default();
        inventory.add_item(Inventory::CRYSTAL, 5);
        inventory.add_item(Inventory::SUPPLY, 3);
        app.world_mut()
            .spawn((SafeHaven, Members::default(), inventory));
        app.world_mut().spawn(Corpse);
        app.world_mut().spawn(Corpse);

        enter_game_over(&mut app);

        assert_eq!(
            app.world().resource::<Score>(),
            &Score {
                survivors: 0,
                dead: 2,
                crystals: 5,
            }
        );
    }

    #[test]
    fn tally_overwrites_stale_score() {
        let mut app = test_app();
        app.add_systems(OnEnter(SceneState::GameOver), tally_score);
        app.insert_resource(Score {
            survivors: 9,
            dead: 9,
            crystals: 9,
        });

        enter_game_over(&mut app);

        assert_eq!(
            app.world().resource::<Score>(),
            &Score::default(),
            "tally always reflects the entered world, never a stale score"
        );
    }

    #[test]
    fn clear_score_removes_on_exit() {
        let mut app = test_app();
        app.add_systems(OnEnter(SceneState::GameOver), tally_score);
        app.add_systems(OnExit(SceneState::GameOver), clear_score);
        enter_game_over(&mut app);
        assert!(app.world().get_resource::<Score>().is_some());

        app.world_mut()
            .resource_mut::<NextState<SceneState>>()
            .set(SceneState::Active);
        app.update();

        assert!(
            app.world().get_resource::<Score>().is_none(),
            "leaving GameOver must not leave a stale score behind"
        );
    }
}
