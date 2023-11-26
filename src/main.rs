use bevy::{log::LogPlugin, prelude::*, window::PresentMode};
use bevy_asset_loader::prelude::*;
use clap::Parser;
use expl_wfc::Seed;
use explore_game::{
    action::ActionPlugin,
    actor::ActorPlugin,
    assets::{AssetState, MainAssets},
    camera::CameraControlPlugin,
    combat::CombatPlugin,
    enemy::EnemyPlugin,
    input::InputPlugin,
    inspector::InspectorPlugin,
    interface::InterfacePlugin,
    inventory::InventoryPlugin,
    map::MapPlugin,
    map_generator::{MapGeneratorPlugin, MapSeed},
    material::MaterialPlugins,
    path::PathPlugin,
    scene::{ScenePlugin, SceneSet, SceneState},
    structure::StructurePlugin,
    terrain::TerrainPlugin,
    turn::TurnPlugin,
};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    seed: Option<Seed>,
}

fn main() {
    let cli = Cli::parse();

    App::new()
        .insert_resource(ClearColor(CLEAR))
        .add_systems(Startup, move |mut commands: Commands| {
            if let Some(seed) = cli.seed {
                commands.spawn(MapSeed(seed));
            }
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Explore Game".to_string(),
                        present_mode: PresentMode::Fifo,
                        ..default()
                    }),
                    ..default()
                })
                // See https://github.com/Leafwing-Studios/leafwing-input-manager/issues/285
                .set(LogPlugin {
                    filter: "wgpu=error,bevy_ecs::event=error".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
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
            ActorPlugin,
            CameraControlPlugin,
            CombatPlugin,
            InputPlugin,
            InspectorPlugin,
            InterfacePlugin,
            InventoryPlugin,
        ))
        .add_plugins((
            MapGeneratorPlugin,
            MapPlugin {
                setup_schedule: OnEnter(SceneState::Active),
                setup_set: SceneSet::Populate,
            },
            PathPlugin,
            ScenePlugin,
            StructurePlugin,
            TerrainPlugin,
            TurnPlugin,
            EnemyPlugin,
        ))
        .add_loading_state(
            LoadingState::new(AssetState::Loading).continue_to_state(AssetState::Loaded),
        )
        .add_collection_to_loading_state::<_, MainAssets>(AssetState::Loading)
        .run();
}
