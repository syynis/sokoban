use std::marker::PhantomData;

use bevy::prelude::*;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentTime>()
            .add_event::<HistoryEvent>();
    }
}

#[derive(Default)]
pub struct HistoryComponentPlugin<C: Component + Clone> {
    phantom: PhantomData<C>,
}

impl<C: Component + Clone> Plugin for HistoryComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_history_commands::<C>.in_set(HandleHistoryEvents),
        );
    }
}

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct HandleHistoryEvents;

#[derive(Resource, Reflect, Default, Copy, Clone, Debug, Deref, DerefMut)]
#[reflect(Resource)]
pub struct CurrentTime(pub Timestamp);

#[derive(Debug, Clone, Default, Copy, Reflect)]
pub struct Timestamp(pub usize);

#[derive(Event)]
pub enum HistoryEvent {
    Record,
    Rewind,
    Reset,
}

fn handle_time(
    mut current_time: ResMut<CurrentTime>,
    mut history_events: EventReader<HistoryEvent>,
) {
    for ev in history_events.read() {
        match ev {
            HistoryEvent::Record => todo!(),
            HistoryEvent::Rewind => todo!(),
            HistoryEvent::Reset => todo!(),
        }
    }
}

#[derive(Component, Clone, Default, Deref, DerefMut, Reflect)]
pub struct History<C: Component + Clone>(Vec<(Timestamp, C)>);

fn handle_history_commands<C>(
    mut history_query: Query<(&mut History<C>, &mut C)>,
    mut history_events: EventReader<HistoryEvent>,
    current_time: Res<CurrentTime>,
) where
    C: Component + Clone,
{
    for ev in history_events.read() {
        match ev {
            HistoryEvent::Record => {
                for (mut history, component) in history_query.iter_mut() {
                    history.push((**current_time, component.clone()));
                }
            }
            HistoryEvent::Rewind => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some((_, prev_component)) = history.pop() {
                        *component = prev_component;
                    }
                }
            }
            HistoryEvent::Reset => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(first) = history.first() {
                        let (_, first_component) = first.clone();
                        history.push((**current_time, component.clone()));
                        *component = first_component;
                    }
                }
            }
        }
    }
}
