use bevy::{ecs::system::Command, prelude::*};

use super::{
    ball::Ball,
    history::{CurrentTime, HandleHistoryEvents, History, HistoryEvent},
    level::LevelRoot,
    player::{MovementTimer, Player},
    DynamicBundle, Pos,
};

pub struct CommandHistoryPlugin;

impl Plugin for CommandHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandHistory>().add_systems(
            Update,
            (rewind, apply_deferred).chain().before(HandleHistoryEvents),
        );
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct CommandHistory(Vec<(usize, Box<dyn UndoableCommand>)>);

pub fn rewind(
    mut cmds: Commands,
    mut history_events: EventReader<HistoryEvent>,
    mut command_history: ResMut<CommandHistory>,
    current_time: Res<CurrentTime>,
) {
    for ev in history_events.read() {
        match ev {
            HistoryEvent::Record => {}
            HistoryEvent::Rewind => loop {
                let Some((time, command)) = command_history.last() else {
                    break;
                };
                if *time == **current_time {
                    command.rollback(&mut cmds);
                    command_history.pop();
                } else {
                    break;
                }
            },
            HistoryEvent::Reset => {}
        }
    }
}

pub trait UndoableCommand: Send + Sync + 'static {
    fn execute(&self, world: &mut World) -> Box<dyn UndoableCommand>;
    fn rollback(&self, cmds: &mut Commands);
}

pub struct DespawnSokobanEntityCommand(pub Entity);

impl Command for DespawnSokobanEntityCommand {
    fn apply(self, world: &mut World) {
        let (pos, history, texture, is_player, is_ball) =
            if let Ok((pos, history, texture, is_player, is_ball)) = world
                .query::<(&Pos, &History<Pos>, &Handle<Image>, Has<Player>, Has<Ball>)>()
                .get(world, self.0)
            {
                (*pos, history.clone(), texture.clone(), is_player, is_ball)
            } else {
                todo!()
            };

        let Ok(level_entity) = world
            .query_filtered::<Entity, With<LevelRoot>>()
            .get_single(world)
        else {
            todo!()
        };

        let current_time = *world.resource::<CurrentTime>();
        world.resource_scope(|world, mut command_history: Mut<CommandHistory>| {
            if is_player {
                let despawn = DespawnSokobanEntity {
                    entity: self.0,
                    pos,
                    history,
                    texture,
                    level_entity,
                    bundle: (
                        Name::new("Player"),
                        Player,
                        DynamicBundle::default(),
                        MovementTimer::default(),
                    ),
                };
                command_history.push((*current_time, despawn.execute(world)));
            } else if is_ball {
                let despawn = DespawnSokobanEntity {
                    entity: self.0,
                    pos,
                    history,
                    texture,
                    level_entity,
                    bundle: (Name::new("Ball"), Ball, DynamicBundle::default()),
                };
                command_history.push((*current_time, despawn.execute(world)));
            }
        });
    }
}

#[derive(Clone)]
pub struct DespawnSokobanEntity<B>
where
    B: Bundle + Clone,
{
    pub entity: Entity,
    pub pos: Pos,
    pub history: History<Pos>,
    pub texture: Handle<Image>,
    pub level_entity: Entity,
    pub bundle: B,
}

impl<B> UndoableCommand for DespawnSokobanEntity<B>
where
    B: Bundle + Clone,
{
    fn execute(&self, world: &mut World) -> Box<dyn UndoableCommand> {
        world.entity_mut(self.entity).despawn_recursive();
        Box::new(self.clone())
    }

    fn rollback(&self, cmds: &mut Commands) {
        cmds.entity(self.level_entity).with_children(|parent| {
            parent.spawn((
                self.pos,
                self.history.clone(),
                SpriteBundle {
                    texture: self.texture.clone(),
                    transform: Transform::from_translation(Vec3::Z),
                    ..default()
                },
                self.bundle.clone(),
            ));
        });
    }
}
