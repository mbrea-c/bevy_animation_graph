use bevy::{
    asset::Handle,
    ecs::{component::Component, event::Event, observer::Trigger, query::With, system::Single},
};
use bevy_animation_graph::{
    core::animation_graph::{NodeId, PinId},
    prelude::AnimationGraph,
};

use crate::ui::global_state::GlobalState;

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveGraphNode {
    pub handle: Handle<AnimationGraph>,
    pub node: NodeId,
    pub selected_pin: Option<PinId>,
}

#[derive(Event)]
pub struct SetActiveGraphNode {
    pub new: ActiveGraphNode,
}

impl SetActiveGraphNode {
    pub fn observe(
        event: Trigger<SetActiveGraphNode>,
        global_state: Single<&mut ActiveGraphNode, With<GlobalState>>,
    ) {
        *global_state.into_inner() = event.event().new.clone();
    }
}
