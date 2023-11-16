use std::ops::AddAssign;

use bevy::{ecs::system::Command, prelude::*};

use super::{
    ball::Ball,
    level::AssetCollection,
    level_select::CurrentLevel,
    momentum::{any_momentum_left, apply_momentum, can_apply_momentum, Momentum},
    player::Player,
    GameState, PixelOffset, Pos,
};

pub struct TileBehaviourPlugin;

impl Plugin for TileBehaviourPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    handle_rubber,
                    handle_void.before(apply_momentum),
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

pub struct SpawnGoal {
    pub pos: Pos,
    pub parent: Entity,
}

impl SpawnGoal {
    pub fn new(pos: Pos, parent: Entity) -> Self {
        Self { pos, parent }
    }
}

impl Command for SpawnGoal {
    fn apply(self, world: &mut World) {
        let assets = world.resource::<AssetCollection>();
        let goal_handle: Handle<Image> = assets.goal.clone();

        world
            .entity_mut(self.parent)
            .with_children(|child_builder| {
                child_builder.spawn((
                    Name::new("Goal"),
                    Goal,
                    self.pos,
                    SpriteBundle {
                        texture: goal_handle,
                        ..default()
                    },
                    PixelOffset(UVec2::Y * 2),
                ));
            });
    }
}
