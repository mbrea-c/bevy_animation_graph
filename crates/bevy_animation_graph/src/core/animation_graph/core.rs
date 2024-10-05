use super::pin;
use crate::{
    core::{
        animation_node::AnimationNode,
        context::{CacheReadFilter, CacheWriteFilter},
        duration_data::DurationData,
        errors::{GraphError, GraphValidationError},
        pose::{BoneId, Pose},
        prelude::GraphContextArena,
        state_machine::high_level::StateMachine,
    },
    node,
    prelude::{
        DataSpec, DataValue, DeferredGizmos, OptDataSpec, PassContext, SpecContext, SystemResources,
    },
    utils::ordered_map::OrderedMap,
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

pub type NodeId = String;
pub type PinId = String;

pub type TargetPin = pin::TargetPin<NodeId, PinId>;
pub type SourcePin = pin::SourcePin<NodeId, PinId>;

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
    pub parameters: HashMap<PinId, DataValue>,
    pub durations: HashMap<PinId, DurationData>,
    pub poses: HashMap<PinId, Pose>,
}

impl InputOverlay {
    pub fn clear(&mut self) {
        self.parameters.clear();
        self.durations.clear();
        self.poses.clear();
    }
}

/// Extra data for the graph that has no effect in evaluation.
/// Used for editor data, such as node positions in screen.
#[derive(Debug, Clone, Reflect, Default, Serialize, Deserialize)]
pub struct Extra {
    /// Positions in canvas of each node
    pub node_positions: HashMap<NodeId, Vec2>,
    /// Position in canvas of special inputs node
    pub input_position: Vec2,
    /// Position in canvas of special outputs node
    pub output_position: Vec2,
}

impl Extra {
    /// Set node position (for editor)
    pub fn set_node_position(&mut self, node_id: impl Into<NodeId>, position: Vec2) {
        self.node_positions.insert(node_id.into(), position);
    }

    /// Set input node position (for editor)
    pub fn set_input_position(&mut self, position: Vec2) {
        self.input_position = position;
    }

    /// Set input node position (for editor)
    pub fn set_output_position(&mut self, position: Vec2) {
        self.output_position = position;
    }

    /// Add default position for new node if not already there
    pub fn node_added(&mut self, node_id: impl Into<NodeId>) {
        let node_id = node_id.into();
        if !self.node_positions.contains_key(&node_id) {
            self.node_positions.insert(node_id, Vec2::ZERO);
        }
    }

    /// Rename node if node exists and new name is available, otherwise return false.
    pub fn rename_node(&mut self, old_id: impl Into<NodeId>, new_id: impl Into<NodeId>) -> bool {
        let old_id = old_id.into();
        let new_id = new_id.into();

        if !self.node_positions.contains_key(&old_id) || self.node_positions.contains_key(&new_id) {
            return false;
        }

        let pos = self.node_positions.remove(&old_id).unwrap();
        self.node_positions.insert(new_id, pos);

        true
    }
}

pub type PinMap<V> = OrderedMap<PinId, V>;

#[derive(Debug, Clone, Asset, Reflect)]
pub struct AnimationGraph {
    #[reflect(ignore)]
    pub nodes: HashMap<String, AnimationNode>,
    /// Inverted, indexed by end pin.
    #[reflect(ignore)]
    pub edges: HashMap<TargetPin, SourcePin>,
    /// Inverted from the inverted state (inverted squared), indexed by start pin.
    #[reflect(ignore)]
    pub edges_inverted: HashMap<SourcePin, TargetPin>,

    pub default_parameters: PinMap<DataValue>,
    pub input_times: PinMap<()>,
    pub output_parameters: PinMap<DataSpec>,
    pub output_time: Option<()>,

    #[reflect(ignore)]
    pub extra: Extra,
}

impl Default for AnimationGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub const DEFAULT_OUTPUT_POSE: &str = "pose";

