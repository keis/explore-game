mod asset;
mod command;
mod component;
mod event;
mod hex;
mod plugin;
mod system;

pub use asset::*;
pub use command::*;
pub use component::*;
pub use event::*;
pub use expl_hexgrid::{layout::SquareGridLayout, HexCoord};
pub use plugin::MapPlugin;

#[cfg(test)]
pub mod tests {
    use super::ZoneLayer;
    use crate::terrain::Terrain;
    use bevy::prelude::*;
    use expl_hexgrid::layout::SquareGridLayout;
    use rstest::*;

    pub fn spawn_game_map(app: &mut App) -> Entity {
        let tiles = app
            .world
            .spawn_batch(vec![
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Ocean,
                Terrain::Ocean,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
            ])
            .collect();
        app.world
            .spawn(ZoneLayer::new(
                SquareGridLayout {
                    width: 3,
                    height: 3,
                },
                tiles,
            ))
            .id()
    }

    #[fixture]
    pub fn app() -> App {
        let mut app = App::new();
        spawn_game_map(&mut app);
        app
    }
}
