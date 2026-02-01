use bevy::ecs::{
    component::Component, entity::Entity, event::EntityEvent, observer::On, system::ResMut,
    world::World,
};

use crate::ui::{UiState, state_management::global::RegisterStateComponent};

#[derive(Debug, Component, Default, Clone)]
pub struct WindowBuffersManager;

impl RegisterStateComponent for WindowBuffersManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(ClearBuffers::observe);
    }
}
#[derive(EntityEvent)]
pub struct ClearBuffers(pub Entity);

impl Default for ClearBuffers {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

impl ClearBuffers {
    pub fn observe(event: On<ClearBuffers>, mut ui_state: ResMut<UiState>) {
        ui_state.buffers.clear_for_window(event.0);
    }
}
