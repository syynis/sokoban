use bevy::{ecs::system::SystemParam, log, prelude::*};
use bevy_ecs_tilemap::{prelude::TilemapGridSize, tiles::TilePos};
use bevy_pile::tilemap::tile_to_world_pos;
use leafwing_input_manager::prelude::*;

use crate::sokoban::momentum::Momentum;

use self::{
    collision::{CollisionMap, CollisionPlugin, CollisionResult},
    goal::GoalPlugin,
    history::{HandleHistoryEvents, History, HistoryEvent, HistoryPlugin},
    momentum::MomentumPlugin,
    player::PlayerPlugin,
    sand::SandPlugin,
    void::VoidPlugin,
};

pub mod ball;
pub mod collision;
pub mod goal;
pub mod history;
pub mod momentum;
pub mod player;
pub mod sand;
pub mod void;

pub struct SokobanPlugin;

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            HistoryPlugin::<Pos>::default(),
            InputManagerPlugin::<SokobanActions>::default(),
            MomentumPlugin,
            GoalPlugin,
            CollisionPlugin,
            SandPlugin,
            VoidPlugin,
        ))
        .add_state::<GameState>()
        .register_type::<Pos>()
        .register_type::<Dir>()
        .register_type::<History<Pos>>()
        .register_type::<SokobanBlock>()
        .add_event::<SokobanEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // General
                (log_state_change),
                // Play
                (
                    handle_sokoban_actions.before(HandleHistoryEvents),
                    handle_sokoban_events.run_if(on_event::<SokobanEvent>()),
                )
                    .run_if(in_state(GameState::Play)),
                // Level Select
                (play).run_if(in_state(GameState::LevelSelect)),
            ),
        )
        .add_systems(PostUpdate, copy_pos_to_transform);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    LevelSelect,
    Play,
}

fn play(
    mut next_state: ResMut<NextState<GameState>>,
    actions: Query<&ActionState<SokobanActions>>,
) {
    let Ok(actions) = actions.get_single() else {
        return;
    };

    if actions.just_pressed(SokobanActions::Play) {
        next_state.set(GameState::Play)
    }
}

fn log_state_change(state: Res<State<GameState>>) {
    if state.is_changed() {
        log::info!("{:?}", state);
    }
}

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum SokobanActions {
    Play,
    Rewind,
}

fn sokoban_actions() -> InputMap<SokobanActions> {
    use SokobanActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::U, Rewind);
    input_map.insert(KeyCode::P, Play);

    input_map
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        (InputManagerBundle::<SokobanActions> {
            input_map: sokoban_actions(),
            ..default()
        },),
        Name::new("SokobanActions"),
    ));
}

fn handle_sokoban_actions(
    actions: Query<&ActionState<SokobanActions>>,
    mut history_events: EventWriter<HistoryEvent>,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };
    if actions.just_pressed(SokobanActions::Rewind) {
        history_events.send(HistoryEvent::Rewind)
    }
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut, Reflect)]
pub struct Pos(pub TilePos);

impl Pos {
    pub fn new(x: u32, y: u32) -> Self {
        Self(TilePos { x, y })
    }

    pub fn add_dir(&mut self, dir: Dir) {
        let dir = IVec2::from(dir);
        self.x = self.x.saturating_add_signed(dir.x);
        self.y = self.y.saturating_add_signed(dir.y);
    }
}

impl From<Pos> for IVec2 {
    fn from(value: Pos) -> Self {
        UVec2::from(*value).as_ivec2()
    }
}

impl From<&Pos> for IVec2 {
    fn from(value: &Pos) -> Self {
        UVec2::from(**value).as_ivec2()
    }
}

pub fn copy_pos_to_transform(mut query: Query<(&Pos, &mut Transform)>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = tile_to_world_pos(pos, &TilemapGridSize { x: 8., y: 8. })
            .extend(transform.translation.z);
    }
}

#[derive(Debug, Copy, Clone, Reflect)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl From<Dir> for IVec2 {
    fn from(direction: Dir) -> IVec2 {
        match direction {
            Dir::Up => IVec2::Y,
            Dir::Left => IVec2::new(-1, 0),
            Dir::Down => IVec2::new(0, -1),
            Dir::Right => IVec2::X,
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum SokobanEvent {
    Move { entity: Entity, direction: Dir },
    Momentum { entity: Entity, direction: Dir },
}

#[derive(SystemParam)]
pub struct SokobanEvents<'w> {
    writer: EventWriter<'w, SokobanEvent>,
}

impl<'w> SokobanEvents<'w> {
    pub fn move_entity(&mut self, entity: Entity, direction: Dir) {
        self.writer.send(SokobanEvent::Move { entity, direction });
    }
}

fn handle_sokoban_events(
    mut sokoban_entities: Query<(&mut Pos, &mut Momentum)>,
    mut sokoban_events: EventReader<SokobanEvent>,
    collision: Res<CollisionMap>,
) {
    for ev in sokoban_events.iter() {
        match ev {
            SokobanEvent::Move { entity, direction } => {
                if let Some((pos, _)) = sokoban_entities.get(*entity).ok() {
                    let push = collision.push_collision(IVec2::from(*pos), *direction);
                    let CollisionResult::Push(push) = push else {
                        continue;
                    };

                    for (idx, e) in push.iter().enumerate() {
                        if idx != 0 {
                            sokoban_entities
                                .get_component_mut::<Momentum>(*e)
                                .expect("Should be valid")
                                .0
                                .replace(*direction);
                        } else {
                            sokoban_entities
                                .get_component_mut::<Pos>(*e)
                                .expect("Should be valid")
                                .add_dir(*direction);
                        }
                    }
                }
            }
            SokobanEvent::Momentum { entity, direction } => {
                if let Some((pos, momentum)) = sokoban_entities.get(*entity).ok() {
                    let Some(dir) = **momentum else { continue };
                    let push = collision.push_collision(IVec2::from(*pos), *direction);
                    match push {
                        CollisionResult::Push(push) => {
                            let mut latest_without_momentum = None;
                            for e in push.iter() {
                                if sokoban_entities
                                    .get_component::<Momentum>(*e)
                                    .expect("Should be valid")
                                    .is_none()
                                {
                                    latest_without_momentum.replace((*e, dir));
                                }
                            }

                            if let Some((transfer, momentum)) = latest_without_momentum {
                                let [(_, mut tm), (_, mut em)] = sokoban_entities
                                    .get_many_mut([transfer, *entity])
                                    .expect("Should be ok");
                                tm.replace(momentum);
                                em.take();
                            }
                        }
                        CollisionResult::Wall => {
                            sokoban_entities
                                .get_component_mut::<Momentum>(*entity)
                                .expect("Should be ok")
                                .take();
                        }
                        CollisionResult::OutOfBounds => {
                            log::warn!("Entity {:?} out of bounds", *entity);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Component, Reflect, PartialEq)]
pub enum SokobanBlock {
    Static,
    Dynamic,
}

#[derive(Clone, Default, Debug, Component)]
pub struct Pusher;

#[derive(Debug, Clone)]
pub struct PushEvent {
    pub pusher: Entity,
    pub direction: Dir,
    pub pushed: Vec<Entity>,
}
