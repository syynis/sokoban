use bevy::{prelude::*, ecs::system::Command};
use bevy_pile::tilemap::tile_to_world_pos;

use super::{player::Player, GameState, Pos};

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_goal.run_if(in_state(GameState::Play)));
    }
}

#[derive(Component)]
pub struct Goal;

pub struct SpawnGoal(pub Pos);

impl Command for SpawnGoal {
    fn apply(self, world: &mut World) {

        world.spawn((
            Name::new("Goal"),
            Goal,
            self.0,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: Some(Vec2 { x: 16., y: 16. }),
                    ..default()
                },
                transform: Transform::from_translation(tile_to_world_pos(&self.0).extend(1.)),
                ..default()
            },
        ));
    }
}

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
