use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    reflect::{TypePath, TypeUuid},
};
use serde::Deserialize;
use thiserror::Error;

use super::{
    ball::SpawnBall,
    cleanup::DependOnState,
    collision::init_collision_map,
    level_select::CurrentLevel,
    player::SpawnPlayer,
    tile_behaviour::{Rubber, Sand, SpawnGoal, Void},
    GameState, Pos, SokobanBlock,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(LevelLoader)
            .init_asset::<Levels>()
            .register_type::<Level>()
            .register_type::<AssetCollection>()
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

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AssetCollection {
    pub levels: Handle<Levels>,
    pub wall: Handle<Image>,
    pub floor: Handle<Image>,
    pub void: Handle<Image>,
    pub sand: Handle<Image>,
    pub rubber: Handle<Image>,
    pub player: Handle<Image>,
    pub ball: Handle<Image>,
    pub goal: Handle<Image>,
}

fn center_camera_on_level(
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    tilemap_q: Query<&LevelMarker, Without<Camera>>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_q.get_single_mut() else {
        return;
    };

    let Ok(level_marker) = tilemap_q.get_single() else {
        return;
    };
    let center = level_marker.size.as_vec2() * 8. / 2. - 4.;
    camera_transform.translation = center.extend(camera_transform.translation.z);
    projection.scale = 0.15;
}

#[derive(Component)]
pub struct LevelMarker {
    pub size: UVec2,
}

fn spawn_level(
    mut cmds: Commands,
    current_level: Res<CurrentLevel>,
    levels_assets: Res<Assets<Levels>>,
    assets: Res<AssetCollection>,
) {
    let levels_handle = &assets.levels;

    let levels = levels_assets
        .get(levels_handle)
        .expect("Level handle should be loaded");
    let level = levels
        .get(**current_level)
        .expect("Current level should only ever be set to a valid level");

    let tilemap_entity = cmds.spawn_empty().id();

    for (idx, tile) in level.tiles.iter().enumerate() {
        let position = UVec2 {
            x: idx as u32 % level.size.x,
            y: level.size.y - (idx as u32 / level.size.x) - 1,
        };

        let texture = match tile {
            TileKind::Wall => &assets.wall,
            TileKind::Floor
            | TileKind::Player
            | TileKind::Goal
            | TileKind::Ball
            | TileKind::BallGoal => &assets.floor,
            TileKind::Void => &assets.void,
            TileKind::Rubber => &assets.rubber,
            TileKind::Sand => &assets.sand,
        }
        .clone();

        let tile_entity = cmds
            .spawn((
                Name::new("Tile"),
                SpriteBundle {
                    texture,
                    ..default()
                },
            ))
            .id();

        let pos = Pos(position);
        let mut tile_cmds = cmds.entity(tile_entity);
        if matches!(
            tile,
            TileKind::Floor
                | TileKind::Player
                | TileKind::Goal
                | TileKind::Ball
                | TileKind::BallGoal
        ) {
            tile_cmds.insert(SpatialBundle {
                transform: Transform::from_translation(
                    pos.to_world_pos().extend(-(1. + level.size.y as f32)),
                ),
                ..default()
            });
        } else {
            tile_cmds.insert(pos);
        }
        match tile {
            TileKind::Sand => {
                tile_cmds.insert((pos, Name::new("Sand"), Sand));
            }
            TileKind::Rubber => {
                tile_cmds.insert((pos, Name::new("Rubber"), SokobanBlock::Static, Rubber));
            }
            TileKind::Wall => {
                tile_cmds.insert((pos, Name::new("Wall"), SokobanBlock::Static));
            }
            TileKind::Void => {
                tile_cmds.insert((pos, Name::new("Void"), Void));
            }
            TileKind::Goal => cmds.add(SpawnGoal::new(pos, tilemap_entity)),
            TileKind::Ball => cmds.add(SpawnBall::new(pos, tilemap_entity)),
            TileKind::Player => cmds.add(SpawnPlayer::new(pos, tilemap_entity)),
            TileKind::BallGoal => {
                cmds.add(SpawnGoal::new(pos, tilemap_entity));
                cmds.add(SpawnBall::new(pos, tilemap_entity));
            }
            TileKind::Floor => {}
        };
        cmds.entity(tilemap_entity).add_child(tile_entity);
    }
    cmds.entity(tilemap_entity).insert((
        LevelMarker { size: level.size },
        SpatialBundle::default(),
        Name::new(format!("Level {}", **current_level)),
        DependOnState(vec![GameState::Play, GameState::Pause]),
    ));
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
            _ => {
                bevy::log::warn!("Couldnt parse tile kind defaulting to wall tile");
                Wall
            }
        }
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
