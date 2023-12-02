use crate::core::animation_graph::{
    AnimationGraph, EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::graph_context::{GraphContext, GraphContextTmp};
use bevy::prelude::*;
use bevy::utils::HashMap;

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
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();

        let mut overlay_input_node = graph.nodes.get(AnimationGraph::INPUT_NODE).unwrap().clone();
        overlay_input_node.node.unwrap_input_mut().parameters = inputs;

        let sub_context = context.context_for_subgraph_or_insert_default(name);

        graph.parameter_pass(
            AnimationGraph::OUTPUT_NODE,
            path.clone(),
            sub_context,
            context_tmp,
            &HashMap::from([(AnimationGraph::INPUT_NODE.into(), overlay_input_node)]),
        );

        sub_context
            .get_parameters(AnimationGraph::OUTPUT_NODE)
            .unwrap()
            .upstream
            .clone()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();

        let params = context.get_parameters(name).unwrap().upstream.clone();

        let mut overlay_input_node = graph.nodes.get(AnimationGraph::INPUT_NODE).unwrap().clone();

        overlay_input_node.node.unwrap_input_mut().parameters = params;
        overlay_input_node.node.unwrap_input_mut().durations = inputs;

        let sub_context = context.context_for_subgraph_or_insert_default(name);

        graph.duration_pass(
            AnimationGraph::OUTPUT_NODE,
            path.clone(),
            sub_context,
            context_tmp,
            &HashMap::from([(AnimationGraph::INPUT_NODE.into(), overlay_input_node)]),
        );

        sub_context
            .get_durations(AnimationGraph::OUTPUT_NODE)
            .unwrap()
            .upstream
            .clone()
    }

    fn time_pass(
        &self,
        input: TimeState,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();

        let params = context.get_parameters(name).unwrap().upstream.clone();
        let durations = context.get_durations(name).unwrap().upstream.clone();

        let mut overlay_input_node = graph.nodes.get(AnimationGraph::INPUT_NODE).unwrap().clone();

        overlay_input_node.node.unwrap_input_mut().parameters = params;
        overlay_input_node.node.unwrap_input_mut().durations = durations;

        let sub_context = context.context_for_subgraph_or_insert_default(name);

        graph.time_pass(
            AnimationGraph::OUTPUT_NODE,
            path.clone(),
            input.update,
            sub_context,
            context_tmp,
            &HashMap::from([(AnimationGraph::INPUT_NODE.into(), overlay_input_node)]),
        );

        let update = sub_context
            .get_times(AnimationGraph::INPUT_NODE, path)
            .unwrap()
            .downstream
            .clone()
            .update;

        // TODO: Think whether we want nodes to receive separate time queries per time-dependent
        // output
        graph
            .nodes
            .get(AnimationGraph::INPUT_NODE)
            .unwrap()
            .node
            .unwrap_input()
            .time_dependent_spec
            .iter()
            .map(|(k, _)| (k.clone(), update))
            .collect()
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();

        let params = context.get_parameters(name).unwrap().upstream.clone();
        let durations = context.get_durations(name).unwrap().upstream.clone();

        let mut overlay_input_node = graph.nodes.get(AnimationGraph::INPUT_NODE).unwrap().clone();

        overlay_input_node.node.unwrap_input_mut().parameters = params;
        overlay_input_node.node.unwrap_input_mut().durations = durations;
        overlay_input_node.node.unwrap_input_mut().time_dependent = inputs;

        let sub_context = context.context_for_subgraph_or_insert_default(name);

        graph.time_dependent_pass(
            AnimationGraph::OUTPUT_NODE,
            path.clone(),
            sub_context,
            context_tmp,
            &HashMap::from([(AnimationGraph::INPUT_NODE.into(), overlay_input_node)]),
        );

        sub_context
            .get_time_dependent(AnimationGraph::OUTPUT_NODE, path)
            .unwrap()
            .upstream
            .clone()
    }

    fn parameter_input_spec(
        &self,
        _context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();
        graph
            .nodes
            .get(AnimationGraph::INPUT_NODE)
            .unwrap()
            .node
            .unwrap_input()
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), EdgeSpec::from(v)))
            .collect()
    }

    fn parameter_output_spec(
        &self,
        _context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();
        graph
            .nodes
            .get(AnimationGraph::OUTPUT_NODE)
            .unwrap()
            .node
            .unwrap_output()
            .parameters
            .clone()
    }

    fn time_dependent_input_spec(
        &self,
        _context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();
        graph
            .nodes
            .get(AnimationGraph::INPUT_NODE)
            .unwrap()
            .node
            .unwrap_input()
            .time_dependent_spec
            .clone()
    }

    fn time_dependent_output_spec(
        &self,
        _context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        let graph = context_tmp.animation_graph_assets.get(&self.graph).unwrap();
        graph
            .nodes
            .get(AnimationGraph::OUTPUT_NODE)
            .unwrap()
            .node
            .unwrap_output()
            .time_dependent
            .clone()
    }

    fn display_name(&self) -> String {
        "󱁉 Graph".into()
    }
}
