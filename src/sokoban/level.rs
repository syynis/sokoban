use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    reflect::{TypePath, TypeUuid},
};
use bevy_asset_loader::prelude::AssetCollection;
use bevy_ecs_tilemap::prelude::*;
use serde::Deserialize;
use thiserror::Error;

use super::{
    ball::SpawnBall,
    cleanup::DependOnState,
    collision::init_collision_map,
    level_select::CurrentLevel,
    player::SpawnPlayer,
    tile_behaviour::{Lamp, Rubber, Sand, SpawnGoal, Void},
    AssetsCollection, GameState, Pos, SokobanBlock,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(LevelLoader)
            .init_asset::<Levels>()
            .register_type::<Level>()
            .register_type::<LevelCollection>()
            .add_systems(
                OnTransition {
                    from: GameState::LevelTransition,
                    to: GameState::Play,
                },
                (spawn_level, apply_deferred, center_camera_on_level)
                    .chain()
                    .before(init_collision_map),
            )
            .add_systems(
                Update,
                reload_on_change
                    .run_if(in_state(GameState::Play))
                    .run_if(on_event::<AssetEvent<Levels>>()),
            );
    }
}

#[derive(Resource, Reflect, Default, Debug, AssetCollection)]
#[reflect(Resource)]
pub struct LevelCollection {
    #[asset(path = "test.levels")]
    pub levels: Handle<Levels>,
}

fn center_camera_on_level(
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    tilemap_q: Query<&TilemapSize, (With<TileStorage>, Without<Camera>)>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_q.get_single_mut() else {
        return;
    };

    let Ok(size) = tilemap_q.get_single() else {
        return;
    };
    let center = Vec2::from(*size) * 8. / 2. - 4.;
    camera_transform.translation = center.extend(camera_transform.translation.z);
    projection.scale = 0.15;
}

#[derive(Component)]
pub struct LevelRoot;

fn spawn_level(
    mut cmds: Commands,
    current_level: Res<CurrentLevel>,
    levels_assets: Res<Assets<Levels>>,
    asset_collection: Res<AssetsCollection>,
    level_collection: Res<LevelCollection>,
) {
    let levels_handle = &level_collection.levels;
    let tiles_handle = &asset_collection.tiles;

    let levels = levels_assets
        .get(levels_handle)
        .expect("Level handle should be loaded");
    let level = levels
        .get(**current_level)
        .expect("Current level should only ever be set to a valid level");

    let size = TilemapSize::from(level.size);
    let mut storage = TileStorage::empty(size);
    let level_root = cmds
        .spawn((
            SpatialBundle::default(),
            DependOnState(vec![GameState::Play, GameState::Pause]),
            Name::new("Level Root"),
            LevelRoot,
        ))
        .id();
    let tilemap_entity = cmds.spawn_empty().id();
    let tile_size = TilemapTileSize::from(Vec2::splat(8.));
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    for (idx, tile) in level.tiles.iter().enumerate() {
        let position = TilePos {
            x: idx as u32 % level.size.x,
            y: level.size.y - (idx as u32 / level.size.x) - 1,
        };

        let tile_entity = cmds
            .spawn((
                Name::new("Tile"),
                TileBundle {
                    position,
                    texture_index: TileTextureIndex::from(*tile),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                },
            ))
            .id();

        let pos = Pos(position);
        let mut tile_cmds = cmds.entity(tile_entity);
        if !matches!(tile, TileKind::Floor) {
            tile_cmds.insert(pos);
        }
        match tile {
            TileKind::Sand => {
                tile_cmds.insert((Name::new("Sand"), Sand));
            }
            TileKind::Rubber => {
                tile_cmds.insert((Name::new("Rubber"), SokobanBlock::Static, Rubber));
            }
            TileKind::Wall => {
                tile_cmds.insert(SokobanBlock::Static);
            }
            TileKind::Void => {
                tile_cmds.insert((Name::new("Void"), Void));
            }
            TileKind::Goal => {
                cmds.add(SpawnGoal::new(pos, level_root));
            }
            TileKind::Ball => cmds.add(SpawnBall::new(pos, level_root)),
            TileKind::Player => cmds.add(SpawnPlayer::new(pos, level_root)),
            TileKind::BallGoal => {
                cmds.add(SpawnGoal::new(pos, level_root));
                cmds.add(SpawnBall::new(pos, level_root))
            }
            TileKind::LampOff => {
                tile_cmds.insert((Name::new("Lamp"), SokobanBlock::Static, Lamp(false)));
            }
            TileKind::LampOn => {
                tile_cmds.insert((Name::new("Lamp"), SokobanBlock::Static, Lamp(true)));
            }
            TileKind::Floor => {}
        };
        storage.set(&position, tile_entity);
        cmds.entity(tilemap_entity).add_child(tile_entity);
    }
    cmds.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size,
            storage,
            texture: TilemapTexture::Single(tiles_handle.clone()),
            tile_size,
            ..default()
        },
        Name::new(format!("Level {}", **current_level)),
    ));
    cmds.entity(level_root).add_child(tilemap_entity);
}

