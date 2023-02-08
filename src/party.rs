use crate::{
    character::Movement,
    indicator::Indicator,
    map::{Offset, PathGuided, ViewRadius},
    slide::Slide,
};

use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use smallvec::SmallVec;

#[derive(Component, Debug, Default)]
pub struct Party {
    pub name: String,
    pub supplies: u32,
    pub members: SmallVec<[Entity; 8]>,
}

#[derive(Bundle, Default)]
pub struct PartyBundle {
    pub party: Party,
    pub movement: Movement,
    pub pickable_bundle: PickableBundle,
    pub indicator: Indicator,
    pub offset: Offset,
    pub view_radius: ViewRadius,
    pub path_guided: PathGuided,
    pub slide: Slide,
}

#[derive(Component)]
pub struct PartyMember {
    pub party: Entity,
}

pub struct JoinParty {
    pub party: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

impl Command for JoinParty {
    fn write(mut self, world: &mut World) {
        for &member in &self.members {
            let mut member_entity = world.entity_mut(member);
            if let Some(mut party_member) = member_entity.get_mut::<PartyMember>() {
                party_member.party = self.party;
            } else {
                member_entity.insert(PartyMember { party: self.party });
            }
        }

        let mut party_entity = world.entity_mut(self.party);
        if let Some(mut party) = party_entity.get_mut::<Party>() {
            party.members.append(&mut self.members);
        }
    }
}

pub fn derive_party_movement(
    mut party_query: Query<(&Party, &mut Movement), Changed<Party>>,
    movement_query: Query<&Movement, Without<Party>>,
) {
    for (party, mut party_movement) in party_query.iter_mut() {
        party_movement.points = movement_query
            .iter_many(&party.members)
            .map(|m| m.points)
            .min()
            .unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::{derive_party_movement, JoinParty, Movement, Party, PartyMember};
    use bevy::{ecs::system::Command, prelude::*};
    use rstest::*;
    use smallvec::SmallVec;

    #[fixture]
    fn app() -> App {
        let mut app = App::new();
        app.add_system(derive_party_movement);
        let party_entity = app
            .world
            .spawn((Party::default(), Movement::default()))
            .id();
        let member_entity = app.world.spawn(Movement { points: 2 }).id();
        let joinparty = JoinParty {
            party: party_entity,
            members: SmallVec::from_slice(&[member_entity]),
        };
        joinparty.write(&mut app.world);
        app
    }

    #[rstest]
    fn join_party(mut app: App) {
        let (party_entity, party) = app.world.query::<(Entity, &Party)>().single(&app.world);
        assert_eq!(party.members.len(), 1);
        let member_from_party_entity = party.members[0];

        let (member_entity, member) = app
            .world
            .query::<(Entity, &PartyMember)>()
            .single(&app.world);

        assert_eq!(member_from_party_entity, member_entity);
        assert_eq!(member.party, party_entity);
    }

    #[rstest]
    fn party_movement(mut app: App) {
        let (mut movement, _member) = app
            .world
            .query::<(&mut Movement, &PartyMember)>()
            .single_mut(&mut app.world);
        movement.points = 3;

        app.update();

        let (party_movement, _party) = app.world.query::<(&Movement, &Party)>().single(&app.world);
        assert_eq!(party_movement.points, 3);
    }
}
