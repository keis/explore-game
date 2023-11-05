mod asset;
mod bundle;
mod component;
mod plugin;
mod system;
mod system_param;

pub use bundle::*;
pub use component::*;
pub use plugin::TerrainPlugin;
pub use system_param::*;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use bevy::prelude::*;

    #[test]
    fn test_height() {
        let height = Height {
            height_amp: 0.5,
            height_base: 0.2,
            outer_amp: Outer::new(&[0.1, 0.2, 0.3, 0.5, 0.3, 0.2]),
            outer_base: Outer::new(&[0.0, 0.1, 0.1, 0.2, 0.2, 0.0]),
        };

        assert_abs_diff_eq!(
            height.height_at(Vec2::splat(0.0), Vec2::splat(0.0)),
            0.45,
            epsilon = 0.01
        );
        assert_abs_diff_eq!(
            height.height_at(Vec2::new(0.85, 0.45), Vec2::new(0.85, 0.45)),
            0.08,
            epsilon = 0.01
        );
        assert_abs_diff_eq!(
            height.height_at(Vec2::new(0.0, 0.9), Vec2::new(0.0, 0.9)),
            0.25,
            epsilon = 0.01
        );
        assert_abs_diff_eq!(
            height.height_at(Vec2::new(0.8, 0.0), Vec2::new(0.8, 0.0)),
            0.069,
            epsilon = 0.01
        );
    }

    #[test]
    fn test_amp_base() {
        let height = Height {
            height_amp: 0.5,
            height_base: 0.2,
            outer_amp: Outer::new(&[0.3, 0.2, 0.3, 0.5, 0.3, 0.2]),
            outer_base: Outer::new(&[0.3, 0.1, 0.1, 0.2, 0.2, 0.0]),
        };

        let (amp, base) = height.amp_and_base(Vec2::splat(0.0));
        assert_abs_diff_eq!(amp, 0.5, epsilon = 0.001);
        assert_abs_diff_eq!(base, 0.2, epsilon = 0.001);

        let (amp, base) = height.amp_and_base(Vec2::new(0.85, 0.45));
        assert_abs_diff_eq!(amp, 0.2, epsilon = 0.001);
        assert_abs_diff_eq!(base, 0.1, epsilon = 0.001);

        let (amp, base) = height.amp_and_base(Vec2::new(0.0, 1.0));
        assert_abs_diff_eq!(amp, 0.2, epsilon = 0.001);
        assert_abs_diff_eq!(base, 0.1, epsilon = 0.001);

        let (amp, base) = height.amp_and_base(Vec2::new(0.0, -1.0));
        assert_abs_diff_eq!(amp, 0.2, epsilon = 0.001);
        assert_abs_diff_eq!(base, 0.0, epsilon = 0.001);

        // TODO: Should really be the edge 0.3
        let (amp, base) = height.amp_and_base(Vec2::new(0.8, 0.2));
        assert_abs_diff_eq!(amp, 0.2, epsilon = 0.001);
        assert_abs_diff_eq!(base, 0.1, epsilon = 0.001);
    }
}
