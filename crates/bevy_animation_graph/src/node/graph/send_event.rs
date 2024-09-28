use crate::{
    core::{
        animation_graph::PinMap,
        animation_node::{NodeLike, ReflectNodeLike},
        context::{PassContext, SpecContext},
        edge_data::{AnimationEvent, DataSpec, EventQueue, SampledEvent},
        errors::GraphError,
    },
    utils::unwrap::UnwrapVal,
};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::graph"]
pub struct SendEvent {
    pub event: AnimationEvent,
}

impl SendEvent {
    pub const CONDITION: &'static str = "condition";
    pub const OUT: &'static str = "events";
}

impl NodeLike for SendEvent {
    fn display_name(&self) -> String {
        "Send Event".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let cond: bool = ctx.data_back(Self::CONDITION)?.val();

        if cond {
            ctx.set_data_fwd(
                Self::OUT,
                EventQueue::with_events([SampledEvent::instant(self.event.clone())]),
            );
        } else {
            ctx.set_data_fwd(Self::OUT, EventQueue::with_events([]));
        }

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::CONDITION.into(), DataSpec::Bool)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::EventQueue)].into()
    }
}
