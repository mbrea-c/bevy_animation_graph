use crate::core::animation_graph::{AnimationGraph, InputOverlay, PinMap, TargetPin};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::CacheWriteFilter;
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
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
    fn clone_value(&self) -> Box<dyn NodeLike> {
        Box::new(self.clone())
    }

    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let graph = ctx
            .resources
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();

        if graph.output_time.is_some() {
            let target_pin = TargetPin::OutputTime;
            let duration = graph.get_duration(target_pin, ctx.child(&input_overlay))?;
            ctx.set_duration_fwd(duration);
        } else {
            ctx.set_duration_fwd(None);
        }

        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let graph = ctx
            .resources
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();

        if graph.output_time.is_some() {
            let input = ctx.time_update_fwd();
            if let Ok(time_update) = input {
                let target_pin = TargetPin::OutputTime;
                let mut ctx = ctx.child(&input_overlay);
                let is_temp = ctx.temp_cache;

                ctx.caches_mut().set(
                    |c| c.set_time_update_back(target_pin, time_update),
                    CacheWriteFilter::for_temp(is_temp),
                );
            }
        }

        for id in graph.output_parameters.keys() {
            let target_pin = TargetPin::OutputData(id.clone());
            let value = graph.get_data(target_pin, ctx.child(&input_overlay))?;
            ctx.set_data_fwd(id, value);
        }

        Ok(())
    }

    fn data_input_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph
            .default_parameters
            .iter()
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
        graph.input_times.clone()
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
