use bevy::prelude::*;

use super::{Dir, SokobanEvent};

pub struct MomentumPlugin;

impl Plugin for MomentumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_momentum);
    }
}

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct Momentum(pub Option<Dir>);

fn handle_momentum(
    mut sokoban_events: EventWriter<SokobanEvent>,
    momentum_query: Query<(Entity, &Momentum)>,
) {
    for (entity, momentum) in momentum_query.iter() {
        if let Some(direction) = **momentum {
            sokoban_events.send(SokobanEvent::Move { entity, direction });
        }
    }
}
