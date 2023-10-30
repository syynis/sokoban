use std::ops::Add;

use bevy::{ecs::system::SystemParam, prelude::*, reflect::TypePath, sprite::ColorMaterialPlugin};
use bevy_ecs_tilemap::{
    prelude::TilemapSize,
    tiles::{TilePos, TileStorage},
};
use bevy_pile::{grid::Grid, tilemap::tile_to_world_pos};
use leafwing_input_manager::prelude::*;

use self::{
    history::{HandleHistoryEvents, History, HistoryEvent, HistoryPlugin},
    player::PlayerPlugin,
};

pub mod cube;
pub mod history;
pub mod momentum;
pub mod player;

pub struct SokobanPlugin;

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            HistoryPlugin::<Pos>::default(),
            InputManagerPlugin::<SokobanActions>::default(),
        ))
        .register_type::<Pos>()
        .register_type::<History<Pos>>()
        .register_type::<SokobanBlock>()
        .register_type::<CollisionMap>()
        .add_event::<SokobanEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_collision_map,
                handle_sokoban_actions.before(HandleHistoryEvents),
                handle_sokoban_events.run_if(on_event::<SokobanEvent>()),
            ),
        )
        .add_systems(PostUpdate, (copy_pos_to_transform, sync_collision_map));
    }
}

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum SokobanActions {
    Rewind,
}

fn sokoban_actions() -> InputMap<SokobanActions> {
    use SokobanActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::U, Rewind);

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
        transform.translation = tile_to_world_pos(pos).extend(transform.translation.z);
    }
}

#[derive(Debug, Copy, Clone)]
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
    names: Query<&Name>,
    mut sokoban_entities: Query<(Entity, &mut Pos, &SokobanBlock)>,
    mut sokoban_events: EventReader<SokobanEvent>,
    tilemap: Query<&TilemapSize>,
    collision: Res<CollisionMap>,
) {
    let Some(tilemap) = tilemap.get_single().ok() else {
        return;
    };
    for ev in sokoban_events.iter() {
        let SokobanEvent::Move { entity, direction } = ev;

        if let Some((_, pos, _)) = sokoban_entities.get(*entity).ok() {
            let push = collision.push_collision(IVec2::from(*pos), *direction);
            let push_names: Vec<String> = push
                .iter()
                .map(|e| names.get(*e).unwrap().as_str().to_owned())
                .collect();
            bevy::log::info!("push {:?}", push_names);

            for e in push.iter() {
                sokoban_entities
                    .get_component_mut::<Pos>(*e)
                    .expect("Should be valid")
                    .add_dir(*direction);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
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

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CollisionMap {
    map: Grid<Option<(Entity, SokobanBlock)>>,
}

impl Default for CollisionMap {
    fn default() -> Self {
        Self {
            map: Grid::new(IVec2::new(0, 0), None),
        }
    }
}

fn init_collision_map(
    mut cmds: Commands,
    tilemap: Query<&TilemapSize, Added<TileStorage>>,
    sokoban_entities: Query<(Entity, &Pos, &SokobanBlock)>,
) {
    let Some(size) = tilemap.get_single().ok() else {
        return;
    };
    bevy::log::info!("Initialized collision map");
    let mut map = Grid::new(IVec2::new(size.x as i32, size.y as i32), None);
    for (entity, pos, block) in sokoban_entities.iter() {
        let pos = IVec2::from(pos);
        map.set(pos, Some((entity, *block)));
    }
    cmds.insert_resource(CollisionMap { map });
}

// TODO dont rebuild but instead only change moved entities
fn sync_collision_map(
    mut collision: ResMut<CollisionMap>,
    sokoban_entities: Query<(Entity, &Pos, &SokobanBlock)>,
) {
    collision.map.iter_mut().for_each(|(_, elem)| {
        elem.take();
    });
    for (entity, pos, block) in sokoban_entities.iter() {
        collision.map.set(IVec2::from(pos), Some((entity, *block)));
    }
}

impl CollisionMap {
    fn push_collision(&self, pusher_pos: IVec2, direction: Dir) -> Vec<Entity> {
        let Some(Some((pusher, _))) = self.map.get(pusher_pos) else {
            return Vec::new();
        };

        let move_in_dir = |pos| -> IVec2 { pos + IVec2::from(direction) };
        let mut to_push = Vec::new();
        to_push.push(*pusher);
        let mut current_pos = pusher_pos;
        let mut dest = move_in_dir(current_pos);
        while let Some(Some((entity, SokobanBlock::Dynamic))) = self.map.get(dest) {
            to_push.push(*entity);
            current_pos = dest;
            dest = move_in_dir(current_pos);
        }
        to_push
    }
}
