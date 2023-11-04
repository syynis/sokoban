use std::path::PathBuf;

use anyhow::{Context, Result};
use bevy::{
    log,
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::future;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
    Void,
    Ball,
    Rubber,
    Sand,
    Player,
}

#[derive(Serialize, Deserialize)]
pub struct LevelFormat {
    pub tiles: Vec<Tile>,
    pub size: UVec2,
}

#[derive(Component)]
pub struct LevelLoader {
    path: PathBuf,
    task: Task<Result<LevelFormat>>,
}

impl LevelLoader {
    pub fn new(path: PathBuf) -> Self {
        let path_copy = path.clone();
        let task_pool = IoTaskPool::get();
        let task = task_pool.spawn(async move {
            let buf = std::fs::read_to_string(path).context("Failed to read file")?;
            let map = ron::from_str(&buf).context("Failed to parse level")?;
            Ok(map)
        });

        Self {
            path: path_copy,
            task,
        }
    }
}

pub fn load_level(mut cmds: Commands, mut loader: Query<(Entity, &mut LevelLoader)>) {
    let Ok((entity, mut loader)) = loader.get_single_mut() else {
        return;
    };

    let Some(result) = future::block_on(future::poll_once(&mut loader.task)) else {
        return;
    };

    match result {
        Ok(level) => {
            let mut entity_ref = cmds.entity(entity);
            let name = loader.path.file_stem().unwrap().to_string_lossy();
            entity_ref
                .remove::<LevelLoader>()
                .insert(Name::new(format!("{}", name)));
        }
        Err(e) => {
            log::warn!("Failed to load map");
            cmds.entity(entity).despawn()
        }
    }
}