impl AnimationGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            edges_inverted: HashMap::new(),

            default_parameters: PinMap::new(),
            input_times: PinMap::new(),
            output_parameters: PinMap::new(),
            output_time: None,

            extra: Extra::default(),
        }
    }

    // --- Core graph interface: add nodes and edges
    // ----------------------------------------------------------------------------------------
    /// Add a new node to the graph
    pub fn add_node(&mut self, node: AnimationNode) {
        self.extra.node_added(&node.name);
        self.nodes.insert(node.name.clone(), node);
    }

    /// Add a new node to the graph
    pub fn remove_node(&mut self, node_id: impl Into<NodeId>) {
        let node_id = node_id.into();
        self.nodes.remove(&node_id);
        self.extra.node_positions.remove(&node_id);
    }

    /// Add a new edge to the graph
    pub fn add_edge(&mut self, source_pin: SourcePin, target_pin: TargetPin) {
        self.edges.insert(target_pin.clone(), source_pin.clone());
        self.edges_inverted.insert(source_pin, target_pin);
    }

    /// Remove an edge from the graph.
    pub fn remove_edge_by_target(&mut self, target_pin: &TargetPin) -> Option<SourcePin> {
        let source_pin = self.edges.remove(target_pin);

        if let Some(source_pin) = &source_pin {
            self.edges_inverted.remove(source_pin);
        }

        source_pin
    }

    /// Rename node if node exists and new name is available, otherwise return false.
    /// Will rename all references to the node in the graph.
    pub fn rename_node(
        &mut self,
        old_node_id: impl Into<NodeId>,
        new_node_id: impl Into<NodeId>,
    ) -> bool {
        let old_id = old_node_id.into();
        let new_id = new_node_id.into();

        if !self.nodes.contains_key(&old_id) || self.nodes.contains_key(&new_id) {
            return false;
        }

        let mut node = self.nodes.remove(&old_id).unwrap();
        node.name.clone_from(&new_id);
        self.nodes.insert(new_id.clone(), node);
        let _ = self.extra.rename_node(&old_id, &new_id);

        let keys = self.edges.keys().cloned().collect::<Vec<_>>();
        for target_pin in keys.into_iter() {
            let source_pin = self.remove_edge_by_target(&target_pin).unwrap();
            let new_target_pin = target_pin.node_renamed(old_id.clone(), new_id.clone());
            let new_source_pin = source_pin.node_renamed(old_id.clone(), new_id.clone());

            self.add_edge(new_source_pin, new_target_pin);
        }

        true
    }
    // ----------------------------------------------------------------------------------------

    // --- Setting graph inputs and outputs
    // ----------------------------------------------------------------------------------------
    /// Sets the value for a default parameter, registering it if it wasn't yet done
    pub fn set_default_parameter(&mut self, parameter_name: impl Into<String>, value: DataValue) {
        let parameter_name = parameter_name.into();
        let mut spec = OptDataSpec::from(&value);
        spec.optional = true;
        self.default_parameters
            .insert(parameter_name.clone(), value);
    }

    /// Get the default value of an input parameter, if it exists
    pub fn get_default_parameter(&mut self, parameter_name: &str) -> Option<DataValue> {
        self.default_parameters.get(parameter_name).cloned()
    }

    /// Register an input pose pin for the graph
    pub fn add_input_time(&mut self, pin_id: impl Into<PinId>) {
        self.input_times.insert(pin_id.into(), ());
    }

    /// Register an output parameter for the graph
    pub fn add_output_parameter(&mut self, pin_id: impl Into<PinId>, spec: DataSpec) {
        self.output_parameters.insert(pin_id.into(), spec);
    }

    /// Enables pose output for this graph
    pub fn add_output_time(&mut self) {
        self.output_time = Some(());
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
            SourcePin::InputData(parameter_name.into()),
            TargetPin::NodeData(target_node.into(), target_edge.into()),
        )
    }

    pub fn add_output_parameter_edge(
        &mut self,
        source_node: impl Into<NodeId>,
        source_edge: impl Into<PinId>,
        output_name: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::NodeData(source_node.into(), source_edge.into()),
            TargetPin::OutputData(output_name.into()),
        )
    }

    pub fn add_input_pose_edge(
        &mut self,
        input_name: impl Into<PinId>,
        target_node: impl Into<NodeId>,
        target_edge: impl Into<PinId>,
    ) {
        self.add_edge(
            SourcePin::InputTime(input_name.into()),
            TargetPin::NodeTime(target_node.into(), target_edge.into()),
        )
    }

    pub fn add_output_pose_edge(&mut self, source_node: impl Into<NodeId>) {
        self.add_edge(
            SourcePin::NodeTime(source_node.into()),
            TargetPin::OutputTime,
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
            SourcePin::NodeData(source_node.into(), source_pin.into()),
            TargetPin::NodeData(target_node.into(), target_pin.into()),
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
            SourcePin::NodeTime(source_node.into()),
            TargetPin::NodeTime(target_node.into(), target_pin.into()),
        );
    }
    // ----------------------------------------------------------------------------------------

    // --- Verification
    // ----------------------------------------------------------------------------------------
    pub fn validate(&self) -> Result<(), GraphValidationError> {
        enum SourceType {
            Parameter,
            Pose,
        }

        let mut counters = HashMap::<SourcePin, SourceType>::new();

        for (_, source_pin) in self.edges.iter() {
            let source_type = match source_pin {
                SourcePin::NodeData(_, _) => SourceType::Parameter,
                SourcePin::InputData(_) => SourceType::Parameter,
                SourcePin::NodeTime(_) => SourceType::Pose,
                SourcePin::InputTime(_) => SourceType::Pose,
            };

            if counters.contains_key(source_pin) {
                let ex = counters.get_mut(source_pin).unwrap();
                match (ex, source_type) {
                    (SourceType::Parameter, SourceType::Pose) => {
                        return Err(GraphValidationError::UnknownError(
                            "Inconsistent edge types connected to the same pin".into(),
                        ))
                    }
                    (SourceType::Pose, SourceType::Parameter) => {
                        return Err(GraphValidationError::UnknownError(
                            "Inconsistent edge types connected to the same pin".into(),
                        ))
                    }
                    (SourceType::Pose, SourceType::Pose) => {
                        return Err(GraphValidationError::UnknownError(
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

    /// Check whether a new edge can be added to the graph. If not, return whether an edge
    /// can be removed to maybe make it possible.
    /// It is not guaranteed that the edge will be legal after a single edge removal,
    /// so this function should be called repeatedly until it returns Ok(()) or Err(None)
    pub fn can_add_edge(&self, edge: Edge, ctx: SpecContext) -> Result<(), Option<Edge>> {
        // --- Verify source and target exist
        // -----------------------------------------------------------------
        if !self.edge_ends_exist(&edge.source, &edge.target, ctx) {
            return Err(None);
        }
        // -----------------------------------------------------------------

        // --- Verify matching types
        // -----------------------------------------------------------------
        if !self.edge_end_types_match(&edge.source, &edge.target, ctx) {
            return Err(None);
        }
        // -----------------------------------------------------------------

        // --- Verify target does not already exist
        // -----------------------------------------------------------------
        if self.edges.contains_key(&edge.target) {
            return Err(Some(Edge {
                source: self.edges.get(&edge.target).unwrap().clone(),
                target: edge.target,
            }));
        }
        // -----------------------------------------------------------------

        Ok(())
    }

    /// Verify that graph edges are legal. If not, return a set of edges that
    /// when removed would make the graph legal.
    ///
    /// Reasons for edges to make the graph illegal are:
    ///  - Two pose edges share the same source.
    ///  - An edge source and target pins have different types. This could be:
    ///    - Pose pin connected to a parameter pin.
    ///    - Pose type mismatch.
    ///    - Parameter type mismatch.
    ///  - An edge source pin, target pin or both are missing. This could be because:
    ///    - The source node, target node or both are missing.
    ///    - The source node or target node do not have the named pin.
    ///  - Cycle.
    pub fn validate_edges(&self, ctx: SpecContext) -> Result<(), HashSet<Edge>> {
        let mut illegal_edges = self.validate_pose_edges_one_to_one();
        illegal_edges.extend(self.validate_edge_type_match(ctx));
        illegal_edges.extend(self.validate_edge_ends_present(ctx));

        // TODO: Cycle detection

        if illegal_edges.is_empty() {
            Ok(())
        } else {
            Err(illegal_edges)
        }
    }

    /// Check whether a given state machine AssetId is present as a node in the graph.
    /// If true, returns name of node containing the state machine.
    pub fn contains_state_machine(&self, fsm: impl Into<AssetId<StateMachine>>) -> Option<NodeId> {
        let fsm_id = fsm.into();
        self.nodes
            .values()
            .filter_map(|node| {
                let fsm = node.inner.as_any().downcast_ref::<node::graph::Fsm>()?;
                if fsm.fsm.id() == fsm_id {
                    Some(node.name.clone())
                } else {
                    None
                }
            })
            .next()
    }

    fn extract_target_param_spec(
        &self,
        target_pin: &TargetPin,
        ctx: SpecContext,
    ) -> Option<DataSpec> {
        match target_pin {
            TargetPin::NodeData(tn, tp) => {
                let node = self.nodes.get(tn)?;
                let p_spec = node.data_input_spec(ctx);
                p_spec.get(tp).copied()
            }
            TargetPin::OutputData(op) => self.output_parameters.get(op).copied(),
            _ => None,
        }
    }

    fn extract_source_param_spec(
        &self,
        source_pin: &SourcePin,
        ctx: SpecContext,
    ) -> Option<DataSpec> {
        match source_pin {
            SourcePin::NodeData(tn, tp) => {
                let node = self.nodes.get(tn)?;
                let p_spec = node.data_output_spec(ctx);
                p_spec.get(tp).copied()
            }
            SourcePin::InputData(ip) => self.default_parameters.get(ip).map(|ip| ip.into()),
            _ => None,
        }
    }

    fn extract_source_pose_spec(&self, source_pin: &SourcePin, ctx: SpecContext) -> Option<()> {
        match source_pin {
            SourcePin::NodeTime(sn) => {
                let node = self.nodes.get(sn)?;
                node.time_output_spec(ctx)
            }
            SourcePin::InputTime(ip) => self.input_times.get(ip).copied(),
            _ => None,
        }
    }

    fn extract_target_pose_spec(&self, target_pin: &TargetPin, ctx: SpecContext) -> Option<()> {
        match target_pin {
            TargetPin::NodeTime(tn, tp) => {
                let node = self.nodes.get(tn)?;
                node.time_input_spec(ctx).get(tp).copied()
            }
            TargetPin::OutputTime => self.output_time,
            _ => None,
        }
    }

    /// Verify that no two pose edges have mismatched types. If not, return a set of edges that
    /// when removed would make the graph legal (according to this restriction).
    fn validate_edge_type_match(&self, ctx: SpecContext) -> HashSet<Edge> {
        let mut illegal_edges = HashSet::new();

        for (target_pin, source_pin) in self.edges.iter() {
            if !self.edge_end_types_match(source_pin, target_pin, ctx) {
                illegal_edges.insert(Edge {
                    source: source_pin.clone(),
                    target: target_pin.clone(),
                });
            }
        }

        illegal_edges
    }

    /// Verify that no two pose edges share the same source. If not, return a set of edges that
    /// when removed would make the graph legal (according to this restriction).
    fn validate_pose_edges_one_to_one(&self) -> HashSet<Edge> {
        let mut illegal_edges = HashSet::new();

        let mut used_sources = HashSet::<SourcePin>::new();

        for (target_pin, source_pin) in self.edges.iter() {
            match source_pin {
                SourcePin::NodeTime(_) | SourcePin::InputTime(_) => {
                    if used_sources.contains(source_pin) {
                        illegal_edges.insert(Edge {
                            source: source_pin.clone(),
                            target: target_pin.clone(),
                        });
                    } else {
                        used_sources.insert(source_pin.clone());
                    }
                }
                _ => {}
            }
        }

        illegal_edges
    }

    fn source_exists(&self, source_pin: &SourcePin, ctx: SpecContext) -> bool {
        self.extract_source_param_spec(source_pin, ctx).is_some()
            || self.extract_source_pose_spec(source_pin, ctx).is_some()
    }

    fn target_exists(&self, target_pin: &TargetPin, ctx: SpecContext) -> bool {
        self.extract_target_param_spec(target_pin, ctx).is_some()
            || self.extract_target_pose_spec(target_pin, ctx).is_some()
    }

    fn edge_end_types_match(
        &self,
        source_pin: &SourcePin,
        target_pin: &TargetPin,
        ctx: SpecContext,
    ) -> bool {
        if let Some(source_spec) = self.extract_source_param_spec(source_pin, ctx) {
            self.extract_target_param_spec(target_pin, ctx)
                .and_then(|target_spec| (source_spec == target_spec).then_some(()))
                .is_some()
        } else {
            self.extract_source_pose_spec(source_pin, ctx)
                .zip(self.extract_target_pose_spec(target_pin, ctx))
                .is_some()
        }
    }

    fn edge_ends_exist(
        &self,
        source_pin: &SourcePin,
        target_pin: &TargetPin,
        ctx: SpecContext,
    ) -> bool {
        self.source_exists(source_pin, ctx) && self.target_exists(target_pin, ctx)
    }

    // Verify that all edges have a source and target. If not, return a set of edges that
    // when removed would make the graph legal (according to this restriction).
    fn validate_edge_ends_present(&self, ctx: SpecContext) -> HashSet<Edge> {
        let mut illegal_edges = HashSet::new();

        for (target_pin, source_pin) in self.edges.iter() {
            if !self.edge_ends_exist(source_pin, target_pin, ctx) {
                illegal_edges.insert(Edge {
                    source: source_pin.clone(),
                    target: target_pin.clone(),
                });
            }
        }

        illegal_edges
    }

    // ----------------------------------------------------------------------------------------

    // --- Computations
    // ----------------------------------------------------------------------------------------
    pub fn get_data(
        &self,
        target_pin: TargetPin,
        mut ctx: PassContext,
    ) -> Result<DataValue, GraphError> {
        // Get source pin
        let Some(source_pin) = self.edges.get(&target_pin) else {
            return Err(GraphError::MissingEdgeToTarget(target_pin));
        };

        let source_value = match source_pin {
            SourcePin::NodeData(node_id, _) => {
                if ctx
                    .caches()
                    .get(
                        |c| c.is_updated(node_id).then_some(()),
                        CacheReadFilter::for_temp(ctx.temp_cache),
                    )
                    .is_none()
                {
                    let node = &self.nodes[node_id];
                    let should_debug = node.should_debug;

                    node.update(ctx.with_node(node_id, self).with_debugging(should_debug))?;

                    let is_temp = ctx.temp_cache;

                    ctx.caches_mut().set(
                        |c| c.set_updated(node_id.clone()),
                        CacheWriteFilter::for_temp(is_temp),
                    );
                }

                let Some(value) = ctx.caches().get(
                    |c| c.get_parameter(source_pin).cloned(),
                    CacheReadFilter::for_temp(ctx.temp_cache),
                ) else {
                    return Err(GraphError::OutputMissing(source_pin.clone()));
                };

                value
            }
            SourcePin::InputData(pin_id) => ctx
                .parent_data_back(pin_id)
                .ok()
                .or_else(|| ctx.overlay.parameters.get(pin_id).cloned())
                .or_else(|| self.default_parameters.get(pin_id).cloned())
                .unwrap(),
            SourcePin::NodeTime(_) => {
                // TODO: Make a graph error
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputTime(_) => {
                // TODO: Make a graph error
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
        };

        Ok(source_value)
    }

    pub fn get_duration(
        &self,
        target_pin: TargetPin,
        ctx: PassContext,
    ) -> Result<DurationData, GraphError> {
        let Some(source_pin) = self.edges.get(&target_pin) else {
            return Err(GraphError::MissingEdgeToTarget(target_pin));
        };

        let source_value = match source_pin {
            SourcePin::NodeData(_, _) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::InputData(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            SourcePin::NodeTime(node_id) => {
                if let Some(dur) = ctx.caches().get(
                    |c| c.get_duration(source_pin),
                    CacheReadFilter::for_temp(ctx.temp_cache),
                ) {
                    dur
                } else {
                    let node = &self.nodes[node_id];
                    let should_debug = node.should_debug;
                    node.duration(ctx.with_node(node_id, self).with_debugging(should_debug))?;
                    let Some(dur) = ctx.caches().get(
                        |c| c.get_duration(source_pin),
                        CacheReadFilter::for_temp(ctx.temp_cache),
                    ) else {
                        // TODO: Make a graph error for duration missing
                        return Err(GraphError::OutputMissing(source_pin.clone()));
                    };
                    dur
                }
            }
            SourcePin::InputTime(pin_id) => {
                if let Some(v) = ctx.overlay.durations.get(pin_id) {
                    *v
                } else {
                    ctx.parent().duration_back(pin_id)?
                }
            }
        };

        Ok(source_value)
    }

    pub fn get_time_update(
        &self,
        source_pin: SourcePin,
        ctx: PassContext,
    ) -> Result<TimeUpdate, GraphError> {
        let Some(target_pin) = self.edges_inverted.get(&source_pin) else {
            return Err(GraphError::MissingEdgeToSource(source_pin));
        };

        match target_pin {
            TargetPin::NodeData(_, _) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            TargetPin::OutputData(_) => {
                panic!("Incompatible pins connected: {source_pin:?} --> {target_pin:?}")
            }
            TargetPin::NodeTime(_, _) => {
                let Some(time_update) = ctx.caches().get(
                    |c| c.get_time_update_back(target_pin).cloned(),
                    CacheReadFilter::for_temp(ctx.temp_cache),
                ) else {
                    return Err(GraphError::TimeUpdateMissing(target_pin.clone()));
                };

                Ok(time_update)
            }
            TargetPin::OutputTime => {
                let Some(time_update) = ctx
                    .caches()
                    .get(
                        |c| c.get_time_update_back(target_pin).cloned(),
                        CacheReadFilter::for_temp(ctx.temp_cache),
                    )
                    .or_else(|| ctx.parent_time_update_fwd().ok())
                else {
                    return Err(GraphError::TimeUpdateMissing(target_pin.clone()));
                };

                Ok(time_update)
            }
        }
    }

    pub fn query(
        &self,
        time_update: TimeUpdate,
        context_arena: &mut GraphContextArena,
        resources: &SystemResources,
        root_entity: Entity,
        entity_map: &HashMap<BoneId, Entity>,
        deferred_gizmos: &mut DeferredGizmos,
    ) -> Result<HashMap<PinId, DataValue>, GraphError> {
        self.query_with_overlay(
            time_update,
            context_arena,
            resources,
            &InputOverlay::default(),
            root_entity,
            entity_map,
            deferred_gizmos,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_with_overlay(
        &self,
        time_update: TimeUpdate,
        context_arena: &mut GraphContextArena,
        resources: &SystemResources,
        overlay: &InputOverlay,
        root_entity: Entity,
        entity_map: &HashMap<BoneId, Entity>,
        deferred_gizmos: &mut DeferredGizmos,
    ) -> Result<HashMap<PinId, DataValue>, GraphError> {
        context_arena.next_frame();
        let mut ctx = PassContext::new(
            context_arena.get_toplevel_id(),
            context_arena,
            resources,
            overlay,
            root_entity,
            entity_map,
            deferred_gizmos,
        );
        ctx.caches_mut().set(
            |c| c.set_time_update_back(TargetPin::OutputTime, time_update),
            CacheWriteFilter::Primary,
        );
        let mut outputs = HashMap::new();
        for k in self.output_parameters.keys() {
            let out = self.get_data(TargetPin::OutputData(k.clone()), ctx.clone())?;
            outputs.insert(k.clone(), out);
        }

        Ok(outputs)
    }
    // ----------------------------------------------------------------------------------------
}
