use crate::core::animation_graph::{AnimationGraph, InputOverlay, PinId, TargetPin, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::PoseFrame;
use crate::prelude::{OptParamSpec, ParamSpec, ParamValue, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
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
    fn parameter_pass(&self, ctx: PassContext) -> HashMap<PinId, ParamValue> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();
        let mut output = HashMap::new();

        for id in graph.output_parameters.keys() {
            let target_pin = TargetPin::OutputParameter(id.clone());
            let value = graph
                .get_parameter(target_pin, ctx.child(&input_overlay))
                .unwrap();
            output.insert(id.clone(), value);
        }

        output
    }

    fn duration_pass(&self, ctx: PassContext) -> Option<DurationData> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();

        if graph.output_pose {
            let target_pin = TargetPin::OutputPose;
            Some(graph.get_duration(target_pin, ctx.child(&input_overlay)))
        } else {
            None
        }
    }

    fn pose_pass(&self, input: TimeUpdate, ctx: PassContext) -> Option<PoseFrame> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay::default();

        if graph.output_pose {
            let target_pin = TargetPin::OutputPose;
            Some(graph.get_pose(input, target_pin, ctx.child(&input_overlay)))
        } else {
            None
        }
    }

    fn parameter_input_spec(&self, ctx: SpecContext) -> HashMap<PinId, OptParamSpec> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();
        graph.input_parameters.clone()
    }

    fn parameter_output_spec(&self, ctx: SpecContext) -> HashMap<PinId, ParamSpec> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();
        graph.output_parameters.clone()
    }

    fn pose_input_spec(&self, ctx: SpecContext) -> HashSet<PinId> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();
        graph.input_poses.clone()
    }

    fn pose_output_spec(&self, ctx: SpecContext) -> bool {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();
        graph.output_pose
    }

    fn display_name(&self) -> String {
        "󱁉 Graph".into()
    }
}
