use std::time::Duration;

use bevy::{log, prelude::*, time::common_conditions::on_timer};

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
        app.register_type::<Momentum>().add_systems(
            Update,
            (
                handle_momentum
                    .run_if(resource_exists::<CollisionMap>())
                    .before(handle_sokoban_events)
                    .before(HandleHistoryEvents),
                (apply_momentum, handle_player_momentum)
                    .chain()
                    .after(HandleHistoryEvents),
            )
                .run_if(on_timer(Duration::from_millis(50)))
                .run_if(in_state(GameState::Play)),
        );
    }
}

#[derive(Default, Component, Copy, Clone, Deref, DerefMut, Reflect)]
pub struct Momentum(pub Option<Dir>);

pub fn handle_momentum(
    mut momentum_query: Query<(Entity, &mut Pos, &mut Momentum)>,
    collision: Res<CollisionMap>,
) {
    let has_momentum: Vec<(Entity, Pos, Dir)> = momentum_query
        .iter()
        .filter_map(|(entity, pos, momentum)| momentum.map(|direction| (entity, *pos, direction)))
        .collect();
    for (entity, pos, direction) in has_momentum.iter() {
        let push = collision.push_collision(IVec2::from(*pos), *direction);
        match push {
            CollisionResult::Push(push) => {
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

pub fn apply_momentum(mut momentum_query: Query<(&mut Pos, &Momentum), Without<Player>>) {
    for (mut pos, momentum) in momentum_query.iter_mut() {
        if let Some(dir) = **momentum {
            pos.add_dir(dir);
        }
    }
}

fn handle_player_momentum(mut player_q: Query<(&mut Pos, &mut Momentum), With<Player>>) {
    if let Ok((mut pos, mut momentum)) = player_q.get_single_mut() {
        if let Some(dir) = **momentum {
            pos.add_dir(dir);
            momentum.take();
        }
    }
}

// Is there any object still moving
pub fn any_momentum_left() -> impl FnMut(Query<&Momentum>) -> bool + Clone {
    move |query: Query<&Momentum>| query.iter().any(|momentum| momentum.is_some())
}
