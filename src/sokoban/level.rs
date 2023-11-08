use std::u8;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadState, LoadedAsset},
    prelude::*,
    reflect::{TypePath, TypeUuid},
};
use bevy_ecs_tilemap::prelude::*;
use serde::Deserialize;

use crate::AssetCollection;

use super::{
    ball::SpawnBall, cleanup::DependOnState, collision::init_collision_map, goal::Goal,
    player::SpawnPlayer, rubber::Rubber, sand::Sand, void::Void, GameState, Pos, SokobanBlock,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LevelLoader>().add_asset::<Levels>();
        app.register_type::<Level>();
        app.add_systems(
            OnEnter(GameState::Play),
            (spawn_level, apply_deferred)
                .chain()
                .before(init_collision_map),
        );
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct CurrentLevel(pub usize);

fn spawn_level(
    mut cmds: Commands,
    current_level: Res<CurrentLevel>,
    levels_assets: Res<Assets<Levels>>,
    asset_collection: Res<AssetCollection>,
    asset_server: Res<AssetServer>,
) {
    let levels_handle = &asset_collection.levels;
    if !matches!(
        asset_server.get_load_state(levels_handle),
        LoadState::Loaded
    ) {
        return;
    }

    let Some(levels) = levels_assets.get(levels_handle) else {
        return;
    };
    let Some(level) = levels.get(**current_level) else {
        bevy::log::warn!("Tried to load level that doesnt exist");
        return;
    };

    let size = TilemapSize::from(level.size);
    let mut storage = TileStorage::empty(size);
    let tilemap_entity = cmds.spawn_empty().id();
    let tile_size = TilemapTileSize::from(Vec2::splat(8.));
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    for (idx, tile) in level.tiles.iter().enumerate() {
        let position = TilePos {
            x: idx as u32 % level.size.x,
            y: idx as u32 / level.size.x,
        };

        let id = match tile {
            TileKind::Wall => 1,
            TileKind::Floor => 0,
            TileKind::Void => 3,
            TileKind::Ball => 0,
            TileKind::Rubber => 5,
            TileKind::Sand => 2,
            TileKind::Player => 0,
            TileKind::Goal => 4,
        };

        let tile_entity = cmds
            .spawn((
                Name::new("Tile"),
                TileBundle {
                    position,
                    texture_index: TileTextureIndex(id),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                },
            ))
            .id();

        let mut tile_cmds = cmds.entity(tile_entity);
        if !matches!(tile, TileKind::Floor) {
            tile_cmds.insert(Pos(position));
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
                tile_cmds.insert((Name::new("Goal"), Goal));
            }
            TileKind::Ball => cmds.add(SpawnBall::new(Pos(position), tilemap_entity)),
            TileKind::Player => cmds.add(SpawnPlayer::new(Pos(position), tilemap_entity)),
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
            texture: TilemapTexture::Single(asset_collection.tiles.clone()),
            tile_size,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Name::new("Level"),
        DependOnState(GameState::Play),
    ));
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
            _ => {
                bevy::log::warn!("Couldnt parse tile kind defaulting to wall tile");
                Wall
            }
        }
    }
}

#[derive(TypePath, TypeUuid, Debug, Deserialize, Deref, DerefMut)]
#[uuid = "39cadc56-aa9c-4543-8540-a018b74b5052"]
pub struct Levels(pub Vec<Level>);

#[derive(Debug, Deserialize)]
struct StringLevels(pub Vec<StringLevel>);

#[derive(Default)]
pub struct LevelLoader;

impl AssetLoader for LevelLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let string_levels = ron::de::from_bytes::<StringLevels>(bytes)?;

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
            load_context.set_default_asset(LoadedAsset::new(levels));

            Ok(())
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
