use bevy::{ecs::system::Command, prelude::*};
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_pile::tilemap::tile_to_world_pos;
use leafwing_input_manager::prelude::*;

use super::{
    handle_sokoban_events,
    history::{History, HistoryEvent},
    momentum::{any_momentum_left, Momentum},
    Dir, GameState, Pos, Pusher, SokobanBlock, SokobanEvents,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerActions>::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                handle_player_actions
                    .before(handle_sokoban_events)
                    .run_if(not(any_momentum_left()))
                    .run_if(in_state(GameState::Play)),
            )
            .add_systems(
                PostUpdate,
                clear_player_momentum.run_if(in_state(GameState::Play)),
            );
    }
}

#[derive(Component)]
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
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let asset_server = world.resource::<AssetServer>();
        let player_handle: Handle<Image> = asset_server.load("player.png");
        world.spawn((
            Name::new("Player"),
            Player,
            self.pos,
            History::<Pos>::default(),
            SokobanBlock::Dynamic,
            Pusher,
            SpriteBundle {
                texture: player_handle,
                transform: Transform::from_translation(
                    tile_to_world_pos(&self.pos, &TilemapGridSize { x: 8., y: 8. }).extend(1.),
                ),
                ..default()
            },
            Momentum::default(),
        ));
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
    mut player_q: Query<Entity, With<Player>>,
    player_actions: Query<&ActionState<PlayerActions>>,
    mut history_events: EventWriter<HistoryEvent>,
    mut sokoban: SokobanEvents,
) {
    let Some(player) = player_q.get_single_mut().ok() else {
        return;
    };

    let Some(player_actions) = player_actions.get_single().ok() else {
        return;
    };

    player_actions
        .get_just_pressed()
        .iter()
        .for_each(|action| sokoban.move_entity(player, Dir::from(*action)));
    if player_actions.get_just_pressed().len() > 0 {
        history_events.send(HistoryEvent::Record)
    }
}

fn clear_player_momentum(mut player_q: Query<&mut Momentum, With<Player>>) {
    let Ok(mut momentum) = player_q.get_single_mut() else {
        return;
    };
    momentum.take();
}
