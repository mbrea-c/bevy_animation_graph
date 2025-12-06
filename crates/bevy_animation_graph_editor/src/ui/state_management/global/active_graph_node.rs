use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, event::Event, world::World},
};
use bevy_animation_graph::core::animation_graph::{AnimationGraph, NodeId, PinId};

use crate::ui::state_management::global::{
    RegisterStateComponent, SetOrInsertEvent, observe_set_or_insert_event,
};

#[derive(Debug, Component, Clone)]
pub struct ActiveGraphNode {
    pub handle: Handle<AnimationGraph>,
    pub node: NodeId,
    pub selected_pin: Option<PinId>,
}

impl RegisterStateComponent for ActiveGraphNode {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveGraphNode, SetActiveGraphNode>);
    }
}

#[derive(Event)]
pub struct SetActiveGraphNode {
    pub new: ActiveGraphNode,
}

impl SetOrInsertEvent for SetActiveGraphNode {
    type Target = ActiveGraphNode;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}
