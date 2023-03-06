use bevy::{log::LogPlugin, prelude::*, window::PresentMode};
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
use clap::Parser;
use expl_wfc::{Seed, SeedType};
use explore_game::{
    action::ActionPlugin,
    assets::MainAssets,
    camera::{CameraBounds, CameraControl, CameraControlPlugin},
    camp::update_camp_view_radius,
    character::reset_movement_points,
    combat,
    enemy::move_enemy,
    indicator::update_indicator,
    input::InputPlugin,
    interface::InterfacePlugin,
    light,
    map::{start_map_generation, MapPlugin, MapSeed},
    material::{TerrainMaterialPlugin, ZoneMaterialPlugin},
    party::{derive_party_movement, despawn_empty_party},
    scene,
    slide::{slide, SlideEvent},
    turn::Turn,
    State,
};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    seed: Option<Seed>,
}

fn main() {
    let cli = Cli::parse();
    let height = 900.0;
    let seed = cli
        .seed
        .unwrap_or_else(|| Seed::new(SeedType::Square(30, 24)));

    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(Turn { number: 0 })
        .insert_resource(MapSeed(seed))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: height * ASPECT_RATIO,
                        height,
                        title: "Explore Game".to_string(),
                        present_mode: PresentMode::Fifo,
                        resizable: false,
                        ..default()
                    },
                    ..default()
                })
                // See https://github.com/Leafwing-Studios/leafwing-input-manager/issues/285
                .set(LogPlugin {
                    filter: "wgpu=error,bevy_ecs::event=error".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                }),
        )
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(CameraControlPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(InterfacePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(ZoneMaterialPlugin)
        .add_plugin(TerrainMaterialPlugin)
        .add_plugin(ActionPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(light::spawn_light)
        .add_startup_system(start_map_generation)
        .add_event::<SlideEvent>()
        .add_state(State::AssetLoading)
        .add_loading_state(
            LoadingState::new(State::AssetLoading)
                .continue_to_state(State::Running)
                .with_collection::<MainAssets>(),
        )
        .add_system_set(
            SystemSet::on_update(State::Running)
                .with_system(scene::spawn_scene)
                .with_system(update_indicator)
                .with_system(reset_movement_points)
                .with_system(derive_party_movement)
                .with_system(despawn_empty_party)
                .with_system(move_enemy)
                .with_system(update_camp_view_radius)
                .with_system(combat::initiate_combat)
                .with_system(combat::combat_round)
                .with_system(combat::despawn_no_health.after(combat::combat_round))
                .with_system(combat::finish_combat.after(combat::despawn_no_health))
                .with_system(slide),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(30.0, 10.0, 30.0);
    let lookto = Vec3::new(-2.0, -20.0, -20.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation)
                .looking_at(translation + lookto, Vec3::Y),
            ..default()
        },
        CameraBounds {
            position: Vec3::new(0.0, 5.0, 10.0),
            extent: Vec3::new(40.0, 25.0, 40.0),
            gap: 1.0,
        },
        CameraControl::default(),
        PickingCameraBundle::default(),
    ));
}
