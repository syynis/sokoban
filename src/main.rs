use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_pile::{cursor::WorldCursorPlugin, tilemap::TileCursorPlugin};
use sokoban::{level::Levels, SokobanPlugin};

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
    app.insert_resource(ClearColor(Color::ANTIQUE_WHITE));
    app.register_type::<AssetCollection>();
    app.add_systems(Startup, setup);
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
