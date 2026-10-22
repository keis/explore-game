use crate::floating_text::FloatingTextSource;
use bevy::prelude::*;
use expl_hexgrid::HexCoord;
use smallvec::SmallVec;

#[derive(Component)]
#[require(Transform, FloatingTextSource)]
pub struct Combat {
    pub(super) position: HexCoord,
    pub(super) initiative_order: SmallVec<[Entity; 8]>,
    pub(super) initiative: usize,
}

#[cfg(test)]
mod tests {
    use super::Combat;
    use crate::floating_text::FloatingTextSource;
    use bevy::prelude::*;
    use expl_hexgrid::HexCoord;
    use smallvec::SmallVec;

    #[test]
    fn combat_requires_presentation() {
        let mut app = App::new();
        let entity = app
            .world_mut()
            .spawn(Combat {
                position: HexCoord::ZERO,
                initiative_order: SmallVec::new(),
                initiative: 0,
            })
            .id();
        let entity = app.world().entity(entity);
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<FloatingTextSource>());
    }
}
