use bevy::ecs::{component::Component, entity::Entity, query::With, world::World};

use crate::ui::native_views::EditorViewState;

pub fn get_entity_state<T: Component, M: Component>(world: &World, entity: Entity) -> Option<&T> {
    let mut query_state = world.try_query_filtered::<&T, With<M>>()?;
    query_state.get(world, entity).ok()
}

pub fn get_view_state<T: Component>(world: &World, view: Entity) -> Option<&T> {
    get_entity_state::<T, EditorViewState>(world, view)
}
