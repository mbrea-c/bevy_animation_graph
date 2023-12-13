use crate::{
    core::{
        animation_node::{AnimationNode, GraphInputNode, GraphOutputNode, NodeLike},
        caches::{DurationCache, ParameterCache, TimeCache, TimeDependentCache},
        frame::PoseFrame,
        graph_context::{GraphContext, GraphContextTmp},
        pose::Pose,
    },
    sampling::linear::SampleLinear,
    utils::hash_map_join::HashMapJoinExt,
};
use bevy::{prelude::*, reflect::TypeUuid, utils::HashMap};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EdgeSpec {
    PoseFrame,
    F32,
}

pub type Edge = ((String, String), (String, String));
pub type EdgePath = Vec<Edge>;

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub enum EdgeValue {
    PoseFrame(#[serde(skip)] PoseFrame),
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
    pub node_edges: HashMap<(String, String), (String, String)>,
    pub default_output: Option<String>,
}

impl Default for AnimationGraph {
    fn default() -> Self {
        Self::new()
    }
}

type SpecExtractor<S> =
    fn(&AnimationNode, &mut GraphContext, &mut GraphContextTmp) -> HashMap<NodeInput, S>;
type PrepareInput<In, Out> = fn(&Out, &str) -> In;
type Mapper<In, Out> =
    fn(&AnimationNode, In, &EdgePath, &mut GraphContext, &mut GraphContextTmp) -> Out;
type ShortCircuit<Out> =
    fn(&AnimationNode, &EdgePath, &mut GraphContext, &mut GraphContextTmp) -> Option<Out>;

struct UpFns<In, Out> {
    pub prepare: PrepareInput<In, Out>,
    pub mapper: Mapper<HashMap<NodeInput, In>, Out>,
}

struct DownFns<In, Out> {
    pub prepare: PrepareInput<In, Out>,
    pub mapper: Mapper<In, Out>,
}

impl<I, O> Clone for UpFns<I, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, O> Copy for UpFns<I, O> {}

impl<I, O> Clone for DownFns<I, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, O> Copy for DownFns<I, O> {}

impl AnimationGraph {
    pub const INPUT_NODE: &'static str = "__INPUT";
    pub const OUTPUT_NODE: &'static str = "__OUTPUT";

    pub fn new() -> Self {
        Self {
            nodes: HashMap::from([
                (
                    Self::INPUT_NODE.into(),
                    GraphInputNode::default().wrapped(Self::INPUT_NODE),
                ),
                (
                    Self::OUTPUT_NODE.into(),
                    GraphOutputNode::default().wrapped(Self::OUTPUT_NODE),
                ),
            ]),
            node_edges: HashMap::new(),
            default_output: None,
        }
    }

    pub fn add_node(&mut self, node: AnimationNode) {
        let node_name = node.name.clone();
        if node_name == Self::INPUT_NODE || node_name == Self::OUTPUT_NODE {
            error!("Node name {node_name} is reserved");
            panic!("Node name {node_name} is reserved")
        }
        self.nodes.insert(node_name.clone(), node);
    }

    pub fn set_default_output(&mut self, name: impl Into<String>) {
        self.default_output = Some(name.into());
    }

    pub fn set_input_parameter(&mut self, parameter_name: impl Into<String>, value: EdgeValue) {
        self.nodes
            .get_mut(Self::INPUT_NODE)
            .unwrap()
            .node
            .unwrap_input_mut()
            .parameters
            .insert(parameter_name.into(), value);
    }

    pub fn get_input_parameter(&mut self, parameter_name: &str) -> Option<EdgeValue> {
        self.nodes
            .get_mut(Self::INPUT_NODE)
            .unwrap()
            .node
            .unwrap_input()
            .parameters
            .get(parameter_name)
            .cloned()
    }

    pub fn register_input_td(&mut self, input_name: impl Into<String>, spec: EdgeSpec) {
        self.nodes
            .get_mut(Self::INPUT_NODE)
            .unwrap()
            .node
            .unwrap_input_mut()
            .time_dependent_spec
            .insert(input_name.into(), spec);
    }

    pub fn register_output_parameter(&mut self, input_name: impl Into<String>, spec: EdgeSpec) {
        self.nodes
            .get_mut(Self::OUTPUT_NODE)
            .unwrap()
            .node
            .unwrap_output_mut()
            .parameters
            .insert(input_name.into(), spec);
    }

    pub fn register_output_td(&mut self, input_name: impl Into<String>, spec: EdgeSpec) {
        self.nodes
            .get_mut(Self::OUTPUT_NODE)
            .unwrap()
            .node
            .unwrap_output_mut()
            .time_dependent
            .insert(input_name.into(), spec);
    }

    pub fn add_input_edge(
        &mut self,
        parameter_name: impl Into<String>,
        target_node: impl Into<String>,
        target_edge: impl Into<String>,
    ) {
        self.add_edge(Self::INPUT_NODE, parameter_name, target_node, target_edge)
    }

    pub fn add_output_edge(
        &mut self,
        source_node: impl Into<String>,
        source_edge: impl Into<String>,
        output_name: impl Into<String>,
    ) {
        self.add_edge(source_node, source_edge, Self::OUTPUT_NODE, output_name)
    }

    pub fn add_edge(
        &mut self,
        source_node: impl Into<String>,
        source_edge: impl Into<String>,
        target_node: impl Into<String>,
        target_edge: impl Into<String>,
    ) {
        self.node_edges.insert(
            (target_node.into(), target_edge.into()),
            (source_node.into(), source_edge.into()),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn map<SpecType, InputUp: Clone, OutputUp: Default, InputDown, OutputDown>(
        &self,
        node_name: &str,
        path_to_node: EdgePath,
        input_spec_extractor: SpecExtractor<SpecType>,
        recurse_spec_extractor: SpecExtractor<SpecType>,
        short_circuiter: ShortCircuit<OutputUp>,
        up: Option<UpFns<InputUp, OutputUp>>,
        down: Option<DownFns<InputDown, OutputDown>>,
        down_input: Option<InputDown>,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) -> OutputUp {
        let node = overlay
            .get(node_name)
            .or_else(|| self.nodes.get(node_name))
            .unwrap();

        if let Some(out) = short_circuiter(node, &path_to_node, context, context_tmp) {
            return out;
        }

        let in_spec =
            input_spec_extractor(self.nodes.get(node_name).unwrap(), context, context_tmp);
        let recurse_spec =
            recurse_spec_extractor(self.nodes.get(node_name).unwrap(), context, context_tmp);

        let output_down = down.map(|down| {
            (down.mapper)(
                node,
                down_input.expect("Have down fns but missing down input"),
                &path_to_node,
                context,
                context_tmp,
            )
        });

        let mut input_up = up.map(|_| HashMap::new());

        for k in recurse_spec.keys() {
            let Some((in_node_name, in_edge_name)) =
                self.node_edges.get(&(node_name.into(), k.into()))
            else {
                continue;
            };

            // Extend path to input node
            let mut new_path = path_to_node.clone();
            new_path.push((
                (in_node_name.clone(), in_edge_name.clone()),
                (node_name.to_string(), k.clone()),
            ));

            let new_down_input = down.map(|down| (down.prepare)(output_down.as_ref().unwrap(), k));

            let output_up = self.map(
                in_node_name,
                new_path,
                input_spec_extractor,
                recurse_spec_extractor,
                short_circuiter,
                up,
                down,
                new_down_input,
                context,
                context_tmp,
                overlay,
            );

            if let Some(up) = up {
                if in_spec.contains_key(k) {
                    let val = (up.prepare)(&output_up, in_edge_name);
                    input_up.as_mut().unwrap().insert(k.clone(), val);
                }
            }
        }

        if let Some(up) = up {
            (up.mapper)(node, input_up.unwrap(), &path_to_node, context, context_tmp)
        } else {
            OutputUp::default()
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn map_up<SpecType, InputUp: Clone, OutputUp: Default>(
        &self,
        node_name: &str,
        path_to_node: EdgePath,
        input_spec_extractor: SpecExtractor<SpecType>,
        recurse_spec_extractor: SpecExtractor<SpecType>,
        short_circuiter: ShortCircuit<OutputUp>,
        up: UpFns<InputUp, OutputUp>,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) -> OutputUp {
        self.map::<SpecType, InputUp, OutputUp, (), ()>(
            node_name,
            path_to_node,
            input_spec_extractor,
            recurse_spec_extractor,
            short_circuiter,
            Some(up),
            None,
            None,
            context,
            context_tmp,
            overlay,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn map_down<SpecType, InputDown, OutputDown: Default>(
        &self,
        node_name: &str,
        path_to_node: EdgePath,
        input_spec_extractor: SpecExtractor<SpecType>,
        recurse_spec_extractor: SpecExtractor<SpecType>,
        short_circuiter: ShortCircuit<()>,
        down: DownFns<InputDown, OutputDown>,
        down_input: InputDown,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) {
        self.map::<SpecType, (), (), InputDown, OutputDown>(
            node_name,
            path_to_node,
            input_spec_extractor,
            recurse_spec_extractor,
            short_circuiter,
            None,
            Some(down),
            Some(down_input),
            context,
            context_tmp,
            overlay,
        )
    }

    fn prepare_input_index_hashmap<T: Clone>(outputs: &HashMap<NodeOutput, T>, edge: &str) -> T {
        outputs
            .get(edge)
            .unwrap_or_else(|| panic!("Edge output {} not found!", edge))
            .clone()
    }

    /// Which inputs are needed to calculate parameter output of this node
    fn parameter_input_spec_extractor(
        n: &AnimationNode,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        n.parameter_input_spec(context, context_tmp)
    }

    /// Which inputs should parameter recalculation be triggered for (superset of input spec)
    fn parameter_recurse_spec_extractor(
        n: &AnimationNode,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        let mut spec = n.parameter_input_spec(context, context_tmp);
        spec.fill_up(&n.time_dependent_input_spec(context, context_tmp), &|v| *v);
        spec
    }

    /// Computes node output and caches the result for later passes
    fn parameter_mapper(
        n: &AnimationNode,
        inputs: HashMap<NodeInput, EdgeValue>,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let outputs = n.parameter_pass(inputs.clone(), &n.name, path, context, context_tmp);

        context
            .get_node_cache_or_insert_default(&n.name)
            .parameter_cache = Some(ParameterCache {
            upstream: inputs,
            downstream: outputs.clone(),
        });

        outputs
    }

    fn short_circuit_parameter(
        n: &AnimationNode,
        _path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> Option<HashMap<NodeOutput, EdgeValue>> {
        context
            .get_parameters(&n.name)
            .map(|c| c.downstream.clone())
    }

    fn short_circuit_durations(
        n: &AnimationNode,
        _path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> Option<HashMap<NodeOutput, Option<f32>>> {
        context.get_durations(&n.name).map(|c| c.downstream.clone())
    }

    fn short_circuit_times(
        n: &AnimationNode,
        path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> Option<()> {
        context.get_times(&n.name, path).map(|_| ())
    }

    fn short_circuit_td(
        n: &AnimationNode,
        path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> Option<HashMap<NodeOutput, EdgeValue>> {
        context
            .get_time_dependent(&n.name, path)
            .map(|c| c.downstream.clone())
    }

    pub fn parameter_pass(
        &self,
        node: &str,
        path: EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) {
        self.map_up(
            node,
            path,
            Self::parameter_input_spec_extractor,
            Self::parameter_recurse_spec_extractor,
            Self::short_circuit_parameter,
            UpFns {
                prepare: Self::prepare_input_index_hashmap,
                mapper: Self::parameter_mapper,
            },
            context,
            context_tmp,
            overlay,
        );
    }

    /// Which inputs are needed to calculate parameter output of this node
    fn duration_input_spec_extractor(
        n: &AnimationNode,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        n.node
            .map(|n| n.time_dependent_input_spec(context, context_tmp))
    }

    /// Computes node output and caches the result for later passes
    fn duration_mapper(
        n: &AnimationNode,
        inputs: HashMap<NodeInput, Option<f32>>,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        let output = n.duration_pass(inputs.clone(), &n.name, path, context, context_tmp);
        context
            .get_node_cache_or_insert_default(&n.name)
            .duration_cache = Some(DurationCache {
            upstream: inputs,
            downstream: output.clone(),
        });

        output
    }

    pub fn duration_pass(
        &self,
        node: &str,
        path: EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) {
        self.map_up(
            node,
            path,
            Self::duration_input_spec_extractor,
            Self::duration_input_spec_extractor,
            Self::short_circuit_durations,
            UpFns {
                prepare: Self::prepare_input_index_hashmap,
                mapper: Self::duration_mapper,
            },
            context,
            context_tmp,
            overlay,
        );
    }

    fn time_spec_extractor(
        n: &AnimationNode,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        n.time_dependent_input_spec(context, context_tmp)
    }

    fn time_mapper(
        n: &AnimationNode,
        input: TimeUpdate,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        let input_state = {
            let last_time_state = context
                .get_other_times(&n.name, path)
                .map_or(TimeState::default(), |c| c.downstream);
            last_time_state.update(input)
        };
        let output = n.time_pass(input_state, &n.name, path, context, context_tmp);
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

    pub fn time_pass(
        &self,
        node: &str,
        path: EdgePath,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) {
        self.map_down(
            node,
            path,
            Self::time_spec_extractor,
            Self::time_spec_extractor,
            Self::short_circuit_times,
            DownFns {
                prepare: Self::prepare_input_index_hashmap,
                mapper: Self::time_mapper,
            },
            time_update,
            context,
            context_tmp,
            overlay,
        );
    }

    /// Which inputs are needed to calculate time-dependent output of this node
    fn time_dependent_input_spec_extractor(
        n: &AnimationNode,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        n.time_dependent_input_spec(context, context_tmp)
    }

    /// Computes node output and caches the result for later passes
    fn time_dependent_mapper(
        n: &AnimationNode,
        inputs: HashMap<NodeInput, EdgeValue>,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let outputs = n.time_dependent_pass(inputs.clone(), &n.name, path, context, context_tmp);

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

    pub fn time_dependent_pass(
        &self,
        node: &str,
        path: EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
        overlay: &HashMap<String, AnimationNode>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        self.map_up(
            node,
            path,
            Self::time_dependent_input_spec_extractor,
            Self::time_dependent_input_spec_extractor,
            Self::short_circuit_td,
            UpFns {
                prepare: Self::prepare_input_index_hashmap,
                mapper: Self::time_dependent_mapper,
            },
            context,
            context_tmp,
            overlay,
        )
    }

    pub fn query(
        &self,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> Pose {
        context.push_caches();
        let overlay = HashMap::new();
        self.parameter_pass(Self::OUTPUT_NODE, vec![], context, context_tmp, &overlay);
        // self.dot_to_tmp_file(Some(context)).unwrap();
        self.duration_pass(Self::OUTPUT_NODE, vec![], context, context_tmp, &overlay);
        self.time_pass(
            Self::OUTPUT_NODE,
            vec![],
            time_update,
            context,
            context_tmp,
            &overlay,
        );
        self.time_dependent_pass(Self::OUTPUT_NODE, vec![], context, context_tmp, &overlay);
        // self.dot_to_tmp_file(Some(context)).unwrap();

        let output = context
            .get_time_dependent(Self::OUTPUT_NODE, &vec![])
            .unwrap()
            .upstream
            .get(self.default_output.as_ref().unwrap())
            .unwrap()
            .clone()
            .unwrap_pose_frame();

        output.sample_linear()
    }
}
