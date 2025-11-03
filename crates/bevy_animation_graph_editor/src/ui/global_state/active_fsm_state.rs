use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::core::state_machine::high_level::{StateId, StateMachine};

use crate::ui::global_state::{
    RegisterStateComponent, SetOrInsertEvent, observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveFsmState {
    pub handle: Handle<StateMachine>,
    pub state: StateId,
}

impl RegisterStateComponent for ActiveFsmState {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveFsmState, SetActiveFsmState>);
    }
}

#[derive(Event)]
pub struct SetActiveFsmState {
    pub new: ActiveFsmState,
}

impl SetOrInsertEvent for SetActiveFsmState {
    type Target = ActiveFsmState;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
