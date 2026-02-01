use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::core::skeleton::Skeleton;

use crate::ui::state_management::global::{
    RegisterStateComponent, SetOrInsertEvent, observe_clear_global_state,
    observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveSkeleton {
    pub handle: Handle<Skeleton>,
}

impl RegisterStateComponent for ActiveSkeleton {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveSkeleton, SetActiveSkeleton>);
        world.add_observer(observe_clear_global_state::<Self>);
    }
}

#[derive(Event)]
pub struct SetActiveSkeleton {
    pub new: ActiveSkeleton,
}

impl SetOrInsertEvent for SetActiveSkeleton {
    type Target = ActiveSkeleton;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
