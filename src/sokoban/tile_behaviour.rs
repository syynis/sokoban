use std::ops::AddAssign;

use bevy::{ecs::system::Command, prelude::*};
use bevy_ecs_tilemap::tiles::TileTextureIndex;

use super::{
    ball::Ball,
    entity::DespawnSokobanEntityCommand,
    level_select::CurrentLevel,
    momentum::{any_momentum_left, apply_momentum, transfer_momentum, Momentum},
    player::Player,
    AssetsCollection, GameState, Pos,
};

pub struct TileBehaviourPlugin;

impl Plugin for TileBehaviourPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                rubber,
                lamp_interaction.before(transfer_momentum),
                lamp_visual.after(lamp_interaction),
                void.after(transfer_momentum).before(apply_momentum),
                sand.after(apply_momentum),
                win.run_if(not(any_momentum_left()).and_then(goal.and_then(lamp()))),
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
#[derive(Component)]
pub struct Lamp(pub bool);

fn lamp_interaction(
    mut lamp_query: Query<(&Pos, &mut Lamp)>,
    momentum_query: Query<(&Pos, &Momentum), Without<Player>>,
) {
    for (pos, momentum) in momentum_query.iter() {
        if let Some(dir) = **momentum {
            let mut dest = *pos;
            dest.add_dir(dir);

            for (lamp_pos, mut lamp) in lamp_query.iter_mut() {
                if *lamp_pos == dest {
                    lamp.0 = !lamp.0;
                    break;
                }
            }
        }
    }
}

fn lamp_visual(mut lamp_query: Query<(&mut TileTextureIndex, &Lamp), Changed<Lamp>>) {
    for (mut id, lamp_state) in lamp_query.iter_mut() {
        if lamp_state.0 {
            id.0 = 6;
        } else {
            id.0 = 5;
        }
    }
}

fn win(mut current_level: ResMut<CurrentLevel>, mut next_state: ResMut<NextState<GameState>>) {
    current_level.add_assign(1);
    next_state.set(GameState::LevelTransition);
}

fn lamp() -> impl FnMut(Query<&Lamp>) -> bool + Clone {
    move |query: Query<&Lamp>| query.iter().all(|lamp| lamp.0)
}

fn goal(balls: Query<&Pos, With<Ball>>, goals: Query<&Pos, With<Goal>>) -> bool {
    goals
        .iter()
        .all(|goal| balls.iter().any(|ball| ball == goal))
}

fn sand(
    sand_query: Query<&Pos, With<Sand>>,
    mut momentum_query: Query<(&Pos, &mut Momentum), (Without<Player>, Changed<Pos>)>,
) {
    for (pos, mut momentum) in momentum_query.iter_mut() {
        if sand_query.iter().any(|sand_pos| sand_pos == pos) {
            momentum.take();
        }
    }
}

fn rubber(
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

fn void(
    mut cmds: Commands,
    void_query: Query<&Pos, With<Void>>,
    sokoban_query: Query<(Entity, &Pos), Without<Void>>,
) {
    for (entity, pos) in sokoban_query.iter() {
        if void_query.iter().any(|void_pos| void_pos == pos) {
            cmds.add(DespawnSokobanEntityCommand(entity));
        }
    }
}

pub struct SpawnGoal {
    pos: Pos,
    tilemap_entity: Entity,
}

impl SpawnGoal {
    pub fn new(pos: Pos, tilemap_entity: Entity) -> Self {
        Self {
            pos,
            tilemap_entity,
        }
    }
}

impl Command for SpawnGoal {
    fn apply(self, world: &mut World) {
        let texture = world.resource::<AssetsCollection>().goal.clone();

        world
            .entity_mut(self.tilemap_entity)
            .with_children(|child_builder| {
                child_builder.spawn((
                    Name::new("Goal"),
                    Goal,
                    self.pos,
                    SpriteBundle {
                        texture,
                        transform: Transform::from_translation(Vec3::Z),
                        ..default()
                    },
                ));
            });
    }
}
