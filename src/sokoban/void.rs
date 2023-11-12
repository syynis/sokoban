use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use super::{momentum::apply_momentum, Pos};

pub struct VoidPlugin;

impl Plugin for VoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            apply_void
                .before(apply_momentum)
                .run_if(on_timer(Duration::from_millis(50))),
        );
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
        if void_query.iter().any(|void_pos| void_pos == pos) {
            cmds.entity(entity).despawn_recursive();
        }
    }
}
