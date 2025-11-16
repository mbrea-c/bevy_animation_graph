use bevy::{asset::AssetId, ecs::entity::Entity, platform::collections::HashMap};
use uuid::Uuid;

use crate::{
    core::{
        animation_graph::{NodeId, PinId, SourcePin, TargetPin, TimeUpdate},
        context::{deferred_gizmos::DeferredGizmoRef, graph_context_arena::SubContextId},
        duration_data::DurationData,
        errors::GraphError,
        id::BoneId,
        space_conversion::SpaceConversionContext,
        state_machine::low_level::LowLevelStateId,
    },
    prelude::{
        AnimationGraph, DataValue, GraphContextArena, GraphContextId, SystemResources,
        deferred_gizmos::DeferredGizmosContext,
        graph_context::GraphState,
        graph_context_arena::GraphContextArenaRef,
        io_env::{GraphIoEnv, GraphIoEnvBox},
        node_caches::NodeCaches,
        node_states::{GraphStateType, NodeStates, StateKey},
        pose_fallback::PoseFallbackContext,
    },
};

#[derive(Clone)]
pub struct GraphContext<'a> {
    pub context_id: GraphContextId,
    pub root_entity: Entity,
    pub state_key: StateKey,
    pub should_debug: bool,
    pub io: GraphIoEnvBox<'a>,

    pub context_arena: GraphContextArenaRef,
    pub deferred_gizmos: DeferredGizmoRef,

    pub resources: &'a SystemResources<'a, 'a>,
    pub entity_map: &'a HashMap<BoneId, Entity>,
}

impl<'a> GraphContext<'a> {
    pub fn new(
        context_id: GraphContextId,
        context_arena: &mut GraphContextArena,
        resources: &'a SystemResources,
        io: &'a dyn GraphIoEnv,
        root_entity: Entity,
        entity_map: &'a HashMap<BoneId, Entity>,
        deferred_gizmos: impl Into<DeferredGizmoRef>,
    ) -> Self {
        Self {
            context_id,
            root_entity,
            state_key: StateKey::Default,
            should_debug: false,
            io: GraphIoEnvBox::new(io),
            context_arena: context_arena.into(),
            deferred_gizmos: deferred_gizmos.into(),
            resources,
            entity_map,
        }
    }

    /// Decorates a pass context with node data. Usually done by `AnimationGraph` before
    /// passing the context down to a node.
    pub fn create_node_context(
        &self,
        node_id: &'a NodeId,
        graph: &'a AnimationGraph,
    ) -> NodeContext<'a> {
        NodeContext {
            node_id,
            graph,
            graph_context: self.clone(),
        }
    }

    pub fn with_debugging(mut self, should_debug: bool) -> Self {
        self.should_debug = should_debug;
        self
    }

    /// Return a mutable reference to the [`GraphState`]
    pub fn context_mut(&mut self) -> &mut GraphState {
        self.context_arena
            .as_mut()
            .get_context_mut(self.context_id)
            .unwrap()
    }

    /// Return a reference to the [`GraphState`]
    pub fn context(&self) -> &GraphState {
        self.context_arena
            .as_ref()
            .get_context(self.context_id)
            .unwrap()
    }

    pub fn node_caches_mut(&mut self) -> &mut NodeCaches {
        &mut self.context_mut().node_caches
    }

    pub fn node_caches(&self) -> &NodeCaches {
        &self.context().node_caches
    }

    pub fn node_states_mut(&mut self) -> &mut NodeStates {
        &mut self.context_mut().node_states
    }

    pub fn node_states(&self) -> &NodeStates {
        &self.context().node_states
    }

    pub fn space_conversion(&'_ self) -> SpaceConversionContext<'_> {
        SpaceConversionContext {
            pose_fallback: PoseFallbackContext {
                entity_map: self.entity_map,
                resources: self.resources,
                fallback_to_identity: false,
            },
        }
    }

    pub fn gizmos(&'_ self) -> DeferredGizmosContext<'_> {
        DeferredGizmosContext {
            gizmos: self.deferred_gizmos.as_mut(),
            resources: self.resources,
            entity_map: self.entity_map,
            space_conversion: self.space_conversion(),
        }
    }

    pub fn use_debug_gizmos(&self, action: impl Fn(DeferredGizmosContext)) {
        if self.should_debug {
            let ctx = self.gizmos();
            action(ctx);
        }
    }

    /// Returns a new pass context with the given temp cache value.
    pub fn with_temp_state_key(mut self) -> Self {
        self.state_key = StateKey::Temporary(Uuid::new_v4());
        self
    }

    pub fn with_state_key(mut self, new_key: StateKey) -> Self {
        self.state_key = new_key;
        self
    }

    pub fn with_io(mut self, new_io: &'a dyn GraphIoEnv) -> Self {
        self.io = GraphIoEnvBox::new(new_io);
        self
    }
}

#[derive(Clone)]
pub struct NodeContext<'a> {
    pub node_id: &'a NodeId,
    pub graph: &'a AnimationGraph,
    pub graph_context: GraphContext<'a>,
}

