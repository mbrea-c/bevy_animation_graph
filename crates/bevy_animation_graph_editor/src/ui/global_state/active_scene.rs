use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::prelude::AnimatedScene;

use crate::ui::global_state::{
    RegisterStateComponent, SetOrInsertEvent, observe_clear_global_state,
    observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveScene {
    pub handle: Handle<AnimatedScene>,
}

impl RegisterStateComponent for ActiveScene {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveScene, SetActiveScene>);
        world.add_observer(observe_clear_global_state::<Self>);
    }
}

#[derive(Event)]
pub struct SetActiveScene {
    pub new: ActiveScene,
}

impl SetOrInsertEvent for SetActiveScene {
    type Target = ActiveScene;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
