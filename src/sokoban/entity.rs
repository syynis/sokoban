use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileStorage;

use super::{
    history::History, level::AssetCollection, player::Player, DynamicBundle, GameState, Pos,
};

pub struct CommandHistoryPlugin;

impl Plugin for CommandHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandHistory>().add_systems(
            Update,
            (despawn, rollback).run_if(in_state(GameState::Play)),
        );
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct CommandHistory(Vec<Box<dyn UndoableCommand>>);

pub fn despawn(
    mut cmds: Commands,
    assets: Res<AssetCollection>,
    level_q: Query<Entity, With<TileStorage>>,
    player_q: Query<(Entity, &Pos, &History<Pos>), With<Player>>,
    mut command_history: ResMut<CommandHistory>,
    keys: Res<Input<KeyCode>>,
) {
    let Ok((entity, pos, history)) = player_q.get_single() else {
        return;
    };
    let Ok(level_entity) = level_q.get_single() else {
        return;
    };
    let player_texture = assets.player.clone();
    let despawn = DespawnSokobanEntity {
        entity,
        pos: *pos,
        history: history.clone(),
        texture: player_texture,
        level_entity,
        bundle: (Name::new("Player"), Player, DynamicBundle::default()),
    };
    if keys.just_pressed(KeyCode::J) {
        println!("Despawn");
        command_history.push(despawn.execute(&mut cmds));
    }
}

pub fn rollback(
    mut cmds: Commands,
    mut command_history: ResMut<CommandHistory>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::H) {
        println!("Rollback");
        if let Some(last) = command_history.pop() {
            last.rollback(&mut cmds);
        }
    }
}

pub trait UndoableCommand: Send + Sync + 'static {
    fn execute(&self, cmds: &mut Commands) -> Box<dyn UndoableCommand>;
    fn rollback(&self, cmds: &mut Commands);
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
    fn execute(&self, cmds: &mut Commands) -> Box<dyn UndoableCommand> {
        cmds.entity(self.entity).despawn_recursive();
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
