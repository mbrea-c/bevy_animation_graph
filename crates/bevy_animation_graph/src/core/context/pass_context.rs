use super::{
    deferred_gizmos::DeferredGizmoRef,
    graph_context::{CacheReadFilter, CacheWriteFilter, GraphStateStack},
    graph_context_arena::{GraphContextArena, GraphContextId, SubContextId},
    GraphContext, SpecContext, SystemResources,
};
use crate::{
    core::{
        animation_graph::{InputOverlay, NodeId, PinId, SourcePin, TargetPin, TimeUpdate},
        duration_data::DurationData,
        errors::GraphError,
        pose::BoneId,
        state_machine::{LowLevelStateMachine, StateId},
    },
    prelude::{AnimationGraph, DataValue},
};
use bevy::{ecs::entity::Entity, utils::HashMap};

#[derive(Clone, Copy)]
pub struct NodeContext<'a> {
    pub node_id: &'a NodeId,
    pub graph: &'a AnimationGraph,
}

#[derive(Clone, Copy)]
pub enum StateRole {
    Source,
    Target,
    Root,
}

#[derive(Clone)]
pub struct FsmContext<'a> {
    /// The stack of states in the FSM call chain
    /// The last state is the current state, if this state is the source/target state in a
    /// transition state then the previous state in the stack is that transition state.
    pub state_stack: StateStack,
    pub fsm: &'a LowLevelStateMachine,
}

#[derive(Clone)]
pub struct StateStack {
    pub stack: Vec<(StateId, StateRole)>,
}

impl StateStack {
    pub fn last_state(&self) -> StateId {
        self.stack.last().unwrap().0.clone()
    }
    pub fn last_role(&self) -> StateRole {
        self.stack.last().unwrap().1
    }
}

#[derive(Clone)]
pub struct PassContext<'a> {
    pub context_id: GraphContextId,
    pub context_arena: GraphContextArenaRef,
    pub resources: &'a SystemResources<'a, 'a>,
    pub overlay: &'a InputOverlay,
    pub node_context: Option<NodeContext<'a>>,
    // Is `Some(...)` whenever the current graph is a state in some FSM
    pub fsm_context: Option<FsmContext<'a>>,
    pub parent: Option<PassContextRef<'a>>,
    pub root_entity: Entity,
    pub entity_map: &'a HashMap<BoneId, Entity>,
    pub deferred_gizmos: DeferredGizmoRef,
    /// Whether this query should mutate the *permanent* or *temporary* chache. Useful when getting
    /// a pose back but not wanting to use the time query to update the times
    pub temp_cache: bool,
    pub should_debug: bool,
}

impl<'a> PassContext<'a> {
    /// Creates a pass context with no parent graph nor node context data
    pub fn new(
        context_id: GraphContextId,
        context_arena: &mut GraphContextArena,
        resources: &'a SystemResources,
        overlay: &'a InputOverlay,
        root_entity: Entity,
        entity_map: &'a HashMap<BoneId, Entity>,
        deferred_gizmos: impl Into<DeferredGizmoRef>,
    ) -> Self {
        Self {
            context_id,
            context_arena: context_arena.into(),
            resources,
            overlay,
            node_context: None,
            parent: None,
            root_entity,
            entity_map,
            deferred_gizmos: deferred_gizmos.into(),
            temp_cache: false,
            should_debug: false,
            fsm_context: None,
        }
    }

    /// Decorates a pass context with node data. Usually done by `AnimationGraph` before
    /// passing the context down to a node.
    pub fn with_node(&self, node_id: &'a NodeId, graph: &'a AnimationGraph) -> Self {
        Self {
            context_id: self.context_id,
            context_arena: self.context_arena.clone(),
            resources: self.resources,
            overlay: self.overlay,
            node_context: Some(NodeContext { node_id, graph }),
            parent: self.parent.clone(),
            root_entity: self.root_entity,
            entity_map: self.entity_map,
            deferred_gizmos: self.deferred_gizmos.clone(),
            temp_cache: self.temp_cache,
            should_debug: self.should_debug,
            fsm_context: self.fsm_context.clone(),
        }
    }

