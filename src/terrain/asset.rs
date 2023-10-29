use bevy::prelude::*;
use expl_hexagon::Hexagon;

#[derive(Resource)]
pub struct HexAssets {
    pub mesh: Handle<Mesh>,
}

pub fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon {
            radius: 1.0,
            subdivisions: 2,
        })),
    });
}
