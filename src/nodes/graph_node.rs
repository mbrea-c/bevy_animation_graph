use crate::core::animation_graph::{
    AnimationGraph, InputOverlay, OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{DurationData, PassContext, SpecContext};
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
    fn parameter_pass(
        &self,
        inputs: HashMap<PinId, ParamValue>,
        ctx: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay {
            parameters: inputs,
            ..default()
        };

        let sub_context = ctx
            .context
            .context_for_subgraph_or_insert_default(ctx.node_id);

        graph.parameter_pass(sub_context, ctx.context_tmp, &input_overlay)
    }

    fn duration_pass(
        &self,
        inputs: HashMap<PinId, DurationData>,
        ctx: PassContext,
    ) -> Option<DurationData> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay {
            durations: inputs,
            // We do not add the parameters because they should already have been cached!
            // in the subgraph context
            ..default()
        };

        let sub_context = ctx
            .context
            .context_for_subgraph_or_insert_default(ctx.node_id);

        graph.duration_pass(sub_context, ctx.context_tmp, &input_overlay)
    }

    fn time_pass(&self, input: TimeState, ctx: PassContext) -> HashMap<PinId, TimeUpdate> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        // We do not add the parameters and durations because they should already have
        // been cached in the subgraph context
        let input_overlay = InputOverlay::default();

        let sub_context = ctx
            .context
            .context_for_subgraph_or_insert_default(ctx.node_id);

        graph.time_pass(input.update, sub_context, ctx.context_tmp, &input_overlay)
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame> {
        let graph = ctx
            .context_tmp
            .animation_graph_assets
            .get(&self.graph)
            .unwrap();

        let input_overlay = InputOverlay {
            time_dependent: inputs,
            // We do not add the parameters and durations because they should already have
            // been cached in the subgraph context
            ..default()
        };

        let sub_context = ctx
            .context
            .context_for_subgraph_or_insert_default(ctx.node_id);

        graph.time_dependent_pass(sub_context, ctx.context_tmp, &input_overlay)
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
