use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_pile::{cursor::WorldCursorPlugin, tilemap::TileCursorPlugin};
use sokoban::{cube::SpawnCube, player::SpawnPlayer, Pos, SokobanPlugin};

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
    app.add_systems(Startup, setup);
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
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

    let tiles: Handle<Image> = asset_server.load("tiles.png");

    let size = TilemapSize::from(UVec2::splat(10));
    let mut storage = TileStorage::empty(size);

    let tilemap_entity = cmds.spawn_empty().id();
    for (y, row) in map.iter().rev().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            let tile_entity = cmds
                .spawn((
                    Name::new("Tile"),
                    TileBundle {
                        position: pos,
                        texture_index: TileTextureIndex(*tile),
                        tilemap_id: TilemapId(tilemap_entity),
                        ..default()
                    },
                ))
                .id();
            storage.set(&pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize::from(Vec2::splat(16.));
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
        pos: Pos::new(0, 0),
    });

    cmds.add(SpawnCube {
        pos: Pos::new(1, 1),
    });
    cmds.add(SpawnCube {
        pos: Pos::new(2, 2),
    });
}
