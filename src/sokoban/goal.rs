use super::{ball::Ball, momentum::any_momentum_left, GameState, Pos};

use bevy::prelude::*;

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            check_goal
                .run_if(in_state(GameState::Play))
                .run_if(not(any_momentum_left())),
        );
    }
}

#[derive(Component)]
pub struct Goal;

fn check_goal(
    balls: Query<&Pos, With<Ball>>,
    goals: Query<&Pos, With<Goal>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let satisfied = goals
        .iter()
        .all(|goal| balls.iter().any(|ball| ball == goal));
    if satisfied {
        next_state.set(GameState::LevelSelect);
    }
}
