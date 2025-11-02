use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::core::state_machine::high_level::{StateMachine, TransitionId};

use crate::ui::global_state::{RegisterGlobalState, SetOrInsertEvent, observe_set_or_insert_event};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveFsmTransition {
    pub handle: Handle<StateMachine>,
    pub transition: TransitionId,
}

impl RegisterGlobalState for ActiveFsmTransition {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(
            observe_set_or_insert_event::<ActiveFsmTransition, SetActiveFsmTransition>,
        );
    }
}

#[derive(Event)]
pub struct SetActiveFsmTransition {
    pub new: ActiveFsmTransition,
}

impl SetOrInsertEvent for SetActiveFsmTransition {
    type Target = ActiveFsmTransition;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
