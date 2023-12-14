use std::error::Error;

use crate::{
    core::{
        animation_node::{AnimationNode, NodeLike},
        frame::PoseFrame,
        graph_context::{GraphContext, GraphContextTmp},
        pose::Pose,
    },
    prelude::{DurationData, PassContext, SpecContext},
    sampling::linear::SampleLinear,
};
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OptParamSpec {
    pub spec: ParamSpec,
    pub optional: bool,
}

impl OptParamSpec {
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

impl From<ParamSpec> for OptParamSpec {
    fn from(value: ParamSpec) -> Self {
        Self {
            spec: value,
            optional: false,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParamSpec {
    F32,
}

pub type NodeId = String;
pub type PinId = String;

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetPin {
    NodeParameter(NodeId, PinId),
    OutputParameter(PinId),
    NodePose(NodeId, PinId),
    OutputPose,
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourcePin {
    NodeParameter(NodeId, PinId),
    InputParameter(PinId),
    NodePose(NodeId),
    InputPose(PinId),
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub source: SourcePin,
    pub target: TargetPin,
}

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub enum ParamValue {
    F32(f32),
}

impl ParamValue {
    pub fn unwrap_f32(self) -> f32 {
        match self {
            Self::F32(f) => f,
        }
    }
}

impl From<f32> for ParamValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<&ParamValue> for OptParamSpec {
    fn from(value: &ParamValue) -> Self {
        match value {
            ParamValue::F32(_) => Self {
                spec: ParamSpec::F32,
                optional: false,
            },
        }
    }
}

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

#[derive(Debug, Clone, Reflect, Default)]
pub struct InputOverlay {
    pub parameters: HashMap<PinId, ParamValue>,
    pub durations: HashMap<PinId, DurationData>,
    pub time_dependent: HashMap<PinId, PoseFrame>,
}

impl InputOverlay {
    pub fn clear(&mut self) {
        self.parameters.clear();
        self.durations.clear();
        self.time_dependent.clear();
    }
}

#[derive(Debug, Clone, Reflect, Default)]
pub struct GraphError(String);

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Inconsistent graph: {}", self.0)
    }
}

impl Error for GraphError {}

#[derive(Debug, Clone, Asset, TypeUuid, Reflect)]
#[uuid = "92411396-01ae-4528-9839-709a7a321263"]
pub struct AnimationGraph {
    pub nodes: HashMap<String, AnimationNode>,
    /// Inverted, indexed by output node name.
    pub node_edges: HashMap<TargetPin, SourcePin>,
    pub default_output: Option<String>,

    pub default_parameters: HashMap<PinId, ParamValue>,
    pub input_parameters: HashMap<PinId, OptParamSpec>,
    pub input_poses: HashSet<PinId>,
    pub output_parameters: HashMap<PinId, ParamSpec>,
    pub output_pose: bool,
}

impl Default for AnimationGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_edges: HashMap::new(),

            default_output: None,
            default_parameters: HashMap::new(),
            input_parameters: HashMap::new(),
            input_poses: HashSet::new(),
            output_parameters: HashMap::new(),
            output_pose: false,
        }
    }

    // --- Core graph interface: add nodes and edges
    // ----------------------------------------------------------------------------------------
    /// Add a new node to the graph
    pub fn add_node(&mut self, node: AnimationNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Add a new edge to the graph
    pub fn add_edge(&mut self, source_pin: SourcePin, target_pin: TargetPin) {
        self.node_edges.insert(target_pin, source_pin);
    }
    // ----------------------------------------------------------------------------------------

    // --- Setting graph inputs and outputs
    // ----------------------------------------------------------------------------------------
    /// Sets the value for a default parameter, registering it if it wasn't yet done
    pub fn set_default_parameter(&mut self, parameter_name: impl Into<String>, value: ParamValue) {
        let parameter_name = parameter_name.into();
        let mut spec = OptParamSpec::from(&value);
        spec.optional = true;
        self.input_parameters.insert(parameter_name.clone(), spec);
        self.default_parameters
            .insert(parameter_name.clone(), value);
    }

    /// Get the default value of an input parameter, if it exists
    pub fn get_default_parameter(&mut self, parameter_name: &str) -> Option<ParamValue> {
        self.default_parameters.get(parameter_name).cloned()
    }

    /// Register an input pose pin for the graph
    pub fn add_input_pose(&mut self, pin_id: impl Into<PinId>) {
        self.input_poses.insert(pin_id.into());
    }

    /// Register an output parameter for the graph
    pub fn add_output_parameter(&mut self, pin_id: impl Into<PinId>, spec: ParamSpec) {
        self.output_parameters.insert(pin_id.into(), spec);
    }

    /// Enables pose output for this graph
    pub fn add_output_pose(&mut self) {
        self.output_pose = true;
    }
    // ----------------------------------------------------------------------------------------

    // --- Helper functions for adding edges
    // ----------------------------------------------------------------------------------------
    pub fn add_input_parameter_edge(
        &mut self,
        parameter_name: impl Into<PinId>,
        target_node: impl Into<NodeId>,
        target_edge: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::InputParameter(parameter_name.into()),
            TargetPin::NodeParameter(target_node.into(), target_edge.into()),
        )
    }

