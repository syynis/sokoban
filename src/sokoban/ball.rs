use bevy::{ecs::system::Command, prelude::*};

use super::{
    history::History, level::AssetCollection, momentum::Momentum, PixelOffset, Pos, SokobanBlock,
    ZOffset,
};

#[derive(Component)]
pub struct Ball;

pub struct SpawnBall {
    pub pos: Pos,
    pub parent: Entity,
}

impl SpawnBall {
    pub fn new(pos: Pos, parent: Entity) -> Self {
        Self { pos, parent }
    }
}

impl Command for SpawnBall {
    fn apply(self, world: &mut World) {
        let assets = world.resource::<AssetCollection>();
        let ball_handle: Handle<Image> = assets.ball.clone();

        world
            .entity_mut(self.parent)
            .with_children(|child_builder| {
                child_builder.spawn((
                    Name::new("Ball"),
                    Ball,
                    self.pos,
                    History::<Pos>::default(),
                    SokobanBlock::Dynamic,
                    SpriteBundle {
                        texture: ball_handle,
                        ..default()
                    },
                    PixelOffset(UVec2::Y),
                    ZOffset(0.5),
                    Momentum::default(),
                ));
            });
    }
}
