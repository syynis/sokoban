use std::path::PathBuf;

use bevy::{asset::AssetPath, log, prelude::*};
use bevy_ecs_tilemap::{
    prelude::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TileColor, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_pile::{cursor::WorldCursorPlugin, tilemap::TileCursorPlugin};
use ron::ser::PrettyConfig;
use sokoban::{
    ball::SpawnBall,
    collision::init_collision_map,
    goal::Goal,
    level::{load_level, LevelFormat, Tile},
    player::SpawnPlayer,
    rubber::Rubber,
    sand::Sand,
    void::Void,
    GameState, Pos, SokobanBlock, SokobanPlugin,
};

use crate::sokoban::level::LevelLoader;

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
    app.add_systems(
        Startup,
        (setup, apply_deferred)
            .chain()
            .before(init_collision_map)
            .run_if(in_state(GameState::Play)),
    );
    app.add_systems(Update, (load, load_level));
    app.run();
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

    let map = vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 5, 5, 5, 0, 0, 1],
        vec![1, 0, 0, 2, 0, 0, 0, 2, 2, 0, 5, 2, 2, 2, 0, 0, 5, 1, 0, 1],
        vec![1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 5, 5, 5, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 0, 0, 6, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 2, 2, 0, 0, 0, 5, 2, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 4, 0, 2, 2, 2, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 2, 2, 2, 0, 0, 5, 1, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

    #[rustfmt::skip]
    let map2 = vec![
        1,1,1,
        1,0,1,
        1,1,1
    ];

    let tiles: Vec<Tile> = map2
        .iter()
        .map(|id| if *id == 1 { Tile::Wall } else { Tile::Floor })
        .collect();
    let format = LevelFormat {
        tiles,
        size: UVec2::splat(3),
    };

    log::info!(
        "{}",
        ron::ser::to_string_pretty(&format, PrettyConfig::default()).unwrap()
    );

    let tiles: Handle<Image> = asset_server.load("tiles.png");

    let size = TilemapSize::from(UVec2::splat(map.len() as u32));
    let mut storage = TileStorage::empty(size);

    let tilemap_entity = cmds.spawn_empty().id();
    for (y, row) in map.iter().rev().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            let tile_idx = match *tile {
                0 => 0,
                1 => 1,
                3 => 4,
                4 => 2,
                5 => 3,
                6 => 1,
                _ => 0,
            };
            let tile_entity = cmds
                .spawn((
                    Name::new("Tile"),
                    TileBundle {
                        position: pos,
                        texture_index: TileTextureIndex(tile_idx),
                        tilemap_id: TilemapId(tilemap_entity),
                        ..default()
                    },
                ))
                .id();
            if *tile == 1 {
                cmds.entity(tile_entity)
                    .insert((SokobanBlock::Static, Pos(pos)));
            }

            if *tile == 2 {
                cmds.add(SpawnBall { pos: Pos(pos) });
            }

            if *tile == 3 {
                cmds.entity(tile_entity).insert((Goal, Pos(pos)));
            }

            if *tile == 4 {
                cmds.entity(tile_entity)
                    .insert((Sand, Pos(pos), TileColor(Color::YELLOW)));
            }
            if *tile == 5 {
                cmds.entity(tile_entity)
                    .insert((Name::new("Void"), Void, Pos(pos)));
            }
            if *tile == 6 {
                cmds.entity(tile_entity).insert((
                    SokobanBlock::Static,
                    Name::new("Rubber"),
                    Rubber,
                    Pos(pos),
                    TileColor(Color::BLUE),
                ));
            }

            storage.set(&pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize::from(Vec2::splat(8.));
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    cmds.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size,
            storage,
            texture: TilemapTexture::Single(tiles),
            tile_size,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Name::new("Level"),
    ));

    cmds.add(SpawnPlayer {
        pos: Pos::new(2, 2),
    });
}

pub fn load(mut cmds: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Q) {
        let path = AssetPath::new(
            PathBuf::from("/home/synis/dev/sokoban/assets/test.ron"),
            None,
        )
        .path()
        .to_path_buf();
        log::info!("{:?}", path);
        let loaded = LevelLoader::new(path);

        cmds.spawn(loaded);
    }
}
