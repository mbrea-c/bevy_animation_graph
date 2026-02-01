use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{
        DataSpec,
        events::{AnimationEvent, EventQueue, SampledEvent},
    },
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct FireEventNode {
    pub event: AnimationEvent,
}

impl FireEventNode {
    pub const EVENT_OUT: &'static str = "event";
    pub const CONDITION_IN: &'static str = "condition";

    pub fn new(event: AnimationEvent) -> Self {
        Self { event }
    }
}

impl NodeLike for FireEventNode {
    fn display_name(&self) -> String {
        "FireEvent".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let cond: bool = ctx.data_back(Self::CONDITION_IN)?.into_bool()?;

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

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::CONDITION_IN, DataSpec::Bool);
        ctx.add_output_data(Self::EVENT_OUT, DataSpec::EventQueue);

        Ok(())
    }
}
