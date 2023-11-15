use std::time::Duration;

use bevy::{log, prelude::*};

use super::{
    collision::{CollisionMap, CollisionResult},
    handle_sokoban_events,
    history::HandleHistoryEvents,
    player::Player,
    Dir, GameState, Pos,
};

pub struct MomentumPlugin;

impl Plugin for MomentumPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MomentumTimer>()
            .register_type::<Momentum>()
            .init_resource::<MomentumTimer>()
            .add_systems(
                Update,
                (
                    (
                        handle_momentum.before(handle_sokoban_events),
                        (apply_momentum, reset_momentum_timer)
                            .chain()
                            .after(HandleHistoryEvents),
                    )
                        .run_if(can_apply_momentum()),
                    tick_momentum_timer.run_if(any_momentum_left()),
                )
                    .run_if(in_state(GameState::Play)),
            );
    }
}

#[derive(Default, Component, Copy, Clone, Deref, DerefMut, Reflect)]
pub struct Momentum(pub Option<Dir>);

fn handle_momentum(
    mut momentum_query: Query<(Entity, &mut Pos, &mut Momentum)>,
    collision: Res<CollisionMap>,
) {
    // Get all entities that are moving right now
    let has_momentum: Vec<(Entity, Pos, Dir)> = momentum_query
        .iter()
        .filter_map(|(entity, pos, momentum)| momentum.map(|direction| (entity, *pos, direction)))
        .collect();
    for (entity, pos, direction) in has_momentum.iter() {
        // Entities that get pushed including the pusher
        let push = collision.push_collision(IVec2::from(*pos), *direction);
        match push {
            CollisionResult::Push(push) => {
                // Transfer pushers momentum ala newtons cradle

                // Find entity without momentum
                let mut latest_without_momentum = None;
                for e in push.iter() {
                    if momentum_query
                        .get_component::<Momentum>(*e)
                        .expect("Dynamic objects have a momentum component")
                        .is_none()
                    {
                        latest_without_momentum.replace((*e, direction));
                    }
                }

                // Transfer momentum
                if let Some((transfer, momentum)) = latest_without_momentum {
                    let [(_, _, mut tm), (_, _, mut em)] =
                        momentum_query.get_many_mut([transfer, *entity]).expect(
                            "Both entities are guaranteed to have a 
                            position and momentum by above invariants",
                        );
                    tm.replace(*momentum);
                    em.take();
                }
            }
            CollisionResult::Wall => {
                // Stoppable force meets immovable object
                momentum_query
                    .get_component_mut::<Momentum>(*entity)
                    .expect("Dynamic objects have a momentum component")
                    .take();
            }
            CollisionResult::OutOfBounds => {
                log::warn!("Entity {:?} out of bounds", entity);
            }
        }
    }
}

pub fn apply_momentum(mut momentum_query: Query<(&mut Pos, &mut Momentum, Option<&Player>)>) {
    for (mut pos, mut momentum, player) in momentum_query.iter_mut() {
        if let Some(dir) = **momentum {
            pos.add_dir(dir);
            if player.is_some() {
                momentum.take();
            };
        }
    }
}

// Is there any object still moving
pub fn any_momentum_left() -> impl FnMut(Query<&Momentum>) -> bool + Clone {
    move |query: Query<&Momentum>| query.iter().any(|momentum| momentum.is_some())
}

#[derive(Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct MomentumTimer(pub Timer);

impl Default for MomentumTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(25), TimerMode::Once))
    }
}

fn tick_momentum_timer(mut movement_timer: ResMut<MomentumTimer>, time: Res<Time>) {
    movement_timer.tick(time.delta());
}

pub fn reset_momentum_timer(mut movement_timer: ResMut<MomentumTimer>) {
    movement_timer.reset();
}

pub fn can_apply_momentum() -> impl FnMut(Res<MomentumTimer>) -> bool + Clone {
    move |timer: Res<MomentumTimer>| timer.finished()
}
