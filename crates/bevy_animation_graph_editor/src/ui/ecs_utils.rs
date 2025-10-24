use bevy::ecs::{
    component::{Component, Mutable},
    entity::Entity,
    query::With,
    world::World,
};

use crate::ui::{native_views::EditorViewState, native_windows::WindowState};

pub fn get_entity_state<T: Component, M: Component>(world: &World, entity: Entity) -> Option<&T> {
    let mut query_state = world.try_query_filtered::<&T, With<M>>()?;
    query_state.get(world, entity).ok()
}

pub fn set_entity_state<T: Component<Mutability = Mutable>, M: Component>(
    world: &mut World,
    entity: Entity,
    value: T,
) -> Option<()> {
    let mut query_state = world.query_filtered::<&mut T, With<M>>();
    let mut component = query_state.get_mut(world, entity).ok()?;
    *component = value;
    Some(())
}

pub fn mutate_entity_state<T: Component<Mutability = Mutable>, M: Component>(
    world: &mut World,
    entity: Entity,
    f: impl FnOnce(&mut T) -> T,
) -> Option<T> {
    let mut query_state = world.query_filtered::<&mut T, With<M>>();
    let component = query_state.get_mut(world, entity).ok()?;
    Some(f(component.into_inner()))
}

pub fn get_view_state<T: Component>(world: &World, view: Entity) -> Option<&T> {
    get_entity_state::<T, EditorViewState>(world, view)
}

pub fn get_window_state<T: Component>(world: &World, window: Entity) -> Option<&T> {
    get_entity_state::<T, WindowState>(world, window)
}
