use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::core::ragdoll::definition::Ragdoll;

use crate::ui::global_state::{
    RegisterStateComponent, SetOrInsertEvent, observe_clear_global_state,
    observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveRagdoll {
    pub handle: Handle<Ragdoll>,
}

impl RegisterStateComponent for ActiveRagdoll {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveRagdoll, SetActiveRagdoll>);
        world.add_observer(observe_clear_global_state::<Self>);
    }
}

#[derive(Event)]
pub struct SetActiveRagdoll {
    pub new: ActiveRagdoll,
}

impl SetOrInsertEvent for SetActiveRagdoll {
    type Target = ActiveRagdoll;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
