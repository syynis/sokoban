use std::u8;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::{TypePath, TypeUuid},
};
use serde::Deserialize;

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
                return Wall;
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
                            .replace("\n", "")
                            .replace(" ", "")
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
