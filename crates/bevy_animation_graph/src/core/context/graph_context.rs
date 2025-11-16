use crate::{
    core::{animation_graph::TimeUpdate, prelude::AnimationGraph},
    prelude::{
        node_caches::NodeCaches,
        node_states::{NodeStates, StateKey},
    },
};
use bevy::{asset::AssetId, platform::collections::HashMap, reflect::prelude::*};

#[derive(Debug, Reflect)]
pub struct GraphState {
    pub node_states: NodeStates,
    pub node_caches: NodeCaches,
    pub query_output_time: QueryOutputTime,
    graph_id: AssetId<AnimationGraph>,
}

impl GraphState {
    pub fn new(graph_id: AssetId<AnimationGraph>) -> Self {
        Self {
            graph_id,
            node_states: NodeStates::default(),
            node_caches: NodeCaches::default(),
            query_output_time: QueryOutputTime::None,
        }
    }

    pub fn next_frame(&mut self) {
        self.node_states.next_frame();
        self.node_caches.next_frame();
    }

    pub fn get_graph_id(&self) -> AssetId<AnimationGraph> {
        self.graph_id
    }
}

#[derive(Debug, Reflect)]
pub enum QueryOutputTime {
    None,
    Forced(TimeUpdate),
    ByKey(HashMap<StateKey, TimeUpdate>),
}

impl QueryOutputTime {
    pub fn from_key(key: StateKey, update: TimeUpdate) -> Self {
        Self::ByKey([(key, update)].into())
    }

    pub fn get(&self, key: StateKey) -> Option<TimeUpdate> {
        match self {
            QueryOutputTime::None => None,
            QueryOutputTime::Forced(time_update) => Some(time_update.clone()),
            QueryOutputTime::ByKey(hash_map) => hash_map.get(&key).cloned(),
        }
    }
}
