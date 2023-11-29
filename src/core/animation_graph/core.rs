use crate::{
    animation::{HashMapJoinExt, InterpolationMode},
    core::{
        animation_node::{AnimationNode, NodeLike, ParameterNode},
        caches::{DurationCache, ParameterCache, TimeCache, TimeDependentCache},
        frame::PoseFrame,
        graph_context::GraphContext,
        pose::Pose,
    },
    sampling::linear::SampleLinear,
};
use bevy::{prelude::*, reflect::TypeUuid, utils::HashMap};

use super::ToDot;

#[derive(Reflect, Clone, Copy, Debug)]
pub enum EdgeSpec {
    PoseFrame,
    F32,
}

pub type Edge = ((String, String), (String, String));
pub type EdgePath = Vec<Edge>;

#[derive(Reflect, Clone, Debug)]
pub enum EdgeValue {
    PoseFrame(PoseFrame),
    F32(f32),
}

impl EdgeValue {
    pub fn unwrap_pose_frame(self) -> PoseFrame {
        match self {
            Self::PoseFrame(p) => p,
            _ => panic!("Edge value is not a pose frame"),
        }
    }

    pub fn unwrap_f32(self) -> f32 {
        match self {
            Self::F32(f) => f,
            _ => panic!("Edge value is not a f32"),
        }
    }
}

impl From<f32> for EdgeValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<&EdgeValue> for EdgeSpec {
    fn from(value: &EdgeValue) -> Self {
        match value {
            EdgeValue::PoseFrame(_) => Self::PoseFrame,
            EdgeValue::F32(_) => Self::F32,
        }
    }
}

pub type NodeInput = String;
pub type NodeOutput = String;

#[derive(Reflect, Clone, Debug, Copy)]
pub enum TimeUpdate {
    Delta(f32),
    Absolute(f32),
}

impl Default for TimeUpdate {
    fn default() -> Self {
        Self::Absolute(0.)
    }
}

#[derive(Reflect, Clone, Debug, Copy)]
pub struct TimeState {
    pub update: TimeUpdate,
    pub time: f32,
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            update: TimeUpdate::Absolute(0.),
            time: 0.,
        }
    }
}

pub trait UpdateTime<T> {
    fn update(&self, update: T) -> Self;
}

impl UpdateTime<TimeUpdate> for TimeState {
    fn update(&self, update: TimeUpdate) -> Self {
        Self {
            update,
            time: match update {
                TimeUpdate::Delta(dt) => self.time + dt,
                TimeUpdate::Absolute(t) => t,
            },
        }
    }
}

impl UpdateTime<Option<TimeUpdate>> for TimeState {
    fn update(&self, update: Option<TimeUpdate>) -> Self {
        if let Some(update) = update {
            Self {
                update,
                time: match update {
                    TimeUpdate::Delta(dt) => self.time + dt,
                    TimeUpdate::Absolute(t) => t,
                },
            }
        } else {
            *self
        }
    }
}

#[derive(Debug, Clone, Asset, TypeUuid, Reflect)]
#[uuid = "92411396-01ae-4528-9839-709a7a321263"]
pub struct AnimationGraph {
    pub nodes: HashMap<String, AnimationNode>,
    /// Inverted, indexed by output node name.
    pub edges: HashMap<(String, String), (String, String)>,
    pub out_node: String,
    pub out_edge: String,
    pub output_interpolation: InterpolationMode,
}

impl Default for AnimationGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationGraph {
    pub const OUTPUT: &'static str = "Pose";
    const PARAMETER_NODE: &'static str = "__PARAMETERS";

    pub fn new() -> Self {
        Self {
            nodes: HashMap::from([(
                Self::PARAMETER_NODE.into(),
                ParameterNode::default().wrapped(Self::PARAMETER_NODE),
            )]),
            edges: HashMap::new(),
            out_node: "".into(),
            out_edge: "".into(),
            output_interpolation: InterpolationMode::Constant,
        }
    }

    pub fn set_interpolation(&mut self, interpolation: InterpolationMode) {
        self.output_interpolation = interpolation;
    }

