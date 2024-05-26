use crate::{
    core::{
        animation_graph::PinMap,
        animation_node::{AnimationNode, AnimationNodeType, CustomNode, NodeLike},
        context::{PassContext, SpecContext},
        edge_data::{AnimationEvent, DataSpec, EventQueue, SampledEvent},
        errors::GraphError,
    },
    utils::unwrap::UnwrapVal,
};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct FireEventNode {
    pub event: AnimationEvent,
}

impl FireEventNode {
    pub const EVENT_OUT: &'static str = "event";
    pub const CONDITION_IN: &'static str = "condition";

    pub fn new(event: AnimationEvent) -> Self {
        Self { event }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::FireEvent(self))
    }
}

impl NodeLike for FireEventNode {
    fn display_name(&self) -> String {
        "FireEvent".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let cond: bool = ctx.data_back(Self::CONDITION_IN)?.val();

        if cond {
            ctx.set_data_fwd(
                Self::EVENT_OUT,
                EventQueue::with_events([SampledEvent::instant(self.event.clone())]),
            );
        } else {
            ctx.set_data_fwd(Self::EVENT_OUT, EventQueue::with_events([]));
        }

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::CONDITION_IN.into(), DataSpec::Bool)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::EVENT_OUT.into(), DataSpec::EventQueue)].into()
    }
}
