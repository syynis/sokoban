use bevy::prelude::*;

use super::{momentum::Momentum, Pos};

pub struct RubberPlugin;

impl Plugin for RubberPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_rubber);
    }
}

#[derive(Component)]
pub struct Rubber;

fn handle_rubber(
    rubber_query: Query<&Pos, With<Rubber>>,
    mut momentum_query: Query<(&Pos, &mut Momentum)>,
) {
    for (pos, mut momentum) in momentum_query.iter_mut() {
        if let Some(dir) = **momentum {
            let mut dest = *pos;
            dest.add_dir(dir);

            if rubber_query
                .iter()
                .find(|rubber_pos| **rubber_pos == dest)
                .is_some()
            {
                momentum.replace(dir.opposite());
            }
        }
    }
}
