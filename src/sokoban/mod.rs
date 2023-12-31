use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_ecs_tilemap::{prelude::TilemapGridSize, tiles::TilePos, TilemapPlugin};
use bevy_nine_slice_ui::NineSlicePlugin;
use bevy_pile::tilemap::tile_to_world_pos;
use leafwing_input_manager::prelude::*;

use crate::sokoban::momentum::Momentum;

use self::{
    audio::{AudioCollection, GameAudioPlugin},
    cleanup::cleanup_on_state_change,
    collision::CollisionPlugin,
    entity::CommandHistoryPlugin,
    history::{HandleHistoryEvents, History, HistoryComponentPlugin, HistoryEvent, HistoryPlugin},
    level::{LevelCollection, LevelPlugin},
    level_select::LevelSelectPlugin,
    level_transition::LevelTransitionPlugin,
    main_menu::MainMenuPlugin,
    momentum::MomentumPlugin,
    pause_menu::PauseMenuPlugin,
    player::PlayerPlugin,
    tile_behaviour::TileBehaviourPlugin,
};

pub mod audio;
pub mod ball;
pub mod cleanup;
pub mod collision;
pub mod entity;
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
pub mod ui;
pub mod util;

pub struct SokobanPlugin;

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            HistoryPlugin,
            HistoryComponentPlugin::<Pos>::default(),
            InputManagerPlugin::<SokobanActions>::default(),
            MomentumPlugin,
            CollisionPlugin,
            MainMenuPlugin,
            LevelSelectPlugin,
            LevelPlugin,
            PauseMenuPlugin,
            LevelTransitionPlugin,
            TileBehaviourPlugin,
            CommandHistoryPlugin,
            TilemapPlugin,
            (GameAudioPlugin, NineSlicePlugin::default()),
        ))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::MainMenu),
        )
        .add_collection_to_loading_state::<_, AssetsCollection>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, LevelCollection>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, AudioCollection>(GameState::AssetLoading)
        .register_type::<Pos>()
        .register_type::<Dir>()
        .register_type::<History<Pos>>()
        .register_type::<SokobanBlock>()
        .register_type::<AssetsCollection>()
        .add_event::<SokobanEvent>()
        .add_systems(Startup, setup)
        .insert_resource(Time::<Fixed>::from_hz(8.))
        .add_systems(
            Update,
            (
                // Play
                undo.after(HandleHistoryEvents)
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
    AssetLoading,
    MainMenu,
    LevelSelect,
    LevelTransition,
    Play,
    Pause,
}

#[derive(Resource, Reflect, Default, AssetCollection, Debug)]
#[reflect(Resource)]
pub struct AssetsCollection {
    #[asset(path = "tiles.png")]
    pub tiles: Handle<Image>,
    #[asset(path = "wall.png")]
    pub wall_tiles: Handle<Image>,
    #[asset(path = "player.png")]
    pub player: Handle<Image>,
    #[asset(path = "ball.png")]
    pub ball: Handle<Image>,
    #[asset(path = "goal.png")]
    pub goal: Handle<Image>,
    #[asset(path = "button.png")]
    pub button: Handle<Image>,
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

#[derive(Debug, Clone, Event)]
pub enum SokobanEvent {
    PlayerMoved,
    PlayerPush,
    BallPush,
    BallHitWall,
    EntityInVoid,
}

fn undo(
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

pub fn copy_pos_to_transform(mut query: Query<(&Pos, &mut Transform), Changed<Pos>>) {
    for (pos, mut transform) in query.iter_mut() {
        let new = tile_to_world_pos(pos, &TilemapGridSize { x: 8., y: 8. })
            .extend(transform.translation.z);

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

#[derive(Debug, Copy, Clone, Component, Reflect, PartialEq)]
pub enum SokobanBlock {
    Static,
    Dynamic,
}

#[derive(Bundle, Clone)]
pub struct DynamicBundle {
    momentum: Momentum,
    block: SokobanBlock,
}

impl Default for DynamicBundle {
    fn default() -> Self {
        Self {
            momentum: Momentum::default(),
            block: SokobanBlock::Dynamic,
        }
    }
}