impl<'a> NodeContext<'a> {
    pub fn with_debugging(mut self, should_debug: bool) -> Self {
        self.graph_context = self.graph_context.with_debugging(should_debug);
        self
    }

    /// Request an input parameter from the graph
    pub fn data_back(&self, pin_id: impl Into<PinId>) -> Result<DataValue, GraphError> {
        let target_pin = TargetPin::NodeData(self.node_id.clone(), pin_id.into());
        self.graph.get_data(target_pin, self.graph_context.clone())
    }

    /// Sets the output value at the given pin for the current node. It's up to the caller to
    /// verify the types are correct, or suffer the consequences.
    pub fn set_data_fwd(&mut self, pin_id: impl Into<PinId>, data: impl Into<DataValue>) {
        let key = self.graph_context.state_key;
        self.graph_context
            .context_mut()
            .node_caches
            .set_output_data(self.node_id.clone(), key, pin_id.into(), data.into());
    }

    /// Request the duration of an input pose pin.
    pub fn duration_back(&self, pin_id: impl Into<PinId>) -> Result<DurationData, GraphError> {
        let target_pin = TargetPin::NodeTime(self.node_id.clone(), pin_id.into());
        self.graph
            .get_duration(target_pin, self.graph_context.clone())
    }

    /// Sets the duration of the current node with current settings.
    pub fn set_duration_fwd(&mut self, duration: DurationData) {
        let key = self.graph_context.state_key;
        self.graph_context.context_mut().node_caches.set_duration(
            self.node_id.clone(),
            key,
            duration,
        );
    }

    /// Sets the duration of the current node with current settings.
    pub fn set_time_update_back(&mut self, pin_id: impl Into<PinId>, time_update: TimeUpdate) {
        let key = self.graph_context.state_key;
        self.graph_context
            .context_mut()
            .node_caches
            .set_input_time_update(self.node_id.clone(), key, pin_id.into(), time_update);
    }

    /// Sets the time state of the current node.
    pub fn set_time(&mut self, time: f32) {
        let key = self.graph_context.state_key;
        self.graph_context
            .context_mut()
            .node_states
            .set_time(self.node_id.clone(), key, time);
    }

    /// Request the cached time update query from the current frame
    pub fn time_update_fwd(&self) -> Result<TimeUpdate, GraphError> {
        let source_pin = SourcePin::NodeTime(self.node_id.clone());
        self.graph
            .get_time_update(source_pin, self.graph_context.clone())
    }

    /// Request the cached timestamp of the output animation in the last frame
    pub fn prev_time(&self) -> f32 {
        self.graph_context
            .context()
            .node_states
            .get_last_time(self.node_id.clone())
    }

    /// Request the cached timestamp of the output animation in the last frame
    pub fn time(&mut self) -> f32 {
        let key = self.graph_context.state_key;
        self.graph_context
            .context_mut()
            .node_states
            .get_time(self.node_id.clone(), key)
    }

    pub fn state<T: GraphStateType>(&self) -> Result<&T, GraphError> {
        let key = self.graph_context.state_key;
        self.graph_context
            .node_states()
            .get::<T>(self.node_id.clone(), key)
    }

    pub fn state_mut<T: GraphStateType + Default>(&mut self) -> Result<&mut T, GraphError> {
        self.state_mut_or_else(T::default)
    }

    pub fn state_mut_or_else<T: GraphStateType>(
        &mut self,
        default: impl FnOnce() -> T,
    ) -> Result<&mut T, GraphError> {
        let key = self.graph_context.state_key;
        self.graph_context.node_states_mut().get_mut_or_insert_with(
            self.node_id.clone(),
            key,
            default,
        )
    }

    pub fn with_temp_state_key(mut self) -> Self {
        self.graph_context = self.graph_context.with_temp_state_key();
        self
    }

    pub fn with_state_key(mut self, new_key: StateKey) -> Self {
        self.graph_context = self.graph_context.with_state_key(new_key);
        self
    }

    pub fn create_child_context(
        &self,
        subgraph_id: AssetId<AnimationGraph>,
        fsm_state: Option<LowLevelStateId>,
    ) -> GraphContext<'a> {
        let subctx_id = SubContextId {
            ctx_id: self.graph_context.context_id,
            node_id: self.node_id.to_owned(),
            state_id: fsm_state,
        };

        GraphContext {
            context_id: self
                .graph_context
                .context_arena
                .as_mut()
                .get_sub_context_or_insert_default(subctx_id, subgraph_id),
            ..self.graph_context.clone()
        }
    }
}
