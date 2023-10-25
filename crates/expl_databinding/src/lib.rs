use bevy_ecs::{
    prelude::*,
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::{Command, EntityCommands, SystemParam},
};
use core::slice;
use smallvec::SmallVec;

/// Registry of entities bound to the data of the entity with this component.
#[derive(Component)]
pub struct DataBindings(SmallVec<[Entity; 4]>);

impl<'a> IntoIterator for &'a DataBindings {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Command to bind sink to source
struct Bind {
    source: Entity,
    sink: Entity,
}

impl Command for Bind {
    fn apply(self, world: &mut World) {
        let mut source_entity = world.entity_mut(self.source);
        if let Some(mut data_bindings) = source_entity.get_mut::<DataBindings>() {
            data_bindings.0.push(self.sink);
        } else {
            source_entity.insert(DataBindings(SmallVec::from_slice(&[self.sink])));
        }
    }
}

pub trait DataBindingExt {
    fn bind_to(&mut self, source: Entity) -> &mut Self;
}

impl<'w, 's, 'a> DataBindingExt for EntityCommands<'w, 's, 'a> {
    /// Bind the entity to another entity `source`
    fn bind_to(&mut self, source: Entity) -> &mut Self {
        let sink = self.id();
        self.commands().add(Bind { source, sink });
        self
    }
}

/// System parameter that provides a mechanism of updating entities with data bindings.
///
/// `Source` is the components to fetch in the source query.
/// `Filter` is the filter to apply to the source query. e.g Changed<>
/// `Sink` is the components to fetch mutably in the sink query.
#[derive(SystemParam)]
pub struct DataBindingUpdate<'w, 's, Source, Sink, Filter>
where
    Source: WorldQuery + 'static,
    Sink: WorldQuery + 'static,
    Filter: ReadOnlyWorldQuery + 'static,
{
    source_query: Query<'w, 's, (&'static DataBindings, Source), Filter>,
    sink_query: Query<'w, 's, Sink>,
}

impl<'w, 's, Source, Sink, Filter> DataBindingUpdate<'w, 's, Source, Sink, Filter>
where
    Source: WorldQuery<ReadOnly = Source> + 'static,
    Sink: WorldQuery + 'static,
    Filter: ReadOnlyWorldQuery + 'static,
{
    /// Apply the given function `f` to each binding of each matching source entity.
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&<Source::ReadOnly as WorldQuery>::Item<'_>, &mut Sink::Item<'_>),
    {
        for (bindings, source_data) in &self.source_query {
            for &entity in bindings {
                if let Ok(mut sink_data) = self.sink_query.get_mut(entity) {
                    (f)(&source_data, &mut sink_data);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct Temperature(u16);

    #[derive(Component)]
    struct Display(String, u16);

    fn configure(mut commands: Commands) {
        let temp1 = commands.spawn(Temperature(21)).id();
        let temp2 = commands.spawn(Temperature(17)).id();
        commands.spawn(Display("--".to_string(), 0)).bind_to(temp1);
        commands.spawn(Display("--".to_string(), 0)).bind_to(temp1);
        commands.spawn(Display("--".to_string(), 0)).bind_to(temp2);
    }

    fn update_display(
        mut data_binding_update: DataBindingUpdate<
            &Temperature,
            &mut Display,
            Changed<Temperature>,
        >,
    ) {
        data_binding_update.for_each(|temp, display| {
            display.0 = format!("{} °C", temp.0);
            display.1 += 1;
        });
    }

    fn update_temperature(mut temperature_query: Query<&mut Temperature>) {
        for mut temp in &mut temperature_query {
            if temp.0 > 19 {
                temp.0 -= 1;
            }
        }
    }

    #[test]
    fn configure_bindings() {
        let mut world = World::default();

        let mut schedule = Schedule::default();
        schedule.add_systems(configure);
        schedule.run(&mut world);

        let temps: Vec<_> = world
            .query::<(&Temperature, &DataBindings)>()
            .iter(&world)
            .collect();
        assert_eq!(temps.len(), 2);
        assert_eq!(temps[0].1 .0.len(), 2);
        assert_eq!(temps[1].1 .0.len(), 1);

        let displays: Vec<_> = world.query::<&Display>().iter(&world).collect();
        assert_eq!(displays.len(), 3);
    }

    #[test]
    fn update_sink() {
        let mut world = World::default();

        let mut schedule = Schedule::default();
        schedule.add_systems(configure);
        schedule.run(&mut world);

        let mut schedule = Schedule::default();
        schedule.add_systems((update_temperature, update_display.after(update_temperature)));

        schedule.run(&mut world);
        let displays: Vec<_> = world
            .query::<&Display>()
            .iter(&world)
            .map(|display| display.1)
            .collect();
        assert_eq!(displays, [1, 1, 1]);

        schedule.run(&mut world);
        let displays: Vec<_> = world
            .query::<&Display>()
            .iter(&world)
            .map(|display| display.1)
            .collect();
        assert_eq!(displays, [2, 2, 1]);

        schedule.run(&mut world);
        let displays: Vec<_> = world
            .query::<&Display>()
            .iter(&world)
            .map(|display| display.1)
            .collect();
        assert_eq!(displays, [2, 2, 1]);
    }
}