    pub fn add_output_parameter_edge(
        &mut self,
        source_node: impl Into<NodeId>,
        source_edge: impl Into<PinId>,
        output_name: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::NodeParameter(source_node.into(), source_edge.into()),
            TargetPin::OutputParameter(output_name.into()),
        )
    }

    pub fn add_input_pose_edge(
        &mut self,
        input_name: impl Into<PinId>,
        target_node: impl Into<NodeId>,
        target_edge: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::InputPose(input_name.into()),
            TargetPin::NodePose(target_node.into(), target_edge.into()),
        )
    }

    pub fn add_output_pose_edge(&mut self, source_node: impl Into<NodeId>) {
        self.add_edge(
            SourcePin::NodePose(source_node.into()),
            TargetPin::OutputPose,
        )
    }

    /// Adds an edge between two nodes in the graph
    pub fn add_node_parameter_edge(
        &mut self,
        source_node: impl Into<NodeId>,
        source_pin: impl Into<PinId>,
        target_node: impl Into<NodeId>,
        target_pin: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::NodeParameter(source_node.into(), source_pin.into()),
            TargetPin::NodeParameter(target_node.into(), target_pin.into()),
        );
    }

    /// Adds an edge between two node pose pins in the graph
    pub fn add_node_pose_edge(
        &mut self,
        source_node: impl Into<NodeId>,
        target_node: impl Into<NodeId>,
        target_pin: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::NodePose(source_node.into()),
            TargetPin::NodePose(target_node.into(), target_pin.into()),
        );
    }
    // ----------------------------------------------------------------------------------------

    // --- Verification
    // ----------------------------------------------------------------------------------------
    pub fn validate(&self) -> Result<(), GraphError> {
        enum SourceType {
            Parameter,
            Pose,
        }

        let mut counters = HashMap::<SourcePin, SourceType>::new();

        for (_, source_pin) in self.node_edges.iter() {
            let source_type = match source_pin {
                SourcePin::NodeParameter(_, _) => SourceType::Parameter,
                SourcePin::InputParameter(_) => SourceType::Parameter,
                SourcePin::NodePose(_) => SourceType::Pose,
                SourcePin::InputPose(_) => SourceType::Pose,
            };

            if counters.contains_key(source_pin) {
                let ex = counters.get_mut(source_pin).unwrap();
                match (ex, source_type) {
                    (SourceType::Parameter, SourceType::Pose) => {
                        return Err(GraphError(
                            "Inconsistent edge types connected to the same pin".into(),
                        ))
                    }
                    (SourceType::Pose, SourceType::Parameter) => {
                        return Err(GraphError(
                            "Inconsistent edge types connected to the same pin".into(),
                        ))
                    }
                    (SourceType::Pose, SourceType::Pose) => {
                        return Err(GraphError(
                            "Only one target can be connected to each pose output".into(),
                        ))
                    }
                    _ => (),
                };
            } else {
                counters.insert(source_pin.clone(), source_type);
            }
        }
        Ok(())
    }
    // ----------------------------------------------------------------------------------------

    // --- Computations
    // ----------------------------------------------------------------------------------------
    fn parameter_map(
        &self,
        target_pin: TargetPin,
        spec: OptParamSpec,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> Option<ParamValue> {
        let source_pin = self.node_edges.get(&target_pin);
        if spec.optional && source_pin.is_none() {
            return None;
        }
        let source_pin = source_pin.unwrap();

        if let Some(val) = context.get_cached_parameter(source_pin) {
            return Some(val.clone());
        }

        let source_value = match source_pin {
            SourcePin::NodeParameter(node_id, pin_id) => {
                self.nodes[node_id]
                    .pose_input_spec(SpecContext::new(context, context_tmp))
                    .iter()
                    .for_each(|pin_id| {
                        self.parameter_propagate(
                            TargetPin::NodePose(node_id.clone(), pin_id.clone()),
                            context,
                            context_tmp,
                            overlay,
                        );
                    });
                let inputs = self.nodes[node_id]
                    .parameter_input_spec(SpecContext::new(context, context_tmp))
                    .iter()
                    .filter_map(|(pin_id, spec)| {
                        self.parameter_map(
                            TargetPin::NodeParameter(node_id.clone(), pin_id.clone()),
                            *spec,
                            context,
                            context_tmp,
                            overlay,
                        )
                        .map(|v| (pin_id.clone(), v))
                    })
                    .collect();

                let outputs = self.nodes[node_id].parameter_pass(
                    inputs,
                    PassContext::new(node_id, context, context_tmp, &self.node_edges),
                );

                for (pin_id, value) in outputs.iter() {
                    context.insert_cached_parameter(
                        SourcePin::NodeParameter(node_id.clone(), pin_id.clone()),
                        value.clone(),
                    );
                }

                outputs[pin_id].clone()
            }
            SourcePin::InputParameter(pin_id) => {
                if let Some(v) = overlay.parameters.get(pin_id) {
                    v.clone()
                } else if let Some(v) = self.default_parameters.get(pin_id) {
                    v.clone()
                } else {
                    panic!("Value of parameter {source_pin:?} not available")
                }
            }
            SourcePin::NodePose(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputPose(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
        };

        Some(source_value)
    }

    fn parameter_propagate(
        &self,
        target_pin: TargetPin,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) {
        let source_pin = self.node_edges.get(&target_pin).unwrap();

        let source_value = match source_pin {
            SourcePin::NodeParameter(_, _) => {
                panic!("Try using parameter_map instead: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputParameter(_) => {
                panic!("Try using parameter_map instead: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::NodePose(node_id) => {
                self.nodes[node_id]
                    .pose_input_spec(SpecContext::new(context, context_tmp))
                    .iter()
                    .for_each(|pin_id| {
                        self.parameter_propagate(
                            TargetPin::NodePose(node_id.clone(), pin_id.clone()),
                            context,
                            context_tmp,
                            overlay,
                        );
                    });
                self.nodes[node_id]
                    .parameter_input_spec(SpecContext::new(context, context_tmp))
                    .iter()
                    .for_each(|(pin_id, spec)| {
                        self.parameter_map(
                            TargetPin::NodeParameter(node_id.clone(), pin_id.clone()),
                            *spec,
                            context,
                            context_tmp,
                            overlay,
                        );
                    });
            }
            SourcePin::InputPose(_) => {}
        };

        source_value
    }

    fn duration_map(
        &self,
        target_pin: TargetPin,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> Option<f32> {
        let source_pin = self
            .node_edges
            .get(&target_pin)
            .unwrap_or_else(|| panic!("Target pin {target_pin:?} is disconnected"));

        if let Some(val) = context.get_cached_duration(source_pin) {
            return val.clone();
        }

        let source_value = match source_pin {
            SourcePin::NodeParameter(_, _) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputParameter(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::NodePose(node_id) => {
                let inputs = self.nodes[node_id]
                    .pose_input_spec(SpecContext::new(context, context_tmp))
                    .iter()
                    .map(|pin_id| {
                        (
                            pin_id.clone(),
                            self.duration_map(
                                TargetPin::NodePose(node_id.clone(), pin_id.clone()),
                                context,
                                context_tmp,
                                overlay,
                            ),
                        )
                    })
                    .collect();

                let output = self.nodes[node_id].duration_pass(
                    inputs,
                    PassContext::new(node_id, context, context_tmp, &self.node_edges),
                );

                if let Some(value) = output {
                    context.insert_cached_duration(
                        SourcePin::NodePose(node_id.clone()),
                        value.clone(),
                    );
                }

                output.unwrap().clone()
            }
            SourcePin::InputPose(pin_id) => {
                if let Some(v) = overlay.durations.get(pin_id) {
                    v.clone()
                } else {
                    panic!("Value of parameter {source_pin:?} not available")
                }
            }
        };

        source_value
    }

    fn time_map(
        &self,
        target_pin: TargetPin,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) {
        let source_pin = self.node_edges.get(&target_pin).unwrap();

        if let Some(_) = context.get_cached_time(source_pin) {
            return;
        }

        let old_time_state = context
            .old_cached_time(source_pin)
            .cloned()
            .unwrap_or_default();
        let time_state = old_time_state.update(time_update);

        // Cache the new value
        context.insert_cached_time(source_pin.clone(), time_state);

        match source_pin {
            SourcePin::NodeParameter(_, _) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputParameter(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::NodePose(node_id) => {
                // Compute time pass
                let back_target_pins = self.nodes[node_id].time_pass(
                    time_state,
                    PassContext::new(node_id, context, context_tmp, &self.node_edges),
                );

                // Propagate the time update to the back edges
                for (pin_id, time_update) in back_target_pins {
                    self.time_map(
                        TargetPin::NodePose(node_id.clone(), pin_id),
                        time_update,
                        context,
                        context_tmp,
                        overlay,
                    );
                }
            }
            SourcePin::InputPose(_) => {
                // Do nothing, the value has already been cached
            }
        };
    }

    fn pose_map(
        &self,
        target_pin: TargetPin,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> PoseFrame {
        let source_pin = self.node_edges.get(&target_pin).unwrap();

        if let Some(val) = context.get_cached_pose(source_pin) {
            return val.clone();
        }

        let source_value = match source_pin {
            SourcePin::NodeParameter(_, _) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputParameter(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::NodePose(node_id) => {
                let inputs = self.nodes[node_id]
                    .pose_input_spec(SpecContext::new(context, context_tmp))
                    .iter()
                    .map(|pin_id| {
                        (
                            pin_id.clone(),
                            self.pose_map(
                                TargetPin::NodePose(node_id.clone(), pin_id.clone()),
                                context,
                                context_tmp,
                                overlay,
                            ),
                        )
                    })
                    .collect();

                let output = self.nodes[node_id].time_dependent_pass(
                    inputs,
                    PassContext::new(node_id, context, context_tmp, &self.node_edges),
                );

                if let Some(value) = &output {
                    context.insert_cached_pose(SourcePin::NodePose(node_id.clone()), value.clone());
                }

                output.unwrap().clone()
            }
            SourcePin::InputPose(pin_id) => {
                if let Some(v) = overlay.time_dependent.get(pin_id) {
                    v.clone()
                } else {
                    panic!("Value of parameter {source_pin:?} not available")
                }
            }
        };

        source_value
    }

    pub fn parameter_pass(
        &self,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> HashMap<PinId, ParamValue> {
        if self.output_pose {
            self.parameter_propagate(TargetPin::OutputPose, context, context_tmp, overlay);
        }

        self.output_parameters
            .iter()
            .map(|(pin_id, spec)| {
                let target_pin = TargetPin::OutputParameter(pin_id.clone());
                let value = self
                    .parameter_map(target_pin, (*spec).into(), context, context_tmp, overlay)
                    .unwrap();

                (pin_id.clone(), value)
            })
            .collect()
    }

    pub fn duration_pass(
        &self,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> Option<DurationData> {
        let target_pin = TargetPin::OutputPose;

        Some(self.duration_map(target_pin, context, context_tmp, overlay))
    }

    pub fn time_pass(
        &self,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> HashMap<PinId, TimeUpdate> {
        let target_pin = TargetPin::OutputPose;

        self.time_map(target_pin, time_update, context, context_tmp, overlay);

        self.input_poses
            .iter()
            .map(|pin_id| {
                let source_pin = SourcePin::InputPose(pin_id.clone());
                let state = context.get_cached_time(&source_pin).unwrap();

                (pin_id.clone(), state.update)
            })
            .collect()
    }

    pub fn time_dependent_pass(
        &self,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> Option<PoseFrame> {
        let target_pin = TargetPin::OutputPose;

        Some(self.pose_map(target_pin, context, context_tmp, overlay))
    }

    pub fn query(
        &self,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
    ) -> Pose {
        self.query_with_overlay(time_update, context, context_tmp, &InputOverlay::default())
    }

    pub fn query_with_overlay(
        &self,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        context_tmp: GraphContextTmp,
        overlay: &InputOverlay,
    ) -> Pose {
        context.push_caches();
        self.parameter_pass(context, context_tmp, overlay);
        self.duration_pass(context, context_tmp, overlay);
        self.time_pass(time_update, context, context_tmp, overlay);
        let final_output = self.time_dependent_pass(context, context_tmp, overlay);

        final_output.unwrap().sample_linear()
    }
    // ----------------------------------------------------------------------------------------
}