    pub fn set_output(&mut self, node: String, edge: String) {
        self.out_node = node;
        self.out_edge = edge;
    }

    pub fn add_node(&mut self, node: AnimationNode) {
        let node_name = node.name.clone();
        if &node_name == Self::PARAMETER_NODE {
            error!("Node name {node_name} is reserved");
            panic!("Node name {node_name} is reserved")
        }
        self.nodes.insert(node_name.clone(), node);
    }

    pub fn set_out_edge(&mut self, node: impl Into<String>, edge: impl Into<String>) {
        self.out_node = node.into();
        self.out_edge = edge.into();
    }

    pub fn set_parameter(&mut self, parameter_name: String, value: EdgeValue) {
        self.nodes
            .get_mut(Self::PARAMETER_NODE)
            .unwrap()
            .node
            .unwrap_parameter_mut()
            .parameters
            .insert(parameter_name, value);
    }

    pub fn get_parameter(&mut self, parameter_name: &str) -> Option<EdgeValue> {
        self.nodes
            .get_mut(Self::PARAMETER_NODE)
            .unwrap()
            .node
            .unwrap_parameter()
            .parameters
            .get(parameter_name)
            .cloned()
    }

    pub fn add_parameter_edge(
        &mut self,
        parameter_name: impl Into<String>,
        target_node: impl Into<String>,
        target_edge: impl Into<String>,
    ) {
        self.add_edge(
            Self::PARAMETER_NODE,
            parameter_name,
            target_node,
            target_edge,
        )
    }

    pub fn add_edge(
        &mut self,
        source_node: impl Into<String>,
        source_edge: impl Into<String>,
        target_node: impl Into<String>,
        target_edge: impl Into<String>,
    ) {
        self.edges.insert(
            (target_node.into(), target_edge.into()),
            (source_node.into(), source_edge.into()),
        );
    }

    pub fn map_upwards_mut<
        S,
        T,
        InputSpecExtractor,
        RecurseSpecExtractor,
        OutputExtractor,
        Mapper,
    >(
        &self,
        node_name: &str,
        path_to_node: EdgePath,
        input_spec_extractor: &InputSpecExtractor,
        recurse_spec_extractor: &RecurseSpecExtractor,
        output_extractor: &OutputExtractor,
        mapper: &Mapper,
        context: &mut GraphContext,
    ) -> HashMap<NodeOutput, T>
    where
        InputSpecExtractor: Fn(&AnimationNode) -> HashMap<NodeInput, S>,
        RecurseSpecExtractor: Fn(&AnimationNode) -> HashMap<NodeInput, S>,
        OutputExtractor: Fn(HashMap<NodeOutput, T>, &str) -> T,
        Mapper: Fn(
            &AnimationNode,
            HashMap<NodeInput, T>,
            &EdgePath,
            &mut GraphContext,
        ) -> HashMap<NodeOutput, T>,
    {
        let in_spec = input_spec_extractor(self.nodes.get(node_name).unwrap());
        let recurse_spec = recurse_spec_extractor(self.nodes.get(node_name).unwrap());

        let mut input: HashMap<NodeOutput, T> = HashMap::new();

        for k in recurse_spec.keys() {
            let (in_node_name, in_edge_name) = self
                .edges
                .get(&(node_name.into(), k.into()))
                .expect(&format!("Missing edge into {node_name}.{k}"))
                .clone();

            // Extend path to input node
            let mut new_path = path_to_node.clone();
            new_path.push((
                (in_node_name.clone(), in_edge_name.clone()),
                (node_name.to_string(), k.clone()),
            ));

            let output = self.map_upwards_mut(
                &in_node_name,
                new_path,
                input_spec_extractor,
                recurse_spec_extractor,
                output_extractor,
                mapper,
                context,
            );
            if in_spec.contains_key(k) {
                let val = output_extractor(output, &in_edge_name);
                input.insert(k.clone(), val);
            }
        }

        let node = self.nodes.get(node_name).unwrap();
        mapper(node, input, &path_to_node, context)
    }

