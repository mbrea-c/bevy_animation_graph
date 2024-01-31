use crate::core::animation_graph::{
    AnimationGraph, InputOverlay, PinId, PinMap, TargetPin, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::frame::{PoseFrame, PoseSpec};
use crate::prelude::{OptParamSpec, ParamSpec, ParamValue, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct GraphNode {
    pub(crate) graph: Handle<AnimationGraph>,
}

impl GraphNode {
    pub fn new(graph: Handle<AnimationGraph>) -> Self {
        Self { graph }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Graph(self))
    }
}

impl NodeLike for GraphNode {
    fn parameter_pass(&self, ctx: PassContext) -> Result<HashMap<PinId, ParamValue>, GraphError> {
        let graph = ctx
            .resources
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();
        let mut output = HashMap::new();

        for id in graph.output_parameters.keys() {
            let target_pin = TargetPin::OutputParameter(id.clone());
            let value = graph.get_parameter(target_pin, ctx.child(&input_overlay))?;
            output.insert(id.clone(), value);
        }

        Ok(output)
    }

    fn duration_pass(&self, ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        let graph = ctx
            .resources
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();

        if graph.output_pose.is_some() {
            let target_pin = TargetPin::OutputPose;
            Ok(Some(
                graph.get_duration(target_pin, ctx.child(&input_overlay))?,
            ))
        } else {
            Ok(None)
        }
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        ctx: PassContext,
    ) -> Result<Option<PoseFrame>, GraphError> {
        let graph = ctx
            .resources
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();

        if graph.output_pose.is_some() {
            let target_pin = TargetPin::OutputPose;
            Ok(Some(graph.get_pose(
                input,
                target_pin,
                ctx.child(&input_overlay),
            )?))
        } else {
            Ok(None)
        }
    }

    fn parameter_input_spec(&self, ctx: SpecContext) -> PinMap<OptParamSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph
            .default_parameters
            .iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    }

    fn parameter_output_spec(&self, ctx: SpecContext) -> PinMap<ParamSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph.output_parameters.clone()
    }

    fn pose_input_spec(&self, ctx: SpecContext) -> PinMap<PoseSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph.input_poses.clone()
    }

    fn pose_output_spec(&self, ctx: SpecContext) -> Option<PoseSpec> {
        let Some(graph) = ctx.graph_assets.get(&self.graph) else {
            return Default::default();
        };
        graph.output_pose
    }

    fn display_name(&self) -> String {
        "📈 Graph".into()
    }
}
