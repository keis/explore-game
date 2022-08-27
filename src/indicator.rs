use bevy::prelude::*;

#[derive(Component)]
pub struct Indicator;

pub fn update_indicator(
    time: Res<Time>,
    mut indicator_query: Query<&mut Transform, With<Indicator>>,
) {
    for mut indicator_transform in indicator_query.iter_mut() {
        indicator_transform.rotate(Quat::from_rotation_y(time.delta_seconds() / 2.0));
    }
}
