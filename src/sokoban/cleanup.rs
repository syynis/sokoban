use bevy::prelude::*;

pub fn cleanup_all_with<T: Component>(mut cmds: Commands, query: Query<Entity, With<T>>) {
    query
        .iter()
        .for_each(|e| cmds.entity(e).despawn_recursive());
}

#[derive(Component, Deref, DerefMut)]
pub struct DependOnState<T: States>(pub Vec<T>);

impl<T: States> DependOnState<T> {
    pub fn single(state: T) -> Self {
        Self(vec![state])
    }
}

pub fn cleanup_on_state_change<T: States>(
    mut cmds: Commands,
    query: Query<(Entity, &DependOnState<T>)>,
    next_state: Res<NextState<T>>,
) {
    let Some(next_state) = &next_state.0 else {
        return;
    };

    for (entity, on_state) in query.iter() {
        if !on_state.contains(next_state) {
            cmds.entity(entity).despawn_recursive();
        }
    }
}
