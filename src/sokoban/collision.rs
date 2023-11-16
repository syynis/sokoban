use bevy::{log, prelude::*};
use bevy_pile::grid::Grid;

use super::{level::LevelMarker, Dir, GameState, Pos, SokobanBlock};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CollisionMap>()
            .add_systems(
                OnTransition {
                    from: GameState::LevelTransition,
                    to: GameState::Play,
                },
                init_collision_map,
            )
            .add_systems(
                PostUpdate,
                sync_collision_map.run_if(in_state(GameState::Play)),
            );
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CollisionMap {
    map: Grid<Option<(Entity, SokobanBlock)>>,
}

impl Default for CollisionMap {
    fn default() -> Self {
        Self {
            map: Grid::new(IVec2::new(0, 0), None),
        }
    }
}

pub fn init_collision_map(
    mut cmds: Commands,
    tilemap: Query<&LevelMarker, Added<LevelMarker>>,
    sokoban_entities: Query<(Entity, &Pos, &SokobanBlock)>,
) {
    let Some(level_marker) = tilemap.get_single().ok() else {
        log::warn!("Not exactly one tilemap");
        return;
    };
    let size = level_marker.size;
    log::debug!("Initialized collision map");
    let mut map = Grid::new(IVec2::new(size.x as i32, size.y as i32), None);
    for (entity, pos, block) in sokoban_entities.iter() {
        let pos = IVec2::from(pos);
        map.set(pos, Some((entity, *block)));
    }
    cmds.insert_resource(CollisionMap { map });
}

// TODO dont rebuild but instead only change moved entities
fn sync_collision_map(
    mut collision: ResMut<CollisionMap>,
    sokoban_entities: Query<(Entity, &Pos, &SokobanBlock)>,
) {
    collision.map.iter_mut().for_each(|(_, elem)| {
        elem.take();
    });
    for (entity, pos, block) in sokoban_entities.iter() {
        collision.map.set(IVec2::from(pos), Some((entity, *block)));
    }
}

pub enum CollisionResult {
    Push(Vec<Entity>),
    Wall,
    OutOfBounds,
}

impl CollisionMap {
    pub fn push_collision(&self, pusher_pos: IVec2, direction: Dir) -> CollisionResult {
        let Some(Some((pusher, _))) = self.map.get(pusher_pos) else {
            return CollisionResult::OutOfBounds;
        };

        let move_in_dir = |pos| -> IVec2 { pos + IVec2::from(direction) };
        let mut moving_entities = Vec::new();
        let mut current_pos = pusher_pos;
        let mut dest = move_in_dir(current_pos);
        let mut pusher = pusher;
        while let Some(dest_entity) = self.map.get(dest) {
            match dest_entity {
                Some((pushed, block)) => match block {
                    SokobanBlock::Static => {
                        return CollisionResult::Wall;
                    }
                    SokobanBlock::Dynamic => {
                        moving_entities.push(*pusher);
                        pusher = pushed;
                        current_pos = dest;
                        dest = move_in_dir(current_pos);
                    }
                },
                None => {
                    moving_entities.push(*pusher);
                    break;
                }
            }
        }
        CollisionResult::Push(moving_entities)
    }
}
