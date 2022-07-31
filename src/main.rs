use bevy::render::{camera::ScalingMode, texture::ImageSettings};
use bevy::{prelude::*, window::PresentMode};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

fn main() {
    let height = 900.0;

    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            width: height * ASPECT_RATIO,
            height,
            title: "Explore Game".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..default()
        })
        .add_startup_system_to_stage(StartupStage::PreStartup, load_ascii)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_plugins(DefaultPlugins)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            top: 1.0,
            bottom: -1.0,
            right: 1.0 * ASPECT_RATIO,
            left: -1.0 * ASPECT_RATIO,
            scaling_mode: ScalingMode::None,
            ..default()
        },
        ..default()
    });
}

struct AsciiSheet(Handle<TextureAtlas>);

fn load_ascii(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Ascii.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::splat(9.0),
        16,
        16,
        Vec2::splat(2.0),
        Vec2::splat(0.0),
    );

    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(AsciiSheet(atlas_handle));
}

fn spawn_player(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let mut sprite = TextureAtlasSprite::new(1);
    sprite.color = Color::rgb(0.3, 0.3, 0.9);
    sprite.custom_size = Some(Vec2::splat(1.0));

    let player = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 900.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Player"))
        .id();

    let mut background_sprite = TextureAtlasSprite::new(1);
    background_sprite.color = Color::rgb(0.5, 0.5, 0.5);
    background_sprite.custom_size = Some(Vec2::splat(1.0));

    let background = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: background_sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -1.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Player"))
        .id();

    commands.entity(player).push_children(&[background]);
}
