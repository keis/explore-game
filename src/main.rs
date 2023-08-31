use bevy::{
    asset::ChangeWatcher, log::LogPlugin, prelude::*, utils::Duration, window::PresentMode,
};
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::prelude::RaycastPickCamera;
use clap::Parser;
use expl_wfc::{Seed, SeedType};
use explore_game::{
    action::ActionPlugin,
    assets::{AssetState, MainAssets},
    camera::{CameraBounds, CameraControl, CameraControlPlugin},
    character::reset_movement_points,
    combat::CombatPlugin,
    enemy::move_enemy,
    input::InputPlugin,
    inspector::InspectorPlugin,
    interface::InterfacePlugin,
    light,
    map::{start_map_generation, MapPlugin, MapSeed},
    material::MaterialPlugins,
    party::{derive_party_movement, despawn_empty_party},
    scene::ScenePlugin,
    slide::{slide, SlideEvent},
    structure::StructurePlugin,
    turn::Turn,
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
                    primary_window: Some(Window {
                        resolution: (height * ASPECT_RATIO, height).into(),
                        title: "Explore Game".to_string(),
                        present_mode: PresentMode::Fifo,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                // See https://github.com/Leafwing-Studios/leafwing-input-manager/issues/285
                .set(LogPlugin {
                    filter: "wgpu=error,bevy_ecs::event=error".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                }),
        )
        .add_plugins(MaterialPlugins)
        .add_state::<AssetState>()
        .add_plugins((
            bevy_mod_billboard::prelude::BillboardPlugin,
            bevy_mod_outline::OutlinePlugin,
            bevy_obj::ObjPlugin,
            bevy_sprite3d::Sprite3dPlugin,
            noisy_bevy::NoisyShaderPlugin,
        ))
        .add_plugins((
            ActionPlugin,
            CameraControlPlugin,
            CombatPlugin,
            InputPlugin,
            InspectorPlugin,
            InterfacePlugin,
            MapPlugin,
            StructurePlugin,
            ScenePlugin,
        ))
        .add_systems(
            Startup,
            (spawn_camera, light::spawn_light, start_map_generation),
        )
        .add_event::<SlideEvent>()
        .add_loading_state(
            LoadingState::new(AssetState::Loading).continue_to_state(AssetState::Loaded),
        )
        .add_collection_to_loading_state::<_, MainAssets>(AssetState::Loading)
        .add_systems(
            Update,
            (
                reset_movement_points.run_if(resource_changed::<Turn>()),
                derive_party_movement,
                despawn_empty_party,
                move_enemy.run_if(resource_changed::<Turn>()),
                slide,
            ),
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
        RaycastPickCamera::default(),
    ));
}
