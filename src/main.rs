use bevy::{asset::LoadState, log, prelude::*};
use bevy_ecs_tilemap::{
    prelude::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TileColor, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_pile::{cursor::WorldCursorPlugin, tilemap::TileCursorPlugin};
use sokoban::{
    ball::SpawnBall,
    collision::init_collision_map,
    goal::Goal,
    level::{Level, LevelLoader, Levels, TileKind},
    player::SpawnPlayer,
    GameState, Pos, SokobanBlock, SokobanPlugin,
};

use crate::sokoban::{rubber::Rubber, sand::Sand, void::Void};

pub mod sokoban;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        WorldCursorPlugin::<PanCam>::default(),
        WorldInspectorPlugin::default(),
        TileCursorPlugin,
        SokobanPlugin,
    ));
    app.init_asset_loader::<LevelLoader>();
    app.add_asset::<Levels>();
    app.register_type::<Level>()
        .register_type::<AssetCollection>();
    app.add_systems(
        Startup,
        (setup, apply_deferred)
            .chain()
            .before(init_collision_map)
            .run_if(in_state(GameState::LevelSelect)),
    );
    app.add_systems(Update, (print_levels, spawn_level));
    app.run();
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AssetCollection {
    pub levels: Handle<Levels>,
    pub tiles: Handle<Image>,
}

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));

    let levels: Handle<Levels> = asset_server.load("test.levels");
    let tiles: Handle<Image> = asset_server.load("tiles.png");
    cmds.insert_resource(AssetCollection { levels, tiles });
}

fn print_levels(
    keys: Res<Input<KeyCode>>,
    levels_assets: Res<Assets<Levels>>,
    asset_server: Res<AssetServer>,
    asset_collection: Res<AssetCollection>,
) {
    if keys.just_pressed(KeyCode::Q) {
        let levels = &asset_collection.levels;

        if matches!(asset_server.get_load_state(levels.id()), LoadState::Loaded) {
            let levels = levels_assets.get(levels).unwrap();
            for level in levels.0.iter() {
                log::info!("{:?}", level);
            }
        }
    }
}

fn spawn_level(
    mut cmds: Commands,
    tilemap_q: Query<Entity>,
    keys: Res<Input<KeyCode>>,
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
    let level = levels.first().unwrap();

    if keys.just_pressed(KeyCode::S) {
        if let Ok(tilemap_entity) = tilemap_q.get_single() {
            cmds.entity(tilemap_entity).despawn_recursive();
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
        ));
    }
}