fn reload_on_change(
    mut asset_events: EventReader<AssetEvent<Levels>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for ev in asset_events.read() {
        match ev {
            AssetEvent::Modified { id: _ } => {
                game_state.set(GameState::LevelTransition);
            }
            AssetEvent::Added { id: _ } => {}
            AssetEvent::Removed { id: _ } => {}
            AssetEvent::LoadedWithDependencies { id: _ } => {}
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Reflect)]
pub enum TileKind {
    Wall,
    Floor,
    Void,
    Ball,
    Rubber,
    Sand,
    Player,
    Goal,
    BallGoal,
    LampOff,
    LampOn,
}

impl From<u8> for TileKind {
    fn from(value: u8) -> Self {
        use TileKind::*;
        match value {
            b'#' => Wall,
            b'_' | b'.' => Floor,
            b'p' => Player,
            b'b' => Ball,
            b'@' => Void,
            b'|' => Rubber,
            b'~' => Sand,
            b'g' => Goal,
            b'B' => BallGoal,
            b'l' => LampOff,
            b'L' => LampOn,
            _ => {
                bevy::log::warn!("Couldnt parse tile kind defaulting to wall tile");
                Wall
            }
        }
    }
}

impl From<TileKind> for TileTextureIndex {
    fn from(value: TileKind) -> Self {
        let id = match value {
            TileKind::Wall => 1,
            TileKind::Floor => 0,
            TileKind::Void => 3,
            TileKind::Ball => 0,
            TileKind::Rubber => 4,
            TileKind::Sand => 2,
            TileKind::Player => 0,
            TileKind::Goal => 0,
            TileKind::BallGoal => 0,
            TileKind::LampOff => 5,
            TileKind::LampOn => 6,
        };
        Self(id)
    }
}

#[derive(TypePath, TypeUuid, Debug, Deserialize, Deref, DerefMut, Asset)]
#[uuid = "39cadc56-aa9c-4543-8540-a018b74b5052"]
pub struct Levels(pub Vec<Level>);

#[derive(Debug, Deserialize)]
struct StringLevels(pub Vec<StringLevel>);

#[derive(Default)]
pub struct LevelLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LevelLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse the ron: {0}")]
    RonError(#[from] ron::error::SpannedError),
}

impl AssetLoader for LevelLoader {
    type Asset = Levels;
    type Settings = ();
    type Error = LevelLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, std::result::Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let string_levels = ron::de::from_bytes::<StringLevels>(&bytes)?;

            let levels = Levels(
                string_levels
                    .0
                    .iter()
                    .map(|string_level| Level {
                        tiles: string_level
                            .tiles
                            .replace(['\n', ' '], "")
                            .as_bytes()
                            .iter()
                            .map(|byte| TileKind::from(*byte))
                            .collect::<Vec<TileKind>>(),
                        size: string_level.size,
                    })
                    .collect::<Vec<Level>>(),
            );

            Ok(levels)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["levels"]
    }
}

#[derive(Deserialize, Debug, Reflect)]
pub struct Level {
    pub tiles: Vec<TileKind>,
    pub size: UVec2,
}

#[derive(Deserialize, Debug, Reflect)]
struct StringLevel {
    pub tiles: String,
    pub size: UVec2,
}
