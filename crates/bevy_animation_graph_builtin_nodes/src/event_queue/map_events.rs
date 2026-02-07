use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{
        DataSpec,
        events::{AnimationEvent, EventQueue},
    },
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct MapEventsNode {
    map_from: AnimationEvent,
    map_to: AnimationEvent,
}

impl MapEventsNode {
    pub const EVENT_IN: &'static str = "events";
    pub const EVENT_OUT: &'static str = "events";
}

impl NodeLike for MapEventsNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let in_events = ctx.data_back(Self::EVENT_IN)?.into_event_queue()?;

        let out_events = EventQueue::with_events(
            in_events
                .events
                .into_iter()
                .filter_map(|mut event| {
                    if event.event == self.map_from {
                        event.event = self.map_to.clone();
                        Some(event)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        );

        ctx.set_data_fwd(Self::EVENT_OUT, out_events);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::EVENT_IN, DataSpec::EventQueue);
        ctx.add_output_data(Self::EVENT_OUT, DataSpec::EventQueue);

        Ok(())
    }

    fn display_name(&self) -> String {
        "Map events".into()
    }
}
