use bevy::{ecs::system::SystemParam, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::sokoban::momentum::Momentum;

use self::{
    cleanup::cleanup_on_state_change,
    collision::{CollisionMap, CollisionPlugin, CollisionResult},
    history::{HandleHistoryEvents, History, HistoryEvent, HistoryPlugin},
    level::LevelPlugin,
    level_select::LevelSelectPlugin,
    level_transition::LevelTransitionPlugin,
    main_menu::MainMenuPlugin,
    momentum::MomentumPlugin,
    pause_menu::PauseMenuPlugin,
    player::PlayerPlugin,
    tile_behaviour::TileBehaviourPlugin,
};

pub mod ball;
pub mod cleanup;
pub mod collision;
pub mod event_scheduler;
pub mod history;
pub mod level;
pub mod level_select;
pub mod level_transition;
pub mod main_menu;
pub mod momentum;
pub mod pause_menu;
pub mod player;
pub mod tile_behaviour;

pub struct SokobanPlugin;

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            HistoryPlugin::<Pos>::default(),
            InputManagerPlugin::<SokobanActions>::default(),
            MomentumPlugin,
            CollisionPlugin,
            MainMenuPlugin,
            LevelSelectPlugin,
            LevelPlugin,
            PauseMenuPlugin,
            LevelTransitionPlugin,
            TileBehaviourPlugin,
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
                // Play
                (
                    handle_history.after(HandleHistoryEvents),
                    handle_sokoban_events
                        .run_if(on_event::<SokobanEvent>())
                        .before(HandleHistoryEvents),
                )
                    .run_if(in_state(GameState::Play)),
                escape,
            ),
        )
        .add_systems(
            StateTransition,
            cleanup_on_state_change::<GameState>.before(apply_state_transition::<GameState>),
        )
        .add_systems(PostUpdate, copy_pos_to_transform);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    LevelSelect,
    LevelTransition,
    Play,
    Pause,
}

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum SokobanActions {
    Undo,
    Escape,
    Reset,
    UiNavUp,
    UiNavRight,
    UiNavDown,
    UiNavLeft,
    UiNavSelect,
}

fn sokoban_actions() -> InputMap<SokobanActions> {
    use SokobanActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::E, Undo);
    input_map.insert_many_to_one(vec![KeyCode::Escape, KeyCode::Q], Escape);
    input_map.insert(KeyCode::R, Reset);

    input_map.insert(KeyCode::W, UiNavUp);
    input_map.insert(KeyCode::D, UiNavRight);
    input_map.insert(KeyCode::S, UiNavDown);
    input_map.insert(KeyCode::A, UiNavLeft);
    input_map.insert(KeyCode::F, UiNavSelect);

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

fn handle_history(
    actions: Query<&ActionState<SokobanActions>>,
    mut history_events: EventWriter<HistoryEvent>,
    mut momentum_query: Query<&mut Momentum>,
) {
    let Ok(actions) = actions.get_single() else {
        return;
    };
    if actions.just_pressed(SokobanActions::Undo) {
        history_events.send(HistoryEvent::Rewind);
        for mut momentum in momentum_query.iter_mut() {
            momentum.take();
        }
    } else if actions.just_pressed(SokobanActions::Reset) {
        history_events.send(HistoryEvent::Reset);
        for mut momentum in momentum_query.iter_mut() {
            momentum.take();
        }
    }
}

fn escape(
    actions: Query<&ActionState<SokobanActions>>,
    mut state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };
    if actions.just_pressed(SokobanActions::Escape) {
        match **current_state {
            GameState::LevelSelect => state.set(GameState::MainMenu),
            GameState::Play => state.set(GameState::Pause),
            GameState::Pause => state.set(GameState::Play),
            _ => {}
        }
    }
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut, Reflect)]
pub struct Pos(pub UVec2);

impl Pos {
    pub fn new(x: u32, y: u32) -> Self {
        Self(UVec2::new(x, y))
    }

    pub fn add_dir(&mut self, dir: Dir) {
        let dir = IVec2::from(dir);
        self.x = self.x.saturating_add_signed(dir.x);
        self.y = self.y.saturating_add_signed(dir.y);
    }

    pub fn to_world_pos(&self) -> Vec2 {
        let scaled = self.0 * 8;
        scaled.as_vec2()
    }
}

impl From<Pos> for IVec2 {
    fn from(value: Pos) -> Self {
        value.as_ivec2()
    }
}

impl From<&Pos> for IVec2 {
    fn from(value: &Pos) -> Self {
        value.as_ivec2()
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct PixelOffset(pub UVec2);

#[derive(Component, Deref, DerefMut)]
pub struct ZOffset(pub f32);

pub fn copy_pos_to_transform(
    mut query: Query<(&Pos, &mut Transform, Option<&PixelOffset>, Option<&ZOffset>), Changed<Pos>>,
) {
    for (pos, mut transform, offset, z) in query.iter_mut() {
        let offset = offset.map_or(UVec2::ZERO, |offset| **offset).as_vec2();
        let z = z.map_or(0., |z| **z);
        let new = pos.to_world_pos() + offset;
        let new = new.extend(-(pos.y as f32 - z));
        transform.translation = new;
    }
}

#[derive(Debug, Copy, Clone, Reflect)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    pub fn opposite(&self) -> Dir {
        use Dir::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

impl From<Dir> for IVec2 {
    fn from(direction: Dir) -> IVec2 {
        match direction {
            Dir::Up => IVec2::Y,
            Dir::Left => IVec2::NEG_X,
            Dir::Down => IVec2::NEG_Y,
            Dir::Right => IVec2::X,
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum SokobanEvent {
    Push { pusher: Entity, direction: Dir },
}

#[derive(SystemParam)]
pub struct SokobanEvents<'w> {
    writer: EventWriter<'w, SokobanEvent>,
}

impl<'w> SokobanEvents<'w> {
    pub fn move_entity(&mut self, entity: Entity, direction: Dir) {
        self.writer.send(SokobanEvent::Push {
            pusher: entity,
            direction,
        });
    }
}

fn handle_sokoban_events(
    mut sokoban_entities: Query<(&mut Pos, &mut Momentum)>,
    mut sokoban_events: EventReader<SokobanEvent>,
    collision: Res<CollisionMap>,
) {
    for ev in sokoban_events.read() {
        let SokobanEvent::Push {
            pusher: entity,
            direction,
        } = ev;
        if let Ok((pos, _)) = sokoban_entities.get(*entity) {
            let push = collision.push_collision(IVec2::from(*pos), *direction);
            if let CollisionResult::Push(push) = push {
                for e in push.iter() {
                    sokoban_entities
                        .get_component_mut::<Momentum>(*e)
                        .expect("Dynamic objects have a momentum component")
                        .replace(*direction);
                }
            };
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
