use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    log::LogPlugin,
    prelude::*,
};
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
    app.edit_schedule(Main, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Warn,
            ..default()
        });
    });
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Sokoban".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "info,wgpu_core=warn,wgpu_hal=warn,sokoban=debug".into(),
            }),
        PanCamPlugin,
        WorldCursorPlugin::<PanCam>::default(),
        WorldInspectorPlugin::default(),
        TileCursorPlugin,
        SokobanPlugin,
    ))
    .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
    .add_systems(Startup, setup)
    .run();
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
