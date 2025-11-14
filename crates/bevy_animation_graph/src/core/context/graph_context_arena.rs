use crate::{
    core::{
        animation_graph::NodeId, prelude::AnimationGraph, state_machine::low_level::LowLevelStateId,
    },
    prelude::graph_context::GraphContext,
};
use bevy::{asset::AssetId, platform::collections::HashMap, reflect::Reflect};

#[derive(Reflect, Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct GraphContextId(usize);

#[derive(Reflect, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SubContextId {
    pub ctx_id: GraphContextId,
    pub node_id: NodeId,
    pub state_id: Option<LowLevelStateId>,
}

#[derive(Reflect, Debug)]
pub struct GraphContextArena {
    contexts: Vec<GraphContext>,
    hierarchy: HashMap<SubContextId, GraphContextId>,
    top_level_context: GraphContextId,
}

impl GraphContextArena {
    pub fn new(graph_id: AssetId<AnimationGraph>) -> Self {
        Self {
            contexts: vec![GraphContext::new(graph_id)],
            hierarchy: HashMap::default(),
            top_level_context: GraphContextId(0),
        }
    }

    pub fn iter_context_ids(&self) -> impl Iterator<Item = GraphContextId> {
        (0..self.contexts.len()).map(GraphContextId)
    }

    fn new_context(&mut self, graph_id: AssetId<AnimationGraph>) -> GraphContextId {
        self.contexts.push(GraphContext::new(graph_id));

        GraphContextId(self.contexts.len() - 1)
    }

    pub fn get_context(&self, id: GraphContextId) -> Option<&GraphContext> {
        self.contexts.get(id.0)
    }

    pub fn next_frame(&mut self) {
        for context in self.contexts.iter_mut() {
            context.next_frame();
        }
    }

    pub fn get_context_mut(&mut self, id: GraphContextId) -> Option<&mut GraphContext> {
        self.contexts.get_mut(id.0)
    }

    pub fn get_toplevel(&self) -> &GraphContext {
        self.get_context(self.get_toplevel_id()).unwrap()
    }

    pub fn get_toplevel_mut(&mut self) -> &mut GraphContext {
        self.get_context_mut(self.get_toplevel_id()).unwrap()
    }

    pub fn get_toplevel_id(&self) -> GraphContextId {
        self.top_level_context
    }

    pub fn context_exists(&self, id: GraphContextId) -> bool {
        id.0 < self.contexts.len()
    }

    pub(super) fn get_sub_context_or_insert_default(
        &mut self,
        subctx_id: SubContextId,
        subgraph_id: AssetId<AnimationGraph>,
    ) -> GraphContextId {
        if !self.context_exists(subctx_id.ctx_id) {
            panic!("Context does not exist");
        }

        if !self.hierarchy.contains_key(&subctx_id) {
            let child_node_id = self.new_context(subgraph_id);
            self.hierarchy.insert(subctx_id.clone(), child_node_id);
        }

        *self.hierarchy.get(&subctx_id).unwrap()
    }
}
