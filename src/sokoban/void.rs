use bevy::prelude::*;

use super::{momentum::apply_momentum, Pos};

pub struct VoidPlugin;

impl Plugin for VoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_void.before(apply_momentum));
    }
}

#[derive(Component)]
pub struct Void;

fn apply_void(
    mut cmds: Commands,
    void_query: Query<&Pos, With<Void>>,
    sokoban_query: Query<(Entity, &Pos), Without<Void>>,
) {
    for (entity, pos) in sokoban_query.iter() {
        if void_query
            .iter()
            .find(|void_pos| *void_pos == pos)
            .is_some()
        {
            cmds.entity(entity).despawn_recursive();
        }
    }
}
