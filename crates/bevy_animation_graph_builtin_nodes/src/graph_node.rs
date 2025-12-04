use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::{AnimationGraph, GraphInputPin, TargetPin, TimeUpdate},
    animation_node::{NodeLike, ReflectNodeLike},
    context::{
        graph_context::QueryOutputTime,
        io_env::GraphIoEnv,
        new_context::{GraphContext, NodeContext},
        spec_context::{NodeInput, NodeOutput, SpecContext},
    },
    duration_data::DurationData,
    edge_data::DataValue,
    errors::GraphError,
};

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

        if graph.io_spec.has_output_time() {
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

        if graph.io_spec.has_output_time() {
            let input = ctx.time_update_fwd();
            if let Ok(time_update) = input {
                let key = sub_ctx.state_key;
                sub_ctx.context_mut().query_output_time =
                    QueryOutputTime::from_key(key, time_update);
            }
        }

        for (id, _) in graph.io_spec.iter_output_data() {
            let target_pin = TargetPin::OutputData(id.clone());
            let value = graph.get_data(target_pin, sub_ctx.clone())?;
            ctx.set_data_fwd(id, value);
        }

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        let graph = ctx
            .resources()
            .graph_assets
            .get(&self.graph)
            .ok_or(GraphError::GraphAssetMissing)?;
        for input in graph.io_spec.sorted_inputs() {
            match input {
                NodeInput::Time(GraphInputPin::Default(pin_id)) => {
                    ctx.add_input_time(pin_id);
                }
                NodeInput::Data(GraphInputPin::Default(pin_id), data_spec) => {
                    ctx.add_input_data(pin_id, data_spec);
                }
                _ => {}
            }
        }

        for output in graph.io_spec.sorted_outputs() {
            match output {
                NodeOutput::Time => {
                    ctx.add_output_time();
                }
                NodeOutput::Data(pin_id, data_spec) => {
                    ctx.add_output_data(pin_id, data_spec);
                }
            }
        }

        Ok(())
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
