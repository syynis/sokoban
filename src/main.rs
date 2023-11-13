use std::time::Duration;

use bevy::{asset::ChangeWatcher, log::LogPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_pile::{cursor::WorldCursorPlugin, tilemap::TileCursorPlugin};
use sokoban::{
    level::{AssetCollection, Levels},
    SokobanPlugin,
};

pub mod sokoban;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                watch_for_changes: Some(ChangeWatcher {
                    delay: Duration::from_secs_f64(0.5),
                }),
                ..default()
            })
            .set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "info,wgpu_core=warn,wgpu_hal=warn,sokoban=debug".into(),
            }),
        PanCamPlugin,
        WorldCursorPlugin::<PanCam>::default(),
        WorldInspectorPlugin::default(),
        TileCursorPlugin,
        SokobanPlugin,
    ));
    app.insert_resource(ClearColor(Color::ANTIQUE_WHITE));
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

    let levels: Handle<Levels> = asset_server.load("test.levels");
    let tiles: Handle<Image> = asset_server.load("tiles.png");
    cmds.insert_resource(AssetCollection { levels, tiles });
}
