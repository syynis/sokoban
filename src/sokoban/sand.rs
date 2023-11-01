use bevy::prelude::*;

use super::{
    momentum::{apply_momentum, Momentum},
    Pos,
};

pub struct SandPlugin;

impl Plugin for SandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_sand.before(apply_momentum));
    }
}

#[derive(Component)]
pub struct Sand;

fn apply_sand(
    sand_query: Query<&Pos, With<Sand>>,
    mut momentum_query: Query<(&Pos, &mut Momentum)>,
) {
    for (pos, mut momentum) in momentum_query.iter_mut() {
        if sand_query
            .iter()
            .find(|sand_pos| *sand_pos == pos)
            .is_some()
        {
            momentum.take();
        }
    }
}
