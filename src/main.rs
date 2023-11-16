#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{log::LogPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use sokoban::{
    level::{AssetCollection, Levels},
    SokobanPlugin,
};

pub mod sokoban;

fn main() {
    let mut app = App::new();

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
        WorldInspectorPlugin::default(),
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
    let wall: Handle<Image> = asset_server.load("wall.png");
    let floor: Handle<Image> = asset_server.load("floor.png");
    let player: Handle<Image> = asset_server.load("player.png");
    let ball: Handle<Image> = asset_server.load("ball.png");
    let goal: Handle<Image> = asset_server.load("goal.png");
    let sand: Handle<Image> = asset_server.load("sand.png");
    let void: Handle<Image> = asset_server.load("void.png");
    let rubber: Handle<Image> = asset_server.load("rubber.png");
    cmds.insert_resource(AssetCollection {
        levels,
        wall,
        floor,
        sand,
        void,
        rubber,
        player,
        ball,
        goal,
    });
}
