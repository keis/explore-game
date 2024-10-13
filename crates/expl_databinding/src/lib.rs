use bevy_app::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryFilter},
    system::{EntityCommands, SystemParam},
    world::Command,
};
use bevy_reflect::prelude::*;
use core::slice;
use smallvec::SmallVec;

pub struct DataBindingPlugin;

impl Plugin for DataBindingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DataBindings>()
            .add_event::<DataBindingExpired>()
            .add_systems(
                PostUpdate,
                remove_expired_data_bindings.run_if(on_event::<DataBindingExpired>()),
            );
    }
}

#[derive(Event)]
struct DataBindingExpired {
    source: Entity,
    sink: Entity,
}

/// Registry of entities bound to the data of the entity with this component.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct DataBindings(SmallVec<[Entity; 4]>);

impl DataBindings {
    fn new(slice: &[Entity]) -> Self {
        Self(SmallVec::from_slice(slice))
    }

    fn bind(&mut self, entity: Entity) {
        self.0.push(entity);
    }

    fn unbind(&mut self, entity: Entity) {
        self.0.retain(|e| *e != entity);
    }
}

impl<'a> IntoIterator for &'a DataBindings {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
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
            data_bindings.bind(self.sink);
        } else {
            source_entity.insert(DataBindings::new(&[self.sink]));
        }
    }
}

pub trait DataBindingExt {
    fn bind_to(&mut self, source: Entity) -> &mut Self;
}

impl DataBindingExt for EntityCommands<'_> {
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
    Source: QueryData + 'static,
    Sink: QueryData + 'static,
    Filter: QueryFilter + 'static,
{
    source_query: Query<'w, 's, (Entity, &'static DataBindings, Source), Filter>,
    sink_query: Query<'w, 's, (Entity, Option<Sink>)>,
    expired_events: EventWriter<'w, DataBindingExpired>,
}

impl<Source, Sink, Filter> DataBindingUpdate<'_, '_, Source, Sink, Filter>
where
    Source: QueryData<ReadOnly = Source> + 'static,
    Sink: QueryData + 'static,
    Filter: QueryFilter + 'static,
{
    /// Apply the given function `f` to each binding of each matching source entity.
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&Source::Item<'_>, &mut Sink::Item<'_>),
    {
        for (source, bindings, source_data) in &self.source_query {
            for &sink in bindings {
                match self.sink_query.get_mut(sink) {
                    Ok((_, Some(mut sink_data))) => {
                        (f)(&source_data, &mut sink_data);
                    }
                    Ok((_, None)) => { /* Entity exists but without the desired components */ }
                    Err(_) => {
                        self.expired_events
                            .send(DataBindingExpired { source, sink });
                    }
                }
            }
        }
    }
}

fn remove_expired_data_bindings(
    mut expired_events: EventReader<DataBindingExpired>,
    mut data_bindings_query: Query<&mut DataBindings>,
) {
    for DataBindingExpired { source, sink } in expired_events.read() {
        let Ok(mut data_bindings) = data_bindings_query.get_mut(*source) else {
            continue;
        };
        data_bindings.unbind(*sink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

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
        let mut app = App::new();
        app.add_plugins(DataBindingPlugin);
        app.world_mut().run_system_once(configure);

        let temps: Vec<_> = app
            .world_mut()
            .query::<(&Temperature, &DataBindings)>()
            .iter(app.world())
            .collect();
        assert_eq!(temps.len(), 2);
        assert_eq!(temps[0].1 .0.len(), 2);
        assert_eq!(temps[1].1 .0.len(), 1);

        let displays: Vec<_> = app
            .world_mut()
            .query::<&Display>()
            .iter(app.world())
            .collect();
        assert_eq!(displays.len(), 3);
    }

    #[test]
    fn update_sink() {
        let mut app = App::new();
        app.add_plugins(DataBindingPlugin);
        app.world_mut().run_system_once(configure);

        let mut schedule = Schedule::default();
        schedule.add_systems((update_temperature, update_display.after(update_temperature)));

        schedule.run(app.world_mut());
        let displays: Vec<_> = app
            .world_mut()
            .query::<&Display>()
            .iter(app.world())
            .map(|display| display.1)
            .collect();
        assert_eq!(displays, [1, 1, 1]);

        schedule.run(app.world_mut());
        let displays: Vec<_> = app
            .world_mut()
            .query::<&Display>()
            .iter(app.world())
            .map(|display| display.1)
            .collect();
        assert_eq!(displays, [2, 2, 1]);

        schedule.run(app.world_mut());
        let displays: Vec<_> = app
            .world_mut()
            .query::<&Display>()
            .iter(app.world())
            .map(|display| display.1)
            .collect();
        assert_eq!(displays, [2, 2, 1]);
    }

    #[test]
    fn expire_deleted_entity() {
        let mut app = App::new();
        app.add_plugins(DataBindingPlugin);
        app.world_mut().run_system_once(configure);

        let display = app
            .world_mut()
            .query_filtered::<Entity, With<Display>>()
            .iter(app.world())
            .next()
            .unwrap();

        app.world_mut().despawn(display);

        app.add_systems(
            Update,
            (update_temperature, update_display.after(update_temperature)),
        );
        app.update();

        let total_bindings: usize = app
            .world_mut()
            .query::<&DataBindings>()
            .iter(app.world())
            .map(|bindings| bindings.0.len())
            .sum();

        assert_eq!(total_bindings, 2);
    }
}
