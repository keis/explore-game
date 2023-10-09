use super::component::*;
use crate::map::{HexCoord, MapPresence, ViewRadius};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub struct Target<'w, 's> {
    presence_query: Query<'w, 's, &'static MapPresence, With<Party>>,
}

impl<'w, 's> Target<'w, 's> {
    pub fn closest_in_view(
        &self,
        position: HexCoord,
        view_radius: &ViewRadius,
    ) -> Option<&MapPresence> {
        self.presence_query
            .iter()
            .filter(|&other| position.distance(other.position) <= view_radius.0)
            .min_by_key(|other| position.distance(other.position))
    }
}
