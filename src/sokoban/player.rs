use bevy::{ecs::system::Command, log, prelude::*};
use leafwing_input_manager::prelude::*;

use super::{
    collision::{CollisionMap, CollisionResult},
    history::{HandleHistoryEvents, History, HistoryEvent},
    momentum::{any_momentum_left, Momentum},
    AssetsCollection, Dir, GameState, Pos, Pusher, SokobanBlock,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerActions>::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                handle_player_actions
                    .before(HandleHistoryEvents)
                    .run_if(not(any_momentum_left()))
                    .run_if(in_state(GameState::Play)),
            );
    }
}

#[derive(Component, Clone)]
pub struct Player;

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum PlayerActions {
    Up,
    Right,
    Down,
    Left,
}

impl From<PlayerActions> for Dir {
    fn from(value: PlayerActions) -> Dir {
        match value {
            PlayerActions::Up => Dir::Up,
            PlayerActions::Left => Dir::Left,
            PlayerActions::Down => Dir::Down,
            PlayerActions::Right => Dir::Right,
        }
    }
}

pub struct SpawnPlayer {
    pub pos: Pos,
    pub tilemap_entity: Entity,
}

impl SpawnPlayer {
    pub fn new(pos: Pos, tilemap_entity: Entity) -> Self {
        Self {
            pos,
            tilemap_entity,
        }
    }
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let texture = world.resource::<AssetsCollection>().player.clone();

        world
            .entity_mut(self.tilemap_entity)
            .with_children(|child_builder| {
                child_builder.spawn((
                    Name::new("Player"),
                    Player,
                    self.pos,
                    History::<Pos>::default(),
                    SokobanBlock::Dynamic,
                    Pusher,
                    SpriteBundle {
                        texture,
                        transform: Transform::from_translation(2. * Vec3::Z),
                        ..default()
                    },
                    Momentum::default(),
                ));
            });
    }
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        (InputManagerBundle::<PlayerActions> {
            input_map: player_actions(),
            ..default()
        },),
        Name::new("PlayerActions"),
    ));
}

fn player_actions() -> InputMap<PlayerActions> {
    use PlayerActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::W, Up);
    input_map.insert(KeyCode::D, Right);
    input_map.insert(KeyCode::S, Down);
    input_map.insert(KeyCode::A, Left);

    input_map
}

pub fn handle_player_actions(
    player_q: Query<&Pos, With<Player>>,
    mut sokoban_entities: Query<&mut Momentum>,
    player_actions: Query<&ActionState<PlayerActions>>,
    mut history_events: EventWriter<HistoryEvent>,
    collision: Res<CollisionMap>,
) {
    let Ok(player_pos) = player_q.get_single() else {
        return;
    };

    let player_actions = player_actions
        .get_single()
        .expect("Player input map should exist");

    for direction in player_actions
        .get_pressed()
        .iter()
        .map(|action| Dir::from(*action))
    {
        match collision.push_collision(IVec2::from(player_pos), direction) {
            CollisionResult::Push(push) => {
                for e in push.iter() {
                    sokoban_entities
                        .get_component_mut::<Momentum>(*e)
                        .expect("Dynamic objects have a momentum component")
                        .replace(direction);
                }
                history_events.send(HistoryEvent::Record);
                break;
            }
            CollisionResult::Wall => {
                log::debug!("Can't move");
            }
            CollisionResult::OutOfBounds => {
                log::warn!("Player out of bounds");
            }
        };
    }
}
