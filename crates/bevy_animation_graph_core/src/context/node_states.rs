use std::any::Any;

use bevy::{
    platform::collections::HashMap,
    reflect::{Reflect, Reflectable},
};
use uuid::Uuid;

use crate::{animation_graph::NodeId, context::node_state_box::NodeStateBox, errors::GraphError};

pub trait GraphStateType: Reflect + Any + std::fmt::Debug + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn GraphStateType>;
}

impl<T> GraphStateType for T
where
    T: Clone + Reflectable + Any + std::fmt::Debug + Send + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn GraphStateType> {
        Box::new(self.clone())
    }
}

#[derive(Reflect, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum StateKey {
    #[default]
    Default,
    Temporary(Uuid),
}

#[derive(Default, Debug, Reflect)]
pub struct NodeState {
    last_state: Option<NodeStateBox>,
    upcoming_state: HashMap<StateKey, NodeStateBox>,

    /// Most nodes need to keep track of time. We handle this separately
    /// to avoid overhead of Box if possible
    last_time: f32,
    upcoming_time: HashMap<StateKey, f32>,
}

impl NodeState {
    pub fn next_frame(&mut self) {
        if let Some(next_state) = self.upcoming_state.get(&StateKey::Default) {
            self.last_state = Some(next_state.clone());
        }

        if let Some(next_time) = self.upcoming_time.get(&StateKey::Default) {
            self.last_time = *next_time;
        }

        self.upcoming_state.clear();
        self.upcoming_time.clear();
    }

    pub fn get_all_upcoming_states<T: GraphStateType>(
        &self,
    ) -> Result<impl Iterator<Item = &T>, GraphError> {
        Ok(self.upcoming_state.values().filter_map(|s| {
            let v: &dyn Any = s.value.as_ref();
            v.downcast_ref::<T>()
        }))
    }

    pub fn get_state<T: GraphStateType>(&self, key: StateKey) -> Result<&T, GraphError> {
        self.upcoming_state
            .get(&key)
            .or(self.last_state.as_ref())
            .ok_or(GraphError::MissingStateValue)
            .and_then(|v| {
                let v: &dyn Any = v.value.as_ref();
                v.downcast_ref::<T>().ok_or(GraphError::MismatchedStateType)
            })
    }

    pub fn get_mut_or_insert_with<T: GraphStateType>(
        &mut self,
        key: StateKey,
        default: impl FnOnce() -> T,
    ) -> Result<&mut T, GraphError> {
        let NodeState {
            last_state,
            upcoming_state,
            ..
        } = self;
        let dyn_mut: &mut dyn Any = upcoming_state
            .entry(key)
            .or_insert_with(|| {
                last_state
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| NodeStateBox {
                        value: Box::new(default()),
                    })
            })
            .value
            .as_mut();

        dyn_mut
            .downcast_mut::<T>()
            .ok_or(GraphError::MismatchedStateType)
    }

    pub fn get_time(&self, key: StateKey) -> f32 {
        self.upcoming_time
            .get(&key)
            .copied()
            .unwrap_or(self.last_time)
    }

    pub fn set_time(&mut self, key: StateKey, time: f32) {
        self.upcoming_time.insert(key, time);
    }

    pub fn get_last_time(&self) -> f32 {
        self.last_time
    }
}

#[derive(Debug, Reflect, Default)]
pub struct NodeStates {
    states: HashMap<NodeId, NodeState>,
}

impl NodeStates {
    pub fn next_frame(&mut self) {
        for node_state in self.states.values_mut() {
            node_state.next_frame();
        }
    }

    pub fn get_all_upcoming_states<T: GraphStateType>(
        &self,
        node_id: NodeId,
    ) -> Result<impl Iterator<Item = &T>, GraphError> {
        self.states
            .get(&node_id)
            .ok_or(GraphError::MissingStateValue)
            .and_then(|n| n.get_all_upcoming_states())
    }

    pub fn get<T: GraphStateType>(&self, node_id: NodeId, key: StateKey) -> Result<&T, GraphError> {
        self.states
            .get(&node_id)
            .ok_or(GraphError::MissingStateValue)
            .and_then(|n| n.get_state(key))
    }

    pub fn get_mut_or_insert_with<T: GraphStateType>(
        &mut self,
        node_id: NodeId,
        key: StateKey,
        default: impl FnOnce() -> T,
    ) -> Result<&mut T, GraphError> {
        self.states
            .entry(node_id)
            .or_default()
            .get_mut_or_insert_with(key, default)
    }

    pub fn get_time(&self, node_id: NodeId, key: StateKey) -> f32 {
        self.states
            .get(&node_id)
            .map(|n| n.get_time(key))
            .unwrap_or(0.)
    }

    pub fn set_time(&mut self, node_id: NodeId, key: StateKey, time: f32) {
        self.states.entry(node_id).or_default().set_time(key, time);
    }

    pub fn get_last_time(&self, node_id: NodeId) -> f32 {
        self.states
            .get(&node_id)
            .map(|n| n.get_last_time())
            .unwrap_or(0.)
    }
}
