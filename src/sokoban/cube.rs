use bevy::{ecs::system::Command, prelude::*};
use bevy_pile::tilemap::tile_to_world_pos;

use super::{history::History, Pos, SokobanBlock};

#[derive(Component)]
pub struct Cube;

pub struct SpawnCube {
    pub pos: Pos,
}

impl Command for SpawnCube {
    fn apply(self, world: &mut World) {
        world.spawn((
            Name::new("Box"),
            Cube,
            self.pos,
            History::<Pos>::default(),
            SokobanBlock::Dynamic,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::splat(16.)),
                    ..default()
                },
                transform: Transform::from_translation(tile_to_world_pos(&self.pos).extend(1.)),
                ..default()
            },
        ));
    }
}
