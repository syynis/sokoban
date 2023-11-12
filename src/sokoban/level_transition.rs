use bevy::prelude::*;

use super::GameState;

pub struct LevelTransitionPlugin;

impl Plugin for LevelTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            transition.run_if(in_state(GameState::LevelTransition)),
        );
    }
}

fn transition(keys: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::T) {
        next_state.set(GameState::Play)
    }
}
