use bevy::ecs::{component::Component, entity::Entity, event::Event, world::World};

use crate::ui::state_management::global::{
    RegisterStateComponent, SetOrInsertEvent, observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone, Hash)]
pub enum InspectorSelection {
    ActiveFsm,
    ActiveFsmTransition,
    ActiveFsmState,

    ActiveGraph,
    ActiveNode,

    #[default]
    Nothing,
}

impl RegisterStateComponent for InspectorSelection {
    fn register(world: &mut World, global_state_entity: Entity) {
        world
            .entity_mut(global_state_entity)
            .insert(InspectorSelection::default());

        world
            .add_observer(observe_set_or_insert_event::<InspectorSelection, SetInspectorSelection>);
    }
}

#[derive(Event)]
pub struct SetInspectorSelection {
    pub selection: InspectorSelection,
}

impl SetOrInsertEvent for SetInspectorSelection {
    type Target = InspectorSelection;

    fn get_component(&self) -> Self::Target {
        self.selection.clone()
    }
}
