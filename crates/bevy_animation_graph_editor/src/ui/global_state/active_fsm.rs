use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::core::state_machine::high_level::StateMachine;

use crate::ui::global_state::{
    RegisterGlobalState, SetOrInsertEvent, observe_clear_global_state, observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveFsm {
    pub handle: Handle<StateMachine>,
}

impl RegisterGlobalState for ActiveFsm {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveFsm, SetActiveFsm>);
        world.add_observer(observe_clear_global_state::<Self>);
    }
}

#[derive(Event)]
pub struct SetActiveFsm {
    pub new: ActiveFsm,
}

impl SetOrInsertEvent for SetActiveFsm {
    type Target = ActiveFsm;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
