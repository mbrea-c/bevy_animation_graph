use crate::core::animation_graph::{AnimationGraph, GraphInputPin, PinMap, TargetPin, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::graph_context::QueryOutputTime;
use crate::core::context::io_env::GraphIoEnv;
use crate::core::context::new_context::{GraphContext, NodeContext};
use crate::core::duration_data::DurationData;
use crate::core::edge_data::{DataSpec, DataValue};
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct GraphNode {
    pub(crate) graph: Handle<AnimationGraph>,
}

impl GraphNode {
    pub fn new(graph: Handle<AnimationGraph>) -> Self {
        Self { graph }
    }
}

impl NodeLike for GraphNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let Some(graph) = ctx
            .graph_context
            .resources
            .animation_graph_assets
            .get(&self.graph)
        else {
            return Ok(());
        };

        let sub_ctx_io = NestedGraphIoEnv {
            parent_ctx: ctx.clone(),
        };

        let sub_ctx = ctx
            .create_child_context(self.graph.id(), None)
            .with_io(&sub_ctx_io);

        if graph.output_time.is_some() {
            let target_pin = TargetPin::OutputTime;
            let duration = graph.get_duration(target_pin, sub_ctx)?;
            ctx.set_duration_fwd(duration);
        } else {
            ctx.set_duration_fwd(None);
        }

        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let Some(graph) = ctx
            .graph_context
            .resources
            .animation_graph_assets
            .get(&self.graph)
        else {
            return Ok(());
        };

        let sub_ctx_io = NestedGraphIoEnv {
            parent_ctx: ctx.clone(),
        };

        let mut sub_ctx = ctx
            .create_child_context(self.graph.id(), None)
            .with_io(&sub_ctx_io);

        if graph.output_time.is_some() {
            let input = ctx.time_update_fwd();
            if let Ok(time_update) = input {
                let key = sub_ctx.state_key;
                sub_ctx.context_mut().query_output_time =
                    QueryOutputTime::from_key(key, time_update);
            }
        }

        for id in graph.output_parameters.keys() {
            let target_pin = TargetPin::OutputData(id.clone());
            let value = graph.get_data(target_pin, sub_ctx.clone())?;
            ctx.set_data_fwd(id, value);
        }

        Ok(())
    }

    fn data_input_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph
            .default_data
            .iter()
            .filter_map(|(k, v)| match k {
                GraphInputPin::Default(pin) => Some((pin, v)),
                _ => None,
            })
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    }

    fn data_output_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph.output_parameters.clone()
    }

    fn time_input_spec(&self, ctx: SpecContext) -> PinMap<()> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph
            .input_times
            .keys()
            .filter_map(|key| match key {
                GraphInputPin::Default(pin) => Some((pin.clone(), ())),
                _ => None,
            })
            .collect()
    }

    fn time_output_spec(&self, ctx: SpecContext) -> Option<()> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph.output_time
    }

    fn display_name(&self) -> String {
        "📈 Graph".into()
    }
}

pub struct NestedGraphIoEnv<'a> {
    pub parent_ctx: NodeContext<'a>,
}

impl<'a> GraphIoEnv for NestedGraphIoEnv<'a> {
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DataValue, GraphError> {
        match pin_id {
            GraphInputPin::Default(pin_id) => self
                .parent_ctx
                .clone()
                .with_state_key(ctx.state_key)
                .data_back(pin_id),
            other => Err(GraphError::InvalidGraphInputPinType(other)),
        }
    }

    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DurationData, GraphError> {
        match pin_id {
            GraphInputPin::Default(pin_id) => self
                .parent_ctx
                .clone()
                .with_state_key(ctx.state_key)
                .duration_back(pin_id),
            other => Err(GraphError::InvalidGraphInputPinType(other)),
        }
    }

    fn get_time_fwd(&self, ctx: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.parent_ctx
            .clone()
            .with_state_key(ctx.state_key)
            .time_update_fwd()
    }
}