    pub fn map_downwards_mut<Input, S, SpecExtractor, Mapper>(
        &self,
        node_name: &str,
        path_to_node: EdgePath,
        input: Input,
        spec_extractor: &SpecExtractor,
        mapper: &Mapper,
        context: &mut GraphContext,
    ) -> ()
    where
        SpecExtractor: Fn(&AnimationNode) -> HashMap<NodeInput, S>,
        Mapper:
            Fn(&AnimationNode, Input, &EdgePath, &mut GraphContext) -> HashMap<NodeOutput, Input>,
    {
        let node = self.nodes.get(node_name).unwrap();
        let mut output = mapper(node, input, &path_to_node, context);
        let backprop_specs = spec_extractor(node);

        for k in backprop_specs.keys() {
            let (in_node_name, in_edge_name) = self
                .edges
                .get(&(node_name.into(), k.into()))
                .expect(&format!("Missing edge into {node_name}.{k}"))
                .clone();

            // Update path with new edge
            let mut new_path = path_to_node.clone();
            new_path.push((
                (in_node_name.clone(), in_edge_name),
                (node_name.to_string(), k.clone()),
            ));

            self.map_downwards_mut(
                &in_node_name,
                new_path,
                output.remove(k).unwrap(),
                spec_extractor,
                mapper,
                context,
            );
        }
    }

    /// Which inputs are needed to calculate parameter output of this node
    fn parameter_input_spec_extractor(n: &AnimationNode) -> HashMap<NodeInput, EdgeSpec> {
        n.node.map(|n| n.parameter_input_spec())
    }

    /// Which inputs should parameter recalculation be triggered for (superset of input spec)
    fn parameter_recurse_spec_extractor(n: &AnimationNode) -> HashMap<NodeInput, EdgeSpec> {
        let mut spec = n.parameter_input_spec();
        spec.fill_up(&n.time_dependent_input_spec(), &|v| v.clone());
        spec
    }

    /// Computes node output and caches the result for later passes
    fn parameter_mapper(
        n: &AnimationNode,
        inputs: HashMap<NodeInput, EdgeValue>,
        path: &EdgePath,
        context: &mut GraphContext,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let outputs = n.parameter_pass(inputs.clone(), &n.name, path, context);

        context
            .get_node_cache_or_insert_default(&n.name)
            .parameter_cache = Some(ParameterCache {
            upstream: inputs,
            downstream: outputs.clone(),
        });

        outputs
    }

    fn parameter_output_extractor(
        outputs: HashMap<NodeOutput, EdgeValue>,
        edge: &str,
    ) -> EdgeValue {
        outputs.get(edge).unwrap().clone()
    }

    pub fn parameter_pass(&self, node: &str, context: &mut GraphContext) {
        let recurse_spec_extractor = |n: &_| Self::parameter_recurse_spec_extractor(n);
        let input_spec_extractor = |n: &_| Self::parameter_input_spec_extractor(n);
        let mapper = |n: &_, i2, p: &_, c: &mut _| Self::parameter_mapper(n, i2, p, c);
        let output_extractor = |o, e: &_| Self::parameter_output_extractor(o, e);

        self.map_upwards_mut(
            node,
            vec![],
            &input_spec_extractor,
            &recurse_spec_extractor,
            &output_extractor,
            &mapper,
            context,
        );
    }

    /// Which inputs are needed to calculate parameter output of this node
    fn duration_input_spec_extractor(n: &AnimationNode) -> HashMap<NodeInput, EdgeSpec> {
        n.node.map(|n| n.time_dependent_input_spec())
    }

    /// Computes node output and caches the result for later passes
    fn duration_mapper(
        n: &AnimationNode,
        inputs: HashMap<NodeInput, Option<f32>>,
        path: &EdgePath,
        context: &mut GraphContext,
    ) -> HashMap<NodeOutput, Option<f32>> {
        let output = n.duration_pass(inputs.clone(), &n.name, path, context);
        context
            .get_node_cache_or_insert_default(&n.name)
            .duration_cache = Some(DurationCache {
            upstream: inputs,
            downstream: output,
        });

        HashMap::from([(String::from(""), output)])
    }