    /// Returns a pass context with node data cleared. Usually done before passing the
    /// context back up to the graph to request further inputs.
    pub fn without_node(&self) -> Self {
        Self {
            context_id: self.context_id,
            context_arena: self.context_arena.clone(),
            resources: self.resources,
            overlay: self.overlay,
            node_context: None,
            parent: self.parent.clone(),
            root_entity: self.root_entity,
            entity_map: self.entity_map,
            deferred_gizmos: self.deferred_gizmos.clone(),
            temp_cache: self.temp_cache,
            should_debug: self.should_debug,
            fsm_context: self.fsm_context.clone(),
        }
    }

    /// Returns a pass context with updated `should_debug`
    pub fn with_debugging(&self, should_debug: bool) -> Self {
        Self {
            context_id: self.context_id,
            context_arena: self.context_arena.clone(),
            resources: self.resources,
            overlay: self.overlay,
            node_context: self.node_context,
            parent: self.parent.clone(),
            root_entity: self.root_entity,
            entity_map: self.entity_map,
            deferred_gizmos: self.deferred_gizmos.clone(),
            temp_cache: self.temp_cache,
            fsm_context: self.fsm_context.clone(),
            should_debug,
        }
    }

    /// Returns a new pass context decorated with `self` as the parent context.
    /// Used when passing the context down to a subgraph.
    /// Allows optionally specifying a state id if the subgraph is part of a state machine
    pub fn child_with_state(
        &'a self,
        fsm_ctx: Option<FsmContext<'a>>,
        overlay: &'a InputOverlay,
    ) -> Self {
        let node_ctx = self.node_context.unwrap();
        let node = node_ctx.graph.nodes.get(node_ctx.node_id).unwrap();
        let graph_id = match &node.node {
            crate::core::prelude::AnimationNodeType::Graph(n) => n.graph.id(),
            crate::core::prelude::AnimationNodeType::Fsm(n) => {
                // TODO: Extract this into a function, probably(?) in the FSM code
                let cur_state_id = fsm_ctx
                    .as_ref()
                    .map(|t| t.state_stack.last_state())
                    .unwrap();
                let fsm = self
                    .resources
                    .state_machine_assets
                    .get(&n.fsm)
                    .unwrap()
                    .get_low_level_fsm();
                let cur_state = fsm.states.get(&cur_state_id).unwrap();
                cur_state.graph.id()
            }
            _ => panic!("Only graph or FSM nodes can have subgraphs"),
        };

        let subctx_id = SubContextId {
            ctx_id: self.context_id,
            node_id: self.node_context.unwrap().node_id.to_owned(),
            state_id: fsm_ctx.clone().map(|t| t.state_stack.last_state()),
        };

        Self {
            context_id: self
                .context_arena
                .as_mut()
                .get_sub_context_or_insert_default(subctx_id, graph_id),
            context_arena: self.context_arena.clone(),
            resources: self.resources,
            overlay,
            node_context: self.node_context,
            parent: Some(self.into()),
            root_entity: self.root_entity,
            entity_map: self.entity_map,
            deferred_gizmos: self.deferred_gizmos.clone(),
            temp_cache: self.temp_cache,
            should_debug: self.should_debug,
            fsm_context: fsm_ctx,
        }
    }

    /// Returns a new pass context decorated with `self` as the parent context.
    /// Used when passing the context down to a subgraph.
    pub fn child(&'a self, overlay: &'a InputOverlay) -> Self {
        self.child_with_state(None, overlay)
    }

    /// Returns a new pass context with the given temp cache value.
    pub fn with_temp(&self, temp_cache: bool) -> Self {
        Self {
            context_id: self.context_id,
            context_arena: self.context_arena.clone(),
            resources: self.resources,
            overlay: self.overlay,
            node_context: self.node_context,
            parent: self.parent.clone(),
            root_entity: self.root_entity,
            entity_map: self.entity_map,
            deferred_gizmos: self.deferred_gizmos.clone(),
            should_debug: self.should_debug,
            temp_cache,
            fsm_context: self.fsm_context.clone(),
        }
    }

