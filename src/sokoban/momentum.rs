use bevy::prelude::*;

use super::{handle_sokoban_events, Dir, Pos, SokobanEvent};

pub struct MomentumPlugin;

impl Plugin for MomentumPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Momentum>().add_systems(
            Update,
            (
                handle_momentum.before(handle_sokoban_events),
                apply_momentum.after(handle_sokoban_events),
            ),
        );
    }
}

#[derive(Default, Component, Copy, Clone, Deref, DerefMut, Reflect)]
pub struct Momentum(pub Option<Dir>);

fn handle_momentum(
    mut sokoban_events: EventWriter<SokobanEvent>,
    momentum_query: Query<(Entity, &Momentum)>,
) {
    for (entity, momentum) in momentum_query.iter() {
        if let Some(direction) = **momentum {
            sokoban_events.send(SokobanEvent::Momentum { entity, direction });
        }
    }
}

pub fn apply_momentum(mut momentum_query: Query<(&mut Pos, &Momentum)>) {
    for (mut pos, momentum) in momentum_query.iter_mut() {
        if let Some(dir) = **momentum {
            pos.add_dir(dir);
        }
    }
}
