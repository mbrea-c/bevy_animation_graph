use crate::{
    core::{
        animation_node::{AnimationNode, NodeLike},
        duration_data::DurationData,
        frame::{BonePoseFrame, PoseFrame, PoseSpec},
        pose::Pose,
    },
    prelude::{
        GraphContext, OptParamSpec, ParamSpec, ParamValue, PassContext, SampleLinearAt,
        SystemResources,
    },
    utils::unwrap::Unwrap,
};
use bevy::{prelude::*, reflect::TypeUuid, utils::HashMap};
use std::error::Error;

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

impl TimeUpdate {
    pub fn apply(&self, time: f32) -> f32 {
        match self {
            Self::Delta(dt) => time + dt,
            Self::Absolute(t) => *t,
        }
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
    pub poses: HashMap<PinId, PoseFrame>,
}

impl InputOverlay {
    pub fn clear(&mut self) {
        self.parameters.clear();
        self.durations.clear();
        self.poses.clear();
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
    pub edges: HashMap<TargetPin, SourcePin>,
    pub default_output: Option<String>,

    pub default_parameters: HashMap<PinId, ParamValue>,
    pub input_parameters: HashMap<PinId, OptParamSpec>,
    pub input_poses: HashMap<PinId, PoseSpec>,
    pub output_parameters: HashMap<PinId, ParamSpec>,
    pub output_pose: Option<PoseSpec>,
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
            edges: HashMap::new(),

            default_output: None,
            default_parameters: HashMap::new(),
            input_parameters: HashMap::new(),
            input_poses: HashMap::new(),
            output_parameters: HashMap::new(),
            output_pose: None,
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
        self.edges.insert(target_pin, source_pin);
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
    pub fn add_input_pose(&mut self, pin_id: impl Into<PinId>, spec: PoseSpec) {
        self.input_poses.insert(pin_id.into(), spec);
    }

    /// Register an output parameter for the graph
    pub fn add_output_parameter(&mut self, pin_id: impl Into<PinId>, spec: ParamSpec) {
        self.output_parameters.insert(pin_id.into(), spec);
    }

    /// Enables pose output for this graph
    pub fn add_output_pose(&mut self, frame_type: PoseSpec) {
        self.output_pose = Some(frame_type);
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

        for (_, source_pin) in self.edges.iter() {
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
    pub fn get_parameter(&self, target_pin: TargetPin, mut ctx: PassContext) -> Option<ParamValue> {
        let source_pin = self.edges.get(&target_pin);

        let Some(source_pin) = source_pin else {
            return None;
        };

        if let Some(val) = ctx.context().get_parameter(source_pin) {
            return Some(val.clone());
        }

        let source_value = match source_pin {
            SourcePin::NodeParameter(node_id, pin_id) => {
                let outputs = self.nodes[node_id].parameter_pass(ctx.with_node(node_id, self));

                for (pin_id, value) in outputs.iter() {
                    ctx.context().set_parameter(
                        SourcePin::NodeParameter(node_id.clone(), pin_id.clone()),
                        value.clone(),
                    );
                }

                outputs[pin_id].clone()
            }
            SourcePin::InputParameter(pin_id) => {
                let out = if ctx.has_parent() {
                    ctx.parent().parameter_back_opt(pin_id)
                } else {
                    None
                }
                .or_else(|| ctx.overlay.parameters.get(pin_id).cloned())
                .or_else(|| self.default_parameters.get(pin_id).cloned());
                out.unwrap()
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

    pub fn get_duration(&self, target_pin: TargetPin, mut ctx: PassContext) -> DurationData {
        let source_pin = self
            .edges
            .get(&target_pin)
            .unwrap_or_else(|| panic!("Target pin {target_pin:?} is disconnected"));

        if let Some(val) = ctx.context().get_duration(source_pin) {
            return val;
        }

        let source_value = match source_pin {
            SourcePin::NodeParameter(_, _) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputParameter(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::NodePose(node_id) => {
                let output = self.nodes[node_id].duration_pass(ctx.with_node(node_id, self));

                if let Some(value) = output {
                    ctx.context()
                        .set_duration(SourcePin::NodePose(node_id.clone()), value);
                }

                output.unwrap()
            }
            SourcePin::InputPose(pin_id) => {
                if let Some(v) = ctx.overlay.durations.get(pin_id) {
                    *v
                } else {
                    ctx.parent().duration_back(pin_id)
                }
            }
        };

        source_value
    }

    pub fn get_pose(
        &self,
        time_update: TimeUpdate,
        target_pin: TargetPin,
        mut ctx: PassContext,
    ) -> PoseFrame {
        let source_pin = self.edges.get(&target_pin).unwrap();

        if let Some(val) = ctx.context().get_pose(source_pin) {
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
                let output = self.nodes[node_id]
                    .pose_pass(time_update, ctx.with_node(node_id, self))
                    .unwrap();

                ctx.context().set_pose(source_pin.clone(), output.clone());
                ctx.context().set_time(source_pin.clone(), output.timestamp);

                output
            }
            SourcePin::InputPose(pin_id) => {
                if let Some(v) = ctx.overlay.poses.get(pin_id) {
                    v.clone()
                } else {
                    ctx.parent().pose_back(pin_id, time_update)
                }
            }
        };

        source_value
    }

    pub fn query(
        &self,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        resources: SystemResources,
    ) -> Pose {
        self.query_with_overlay(time_update, context, resources, &InputOverlay::default())
    }

    pub fn query_with_overlay(
        &self,
        time_update: TimeUpdate,
        context: &mut GraphContext,
        resources: SystemResources,
        overlay: &InputOverlay,
    ) -> Pose {
        context.push_caches();
        let out = self.get_pose(
            time_update,
            TargetPin::OutputPose,
            PassContext::new(context, resources, overlay),
        );
        let time = out.timestamp;
        let bone_frame: BonePoseFrame = out.data.unwrap();

        bone_frame.sample_linear_at(time)
    }
    // ----------------------------------------------------------------------------------------
}
