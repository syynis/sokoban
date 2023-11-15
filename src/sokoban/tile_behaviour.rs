use std::ops::AddAssign;

use bevy::prelude::*;

use super::{
    ball::Ball,
    level::CurrentLevel,
    momentum::{any_momentum_left, apply_momentum, can_apply_momentum, handle_momentum, Momentum},
    player::Player,
    GameState, Pos,
};

pub struct TileBehaviourPlugin;

impl Plugin for TileBehaviourPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    handle_rubber,
                    // HACK Run handle_momentum again for proper void mechanics
                    (handle_void, handle_momentum)
                        .chain()
                        .before(apply_momentum),
                    handle_sand.after(apply_momentum),
                )
                    .run_if(can_apply_momentum()),
                handle_goal.run_if(not(any_momentum_left())),
            )
                .run_if(in_state(GameState::Play)),
        );
    }
}

#[derive(Component)]
pub struct Sand;
#[derive(Component)]
pub struct Goal;
#[derive(Component)]
pub struct Rubber;
#[derive(Component)]
pub struct Void;

fn handle_sand(
    sand_query: Query<&Pos, With<Sand>>,
    mut momentum_query: Query<(&Pos, &mut Momentum), (Without<Player>, Changed<Pos>)>,
) {
    for (pos, mut momentum) in momentum_query.iter_mut() {
        if sand_query.iter().any(|sand_pos| sand_pos == pos) {
            momentum.take();
        }
    }
}

fn handle_rubber(
    rubber_query: Query<&Pos, With<Rubber>>,
    mut momentum_query: Query<(&Pos, &mut Momentum)>,
) {
    for (pos, mut momentum) in momentum_query.iter_mut() {
        if let Some(dir) = **momentum {
            let mut dest = *pos;
            dest.add_dir(dir);

            if rubber_query.iter().any(|rubber_pos| *rubber_pos == dest) {
                momentum.replace(dir.opposite());
            }
        }
    }
}

fn handle_void(
    mut cmds: Commands,
    void_query: Query<&Pos, With<Void>>,
    sokoban_query: Query<(Entity, &Pos), (Without<Void>, Changed<Pos>)>,
) {
    for (entity, pos) in sokoban_query.iter() {
        if void_query.iter().any(|void_pos| void_pos == pos) {
            cmds.entity(entity).despawn_recursive();
        }
    }
}

fn handle_goal(
    balls: Query<&Pos, With<Ball>>,
    goals: Query<&Pos, With<Goal>>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let satisfied = goals
        .iter()
        .all(|goal| balls.iter().any(|ball| ball == goal));
    if satisfied {
        current_level.add_assign(1);
        next_state.set(GameState::LevelTransition);
    }
}
