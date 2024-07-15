use bevy::{log::LogPlugin, prelude::*, window::PresentMode};
use clap::Parser;
use expl_wfc::Seed;
use explore_game::{map_generator::MapSeed, material::MaterialPlugins, plugins::ExplPlugins};

pub const CLEAR: Color = Color::srgb(0.1, 0.1, 0.1);

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
        .add_plugins((ExplPlugins, MaterialPlugins))
        .run();
}
