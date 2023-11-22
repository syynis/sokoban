use bevy::{ecs::system::Command, prelude::*};

use super::{history::History, level::AssetCollection, momentum::Momentum, Pos, SokobanBlock};

#[derive(Component)]
pub struct Ball;

pub struct SpawnBall {
    pub pos: Pos,
    pub tilemap_entity: Entity,
}

impl SpawnBall {
    pub fn new(pos: Pos, tilemap_entity: Entity) -> Self {
        Self {
            pos,
            tilemap_entity,
        }
    }
}

impl Command for SpawnBall {
    fn apply(self, world: &mut World) {
        let texture = world.resource::<AssetCollection>().ball.clone();

        world
            .entity_mut(self.tilemap_entity)
            .with_children(|child_builder| {
                child_builder.spawn((
                    Name::new("Ball"),
                    Ball,
                    self.pos,
                    History::<Pos>::default(),
                    SokobanBlock::Dynamic,
                    SpriteBundle {
                        texture,
                        transform: Transform::from_translation(2. * Vec3::Z),
                        ..default()
                    },
                    Momentum::default(),
                ));
            });
    }
}