    fn duration_output_extractor(
        outputs: HashMap<NodeOutput, Option<f32>>,
        _edge: &str,
    ) -> Option<f32> {
        outputs.get("").unwrap().clone()
    }

    pub fn duration_pass(&self, node: &str, context: &mut GraphContext) {
        let spec_extractor = |n: &_| Self::duration_input_spec_extractor(n);
        let mapper = |a: &_, c, d: &_, e: &mut _| Self::duration_mapper(a, c, d, e);
        let output_extractor = |o, e: &_| Self::duration_output_extractor(o, e);

        self.map_upwards_mut(
            node,
            vec![],
            &spec_extractor,
            &spec_extractor,
            &output_extractor,
            &mapper,
            context,
        );
    }

    fn time_spec_extractor(n: &AnimationNode) -> HashMap<NodeInput, EdgeSpec> {
        n.time_dependent_input_spec()
    }

    fn time_mapper(
        n: &AnimationNode,
        input: TimeUpdate,
        path: &EdgePath,
        context: &mut GraphContext,
    ) -> HashMap<NodeInput, TimeUpdate> {
        let input_state = {
            let last_time_state = context
                .get_other_times(&n.name, path)
                .map_or(TimeState::default(), |c| c.downstream);
            last_time_state.update(input)
        };
        let output = n.time_pass(input_state, &n.name, path, context);
        context
            .get_node_cache_or_insert_default(&n.name)
            .time_caches
            .insert(
                path.clone(),
                TimeCache {
                    downstream: input_state,
                    upstream: output.clone(),
                },
            );

        output
    }

    pub fn time_pass(&self, node: &str, time_update: TimeUpdate, context: &mut GraphContext) {
        let spec_extractor = |n: &_| Self::time_spec_extractor(n);
        let mapper = |n: &_, i, p: &_, c: &mut _| Self::time_mapper(n, i, p, c);

        self.map_downwards_mut(node, vec![], time_update, &spec_extractor, &mapper, context);
    }

    /// Which inputs are needed to calculate time-dependent output of this node
    fn time_dependent_input_spec_extractor(n: &AnimationNode) -> HashMap<NodeInput, EdgeSpec> {
        n.time_dependent_input_spec()
    }

    /// Computes node output and caches the result for later passes
    fn time_dependent_mapper(
        n: &AnimationNode,
        inputs: HashMap<NodeInput, EdgeValue>,
        path: &EdgePath,
        context: &mut GraphContext,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let outputs = n.time_dependent_pass(inputs.clone(), &n.name, path, context);

        context
            .get_node_cache_or_insert_default(&n.name)
            .time_dependent_caches
            .insert(
                path.clone(),
                TimeDependentCache {
                    upstream: inputs,
                    downstream: outputs.clone(),
                },
            );

        outputs
    }

    fn time_dependent_output_extractor(
        outputs: HashMap<NodeOutput, EdgeValue>,
        edge: &str,
    ) -> EdgeValue {
        outputs.get(edge).unwrap().clone()
    }

    pub fn time_dependent_pass(
        &self,
        node: &str,
        context: &mut GraphContext,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let spec_extractor = |n: &_| Self::time_dependent_input_spec_extractor(n);
        let output_extractor = |o, e: &_| Self::time_dependent_output_extractor(o, e);
        let mapper = |n: &_, i, p: &_, c: &mut _| Self::time_dependent_mapper(n, i, p, c);

        self.map_upwards_mut(
            node,
            vec![],
            &spec_extractor,
            &spec_extractor,
            &output_extractor,
            &mapper,
            context,
        )
    }

    pub fn query(&self, time_update: TimeUpdate, context: &mut GraphContext) -> Pose {
        context.push_caches();
        let out_node = &self.out_node.clone();
        self.parameter_pass(out_node, context);
        self.duration_pass(out_node, context);
        self.time_pass(out_node, time_update, context);
        let mut final_output = self.time_dependent_pass(out_node, context);

        // self.dot_to_tmp_file(Some(context)).unwrap();

        let final_frame = final_output
            .remove(&self.out_edge)
            .unwrap()
            .unwrap_pose_frame();

        final_frame.sample_linear()
    }
}
