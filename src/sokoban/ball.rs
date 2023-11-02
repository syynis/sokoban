use bevy::{ecs::system::Command, prelude::*};
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_pile::tilemap::tile_to_world_pos;

use super::{history::History, momentum::Momentum, Pos, SokobanBlock};

#[derive(Component)]
pub struct Ball;

pub struct SpawnBall {
    pub pos: Pos,
}

impl Command for SpawnBall {
    fn apply(self, world: &mut World) {
        let asset_server = world.resource::<AssetServer>();
        let ball_handle: Handle<Image> = asset_server.load("ball.png");
        world.spawn((
            Name::new("Ball"),
            Ball,
            self.pos,
            History::<Pos>::default(),
            SokobanBlock::Dynamic,
            SpriteBundle {
                texture: ball_handle,
                transform: Transform::from_translation(
                    tile_to_world_pos(&self.pos, &TilemapGridSize { x: 8., y: 8. }).extend(1.),
                ),
                ..default()
            },
            Momentum::default(),
        ));
    }
}
