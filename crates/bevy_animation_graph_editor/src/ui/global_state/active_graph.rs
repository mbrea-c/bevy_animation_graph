use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::prelude::AnimationGraph;

use crate::ui::global_state::{RegisterGlobalState, SetOrInsertEvent, observe_set_or_insert_event};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveGraph {
    pub handle: Handle<AnimationGraph>,
}

impl RegisterGlobalState for ActiveGraph {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveGraph, SetActiveGraph>);
    }
}

#[derive(Event)]
pub struct SetActiveGraph {
    pub new: ActiveGraph,
}

impl SetOrInsertEvent for SetActiveGraph {
    type Target = ActiveGraph;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
