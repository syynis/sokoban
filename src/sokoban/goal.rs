use super::{player::Player, GameState, Pos};

use bevy::prelude::*;

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_goal.run_if(in_state(GameState::Play)));
    }
}

#[derive(Component)]
pub struct Goal;

fn check_goal(
    player: Query<&Pos, With<Player>>,
    goal: Query<&Pos, With<Goal>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(goal_pos) = goal.get_single() else {
        bevy::log::warn!("Either no or more than one goal exists");
        return;
    };

    let Ok(player_pos) = player.get_single() else {
        bevy::log::warn!("Either no or more than one player exists");
        return;
    };

    if goal_pos == player_pos {
        next_state.set(GameState::LevelSelect);
    }
}
