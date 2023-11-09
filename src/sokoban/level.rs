use std::u8;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadState, LoadedAsset},
    log,
    prelude::*,
    reflect::{TypePath, TypeUuid},
};
use bevy_ecs_tilemap::prelude::*;
use bevy_pile::tilemap::{access::TilemapAccess, layer::Layer};
use serde::Deserialize;

use super::{
    ball::SpawnBall, cleanup::DependOnState, collision::init_collision_map, goal::Goal,
    player::SpawnPlayer, rubber::Rubber, sand::Sand, void::Void, GameState, Pos, SokobanBlock,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LevelLoader>().add_asset::<Levels>();
        app.register_type::<Level>()
            .register_type::<AssetCollection>()
            .register_type::<CurrentLevel>();
        app.add_systems(
            OnEnter(GameState::Play),
            (spawn_level, apply_deferred, center_camera_on_level)
                .chain()
                .before(init_collision_map),
        );
        app.add_systems(
            PostUpdate,
            react_to_changes
                .run_if(in_state(GameState::Play))
                .run_if(on_event::<AssetEvent<Levels>>()),
        );
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AssetCollection {
    pub levels: Handle<Levels>,
    pub tiles: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct CurrentLevel(pub usize);

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

fn spawn_level(
    mut cmds: Commands,
    current_level: Res<CurrentLevel>,
    levels_assets: Res<Assets<Levels>>,
    asset_collection: Res<AssetCollection>,
    asset_server: Res<AssetServer>,
) {
    let levels_handle = &asset_collection.levels;
    // TODO Implement asset loading to guarantee that nothing happens before assets are loaded at startup
    if !matches!(
        asset_server.get_load_state(levels_handle),
        LoadState::Loaded
    ) {
        return;
    }

    let levels = levels_assets
        .get(levels_handle)
        .expect("Level handle should be loaded");
    let level = levels
        .get(**current_level)
        .expect("Current level should only ever be set to a valid level");

    let size = TilemapSize::from(level.size);
    let mut storage = TileStorage::empty(size);
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
        insert_needed_components(&mut cmds, tile, tile_entity, position, tilemap_entity);
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
            ..default()
        },
        Name::new("Level"),
        DependOnState(GameState::Play),
        Layer::World,
    ));
}

fn react_to_changes(
    mut cmds: Commands,
    mut asset_events: EventReader<AssetEvent<Levels>>,
    old_tilemap: Query<Entity, With<TileStorage>>,
    current_level: Res<CurrentLevel>,
    levels_assets: Res<Assets<Levels>>,
    asset_collection: Res<AssetCollection>,
) {
    for ev in asset_events.iter() {
        match ev {
            AssetEvent::Modified { handle } => {
                let levels = levels_assets
                    .get(handle)
                    .expect("Asset was modified so it should exist");
                let level = levels.get(**current_level).unwrap();
                cmds.entity(old_tilemap.get_single().unwrap())
                    .despawn_recursive();
                let size = TilemapSize::from(level.size);
                let mut storage = TileStorage::empty(size);
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
                    /* TODO find out why this doesnt work
                    let Some(tile_entity) = access.replace(
                        &position,
                        bevy_pile::tilemap::TileProperties {
                            id: TileTextureIndex::from(*tile),
                            flip: TileFlip::default(),
                        },
                        Layer::World,
                    ) else {
                        log::warn!("Shouldnt happen");
                        return;
                    };

                    let tilemap_entity = access
                        .tilemap_entity(Layer::World)
                        .expect("Tilemap should exist");
                    */
                    insert_needed_components(
                        &mut cmds,
                        tile,
                        tile_entity,
                        position,
                        tilemap_entity,
                    );
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
                        ..default()
                    },
                    Name::new("Level"),
                    DependOnState(GameState::Play),
                    Layer::World,
                ));
            }
            AssetEvent::Created { handle: _ } => {}
            AssetEvent::Removed { handle: _ } => {}
        }
    }
}

fn insert_needed_components(
    cmds: &mut Commands,
    tile: &TileKind,
    tile_entity: Entity,
    position: TilePos,
    tilemap_entity: Entity,
) {
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

impl From<TileKind> for TileTextureIndex {
    fn from(value: TileKind) -> Self {
        let id = match value {
            TileKind::Wall => 1,
            TileKind::Floor => 0,
            TileKind::Void => 3,
            TileKind::Ball => 0,
            TileKind::Rubber => 5,
            TileKind::Sand => 2,
            TileKind::Player => 0,
            TileKind::Goal => 4,
        };
        Self(id)
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
