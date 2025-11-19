use bevy::{
    platform::collections::{HashMap, HashSet},
    reflect::Reflect,
};

use crate::core::{
    animation_graph::{NodeId, PinId, SourcePin, TargetPin, TimeUpdate},
    context::node_states::StateKey,
    duration_data::DurationData,
    edge_data::DataValue,
    errors::GraphError,
};

#[derive(Reflect, Default, Debug)]
pub struct NodeCache {
    pub output_data: HashMap<(StateKey, PinId), DataValue>,
    /// Time update coming from the "output time" pin. Perhaps should be called "input time
    /// update".
    pub output_time_update: HashMap<StateKey, TimeUpdate>,
    /// Time updates sent back to nodes via "input time" pins
    pub input_time_updates: HashMap<(StateKey, PinId), TimeUpdate>,
    pub duration: HashMap<StateKey, DurationData>,
    pub updated: HashSet<StateKey>,
}

#[derive(Reflect, Default, Debug)]
pub struct NodeCaches {
    caches: HashMap<NodeId, NodeCache>,
}

impl NodeCaches {
    pub fn next_frame(&mut self) {
        self.caches.clear();
    }

    pub fn get_duration(&self, node_id: NodeId, key: StateKey) -> Result<DurationData, GraphError> {
        let error = || GraphError::DurationMissing(SourcePin::NodeTime(node_id.clone()));

        self.caches
            .get(&node_id)
            .ok_or_else(&error)
            .and_then(|c| c.duration.get(&key).ok_or_else(&error))
            .cloned()
    }

    pub fn set_duration(&mut self, node_id: NodeId, key: StateKey, duration: DurationData) {
        self.cache_mut(node_id).duration.insert(key, duration);
    }

    pub fn get_output_data(
        &self,
        node_id: NodeId,
        key: StateKey,
        pin: PinId,
    ) -> Result<DataValue, GraphError> {
        let error = || GraphError::OutputMissing(SourcePin::NodeData(node_id.clone(), pin.clone()));

        self.caches
            .get(&node_id)
            .ok_or_else(&error)
            .and_then(|c| c.output_data.get(&(key, pin.clone())).ok_or_else(&error))
            .cloned()
    }

    pub fn set_output_data(&mut self, node_id: NodeId, key: StateKey, pin: PinId, data: DataValue) {
        self.cache_mut(node_id).output_data.insert((key, pin), data);
    }

    pub fn get_output_time_update(
        &self,
        node_id: NodeId,
        key: StateKey,
    ) -> Result<TimeUpdate, GraphError> {
        let error = || GraphError::TimeUpdateMissingFwd(SourcePin::NodeTime(node_id.clone()));

        self.caches
            .get(&node_id)
            .ok_or_else(&error)
            .and_then(|c| c.output_time_update.get(&key).ok_or_else(&error))
            .cloned()
    }

    pub fn set_output_time_update(&mut self, node_id: NodeId, key: StateKey, update: TimeUpdate) {
        self.cache_mut(node_id)
            .output_time_update
            .insert(key, update);
    }

    pub fn get_input_time_update(
        &self,
        node_id: NodeId,
        key: StateKey,
        pin: PinId,
    ) -> Result<TimeUpdate, GraphError> {
        let error =
            || GraphError::TimeUpdateMissingBack(TargetPin::NodeTime(node_id.clone(), pin.clone()));

        self.caches
            .get(&node_id)
            .ok_or_else(&error)
            .and_then(|c| {
                c.input_time_updates
                    .get(&(key, pin.clone()))
                    .ok_or_else(&error)
            })
            .cloned()
    }

    pub fn set_input_time_update(
        &mut self,
        node_id: NodeId,
        key: StateKey,
        pin: PinId,
        update: TimeUpdate,
    ) {
        self.cache_mut(node_id)
            .input_time_updates
            .insert((key, pin), update);
    }

    pub fn is_updated(&self, node_id: NodeId, key: StateKey) -> bool {
        self.caches
            .get(&node_id)
            .is_some_and(|c| c.updated.contains(&key))
    }

    pub fn mark_updated(&mut self, node_id: NodeId, key: StateKey) {
        self.cache_mut(node_id).updated.insert(key);
    }

    fn cache_mut(&mut self, node_id: NodeId) -> &mut NodeCache {
        self.caches.entry(node_id).or_default()
    }
}