    /// Access the parent pass context.
    pub fn parent(&'a self) -> Self {
        self.parent.as_ref().unwrap().as_ref()
    }

    /// Verify whether the current context has a parent. Should be true when inside a subgraph and
    /// false otherwise
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    /// Return a mutable reference to the `GraphContext`
    pub fn context_mut(&mut self) -> &mut GraphContext {
        self.context_arena
            .as_mut()
            .get_context_mut(self.context_id)
            .unwrap()
    }

    /// Return a reference to the `GraphContext`
    pub fn context(&self) -> &GraphContext {
        self.context_arena
            .as_ref()
            .get_context(self.context_id)
            .unwrap()
    }

    pub fn spec_context(&'a self) -> SpecContext<'a> {
        SpecContext {
            graph_assets: &self.resources.animation_graph_assets,
            fsm_assets: &self.resources.state_machine_assets,
        }
    }

    pub fn caches(&self) -> &GraphStateStack {
        &self.context().caches
    }

    pub fn caches_mut(&mut self) -> &mut GraphStateStack {
        &mut self.context_mut().caches
    }

    pub fn str_ctx_stack(&self) -> String {
        let mut s = if let Some(fsm_ctx) = &self.fsm_context {
            format!(
                "- [FSM] {}: {}",
                "state_id",
                fsm_ctx.state_stack.last_state()
            )
        } else if let Some(node_ctx) = &self.node_context {
            format!("- [Node] {}: {}", "node_id", node_ctx.node_id)
        } else {
            "- [Graph]".to_string()
        };

        if self.has_parent() {
            s = format!("{}\n{}", s, self.parent().str_ctx_stack());
        }

        s
    }

    pub fn node_id(&self) -> NodeId {
        self.node_context.as_ref().unwrap().node_id.clone()
    }
}

impl<'a> PassContext<'a> {
    /// Request an input parameter from the graph
    pub fn data_back(&mut self, pin_id: impl Into<PinId>) -> Result<DataValue, GraphError> {
        let node_ctx = self.node_context.unwrap();
        let target_pin = TargetPin::NodeData(node_ctx.node_id.clone(), pin_id.into());
        node_ctx.graph.get_data(target_pin, self.without_node())
    }

    /// Request an input data value from the "parent" of the current graph. The parent could either be a state machine
    /// or another graph (depending if the current graph is a FSM state or a graph node)
    pub fn parent_data_back(&mut self, pin_id: impl Into<PinId>) -> Result<DataValue, GraphError> {
        if let Some(fsm_ctx) = &self.fsm_context {
            let pin_id = pin_id.into();
            fsm_ctx.fsm.get_data(
                fsm_ctx.state_stack.clone(),
                TargetPin::OutputData(pin_id),
                self.parent(),
            )
        } else if self.has_parent() {
            self.parent().data_back(pin_id)
        } else {
            Err(GraphError::MissingParentGraph)
        }
    }

    /// Sets the output value at the given pin for the current node. It's up to the caller to
    /// verify the types are correct, or suffer the consequences.
    pub fn set_data_fwd(&mut self, pin_id: impl Into<PinId>, data: impl Into<DataValue>) {
        let node_ctx = self.node_context.unwrap();
        let temp_cache = self.temp_cache;
        self.caches_mut().set(
            move |c| {
                c.set_parameter(
                    SourcePin::NodeData(node_ctx.node_id.clone(), pin_id.into()),
                    data.into(),
                )
            },
            CacheWriteFilter::for_temp(temp_cache),
        );
    }

    /// Request the duration of an input pose pin.
    pub fn duration_back(&mut self, pin_id: impl Into<PinId>) -> Result<DurationData, GraphError> {
        let node_ctx = self.node_context.unwrap();
        let target_pin = TargetPin::NodeTime(node_ctx.node_id.clone(), pin_id.into());
        node_ctx.graph.get_duration(target_pin, self.without_node())
    }

    /// Sets the duration of the current node with current settings.
    pub fn set_duration_fwd(&mut self, duration: DurationData) {
        let node_ctx = self.node_context.unwrap();
        let temp_cache = self.temp_cache;
        self.caches_mut().set(
            move |c| c.set_duration(SourcePin::NodeTime(node_ctx.node_id.clone()), duration),
            CacheWriteFilter::for_temp(temp_cache),
        );
    }

    /// Sets the duration of the current node with current settings.
    pub fn set_time_update_back(&mut self, pin_id: impl Into<PinId>, time_update: TimeUpdate) {
        let node_ctx = self.node_context.unwrap();
        let temp_cache = self.temp_cache;
        self.caches_mut().set(
            move |c| {
                c.set_time_update_back(
                    TargetPin::NodeTime(node_ctx.node_id.clone(), pin_id.into()),
                    time_update,
                )
            },
            CacheWriteFilter::for_temp(temp_cache),
        );
    }

    /// Sets the time state of the current node.
    pub fn set_time(&mut self, time: f32) {
        let node_ctx = self.node_context.unwrap();
        let temp_cache = self.temp_cache;
        self.caches_mut().set(
            |c| c.set_time(SourcePin::NodeTime(node_ctx.node_id.clone()), time),
            CacheWriteFilter::for_temp(temp_cache),
        );
    }

    /// Request the cached time update query from the current frame
    pub fn time_update_fwd(&self) -> Result<TimeUpdate, GraphError> {
        let node_ctx = self.node_context.unwrap();
        let source_pin = SourcePin::NodeTime(node_ctx.node_id.clone());
        node_ctx
            .graph
            .get_time_update(source_pin, self.without_node())
    }

    pub fn parent_time_update_fwd(&self) -> Result<TimeUpdate, GraphError> {
        if let Some(fsm_ctx) = &self.fsm_context {
            fsm_ctx.fsm.get_time_update(
                fsm_ctx.state_stack.clone(),
                TargetPin::OutputTime,
                self.parent(),
            )
        } else if self.has_parent() {
            self.parent().time_update_fwd()
        } else {
            Err(GraphError::MissingParentGraph)
        }
    }

    /// Request the cached timestamp of the output animation in the last frame
    pub fn prev_time(&self) -> f32 {
        let node_ctx = self.node_context.unwrap();
        let source_pin = SourcePin::NodeTime(node_ctx.node_id.clone());
        self.caches()
            .get(|c| c.get_prev_time(&source_pin), CacheReadFilter::PRIMARY)
            .unwrap_or(0.)
    }

    /// Request the cached timestamp of the output animation in the last frame
    pub fn time(&self) -> f32 {
        let node_ctx = self.node_context.unwrap();
        let source_pin = SourcePin::NodeTime(node_ctx.node_id.clone());
        self.caches()
            .get(|c| c.get_time(&source_pin), CacheReadFilter::PRIMARY)
            .unwrap_or(0.)
    }
}

#[derive(Clone)]
pub struct PassContextRef<'a> {
    ctx: *const PassContext<'a>,
}

impl<'a> From<&'a PassContext<'a>> for PassContextRef<'a> {
    fn from(value: &'a PassContext) -> Self {
        Self { ctx: value }
    }
}

impl<'a> PassContextRef<'a> {
    pub fn as_ref(&self) -> PassContext<'a> {
        unsafe { self.ctx.as_ref().unwrap().clone() }
    }
}

#[derive(Clone)]
pub struct GraphContextArenaRef {
    context: *mut GraphContextArena,
}

impl From<&mut GraphContextArena> for GraphContextArenaRef {
    fn from(value: &mut GraphContextArena) -> Self {
        Self { context: value }
    }
}

impl GraphContextArenaRef {
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut GraphContextArena {
        unsafe { self.context.as_mut().unwrap() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_ref(&self) -> &GraphContextArena {
        unsafe { self.context.as_ref().unwrap() }
    }
}
