use bevy::{
    asset::Assets,
    platform::collections::{HashMap, HashSet},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation_graph::{AnimationGraph, GraphInputPin, PinId},
    edge_data::DataSpec,
    state_machine::high_level::StateMachine,
};

pub struct SpecContext<'a> {
    resources: SpecResources<'a>,
    node_spec: &'a mut NodeSpec,
}

impl<'a> SpecContext<'a> {
    pub fn new(resources: SpecResources<'a>, node_spec: &'a mut NodeSpec) -> Self {
        Self {
            resources,
            node_spec,
        }
    }

    pub fn update_from_node_spec(&mut self, node_spec: &NodeSpec) -> &mut Self {
        self.node_spec.update_from_node_spec(node_spec);
        self
    }

    pub fn add_input_time(&mut self, pin_id: impl Into<PinId>) -> &mut Self {
        self.node_spec.add_input_time(pin_id.into());
        self
    }

    pub fn add_input_data(&mut self, pin_id: impl Into<PinId>, data_spec: DataSpec) -> &mut Self {
        self.node_spec.add_input_data(pin_id.into(), data_spec);
        self
    }

    pub fn add_output_time(&mut self) -> &mut Self {
        self.node_spec.add_output_time();
        self
    }

    pub fn add_output_data(&mut self, pin_id: impl Into<PinId>, data_spec: DataSpec) -> &mut Self {
        self.node_spec.add_output_data(pin_id.into(), data_spec);
        self
    }

    pub fn resources(&self) -> SpecResources<'a> {
        self.resources
    }
}

#[derive(Clone, Copy)]
pub struct SpecResources<'a> {
    pub graph_assets: &'a Assets<AnimationGraph>,
    pub fsm_assets: &'a Assets<StateMachine>,
}

pub type NodeSpec = IoSpec<PinId>;
pub type GraphSpec = IoSpec<GraphInputPin>;

#[derive(Reflect, Clone, Debug, PartialEq, Eq)]
pub enum NodeInput<I> {
    Time(I),
    Data(I, DataSpec),
}

impl<I: Default> Default for NodeInput<I> {
    fn default() -> Self {
        Self::Data(I::default(), DataSpec::default())
    }
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq)]
pub enum NodeOutput {
    Time,
    Data(PinId, DataSpec),
}

impl Default for NodeOutput {
    fn default() -> Self {
        Self::Data(PinId::default(), DataSpec::default())
    }
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeInputPin<I> {
    Time(I),
    Data(I),
}

impl<I> From<NodeInput<I>> for NodeInputPin<I> {
    fn from(value: NodeInput<I>) -> Self {
        match value {
            NodeInput::Time(pin_id) => NodeInputPin::Time(pin_id),
            NodeInput::Data(pin_id, _) => NodeInputPin::Data(pin_id),
        }
    }
}

impl<I: Clone> From<&NodeInput<I>> for NodeInputPin<I> {
    fn from(value: &NodeInput<I>) -> Self {
        match value {
            NodeInput::Time(pin_id) => NodeInputPin::Time(pin_id.clone()),
            NodeInput::Data(pin_id, _) => NodeInputPin::Data(pin_id.clone()),
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeOutputPin {
    Time,
    Data(PinId),
}

impl From<NodeOutput> for NodeOutputPin {
    fn from(value: NodeOutput) -> Self {
        match value {
            NodeOutput::Time => NodeOutputPin::Time,
            NodeOutput::Data(pin_id, _) => NodeOutputPin::Data(pin_id),
        }
    }
}

impl From<&NodeOutput> for NodeOutputPin {
    fn from(value: &NodeOutput) -> Self {
        match value {
            NodeOutput::Time => NodeOutputPin::Time,
            NodeOutput::Data(pin_id, _) => NodeOutputPin::Data(pin_id.clone()),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct IoSpec<I> {
    input_times: HashSet<I>,
    input_data: HashMap<I, DataSpec>,
    output_time: bool,
    output_data: HashMap<PinId, DataSpec>,

    input_order: HashMap<NodeInputPin<I>, i32>,
    output_order: HashMap<NodeOutputPin, i32>,

    next_input_order: i32,
    next_output_order: i32,
}

impl<I> Default for IoSpec<I> {
    fn default() -> Self {
        Self {
            input_times: Default::default(),
            input_data: Default::default(),
            output_time: Default::default(),
            output_data: Default::default(),
            input_order: Default::default(),
            output_order: Default::default(),
            next_input_order: Default::default(),
            next_output_order: Default::default(),
        }
    }
}

impl<I> IoSpec<I>
where
    I: Clone + Eq + std::hash::Hash,
{
    pub fn add_input_time(&mut self, pin_id: I) {
        self.input_times.insert(pin_id.clone());
        self.input_order
            .insert(NodeInputPin::Time(pin_id), self.next_input_order);
        self.next_input_order += 1;
    }

    pub fn add_input_data(&mut self, pin_id: I, data_spec: DataSpec) {
        self.input_data.insert(pin_id.clone(), data_spec);
        self.input_order
            .insert(NodeInputPin::Data(pin_id), self.next_input_order);
        self.next_input_order += 1;
    }

    pub fn add_output_time(&mut self) {
        self.output_time = true;
        self.output_order
            .insert(NodeOutputPin::Time, self.next_output_order);
        self.next_output_order += 1;
    }

    pub fn add_output_data(&mut self, pin_id: PinId, data_spec: DataSpec) {
        self.output_data.insert(pin_id.clone(), data_spec);
        self.output_order
            .insert(NodeOutputPin::Data(pin_id), self.next_output_order);
        self.next_output_order += 1;
    }

    pub fn input_compare_key(&self, input: &NodeInputPin<I>) -> i32 {
        self.input_order.get(input).copied().unwrap_or(i32::MAX)
    }

    pub fn output_compare_key(&self, output: &NodeOutputPin) -> i32 {
        self.output_order.get(output).copied().unwrap_or(i32::MAX)
    }

    pub fn get_input_data(&self, pin_id: &I) -> Option<DataSpec> {
        self.input_data.get(pin_id).copied()
    }

    pub fn has_input_time(&self, pin_id: &I) -> bool {
        self.input_times.contains(pin_id)
    }

    pub fn get_output_data(&self, pin_id: &PinId) -> Option<DataSpec> {
        self.output_data.get(pin_id).copied()
    }

    pub fn has_output_time(&self) -> bool {
        self.output_time
    }

    pub fn update_from_node_spec(&mut self, other: &IoSpec<I>) {
        for input in other.sorted_inputs() {
            match input {
                NodeInput::Time(key) => self.add_input_time(key),
                NodeInput::Data(key, data_spec) => self.add_input_data(key, data_spec),
            }
        }

        for output in other.sorted_outputs() {
            match output {
                NodeOutput::Time => self.add_output_time(),
                NodeOutput::Data(key, data_spec) => self.add_output_data(key, data_spec),
            }
        }
    }

    /// Unsorted iterator over input data
    pub fn iter_input_data(&self) -> impl Iterator<Item = (&I, &DataSpec)> {
        self.input_data.iter()
    }

    /// Unsorted iterator over input times
    pub fn iter_input_times(&self) -> impl Iterator<Item = &I> {
        self.input_times.iter()
    }

    /// Unsorted iterator over output data
    pub fn iter_output_data(&self) -> impl Iterator<Item = (&PinId, &DataSpec)> {
        self.output_data.iter()
    }

    /// Sorted vector of inputs
    pub fn sorted_inputs(&self) -> Vec<NodeInput<I>>
    where
        I: Clone + Eq + std::hash::Hash,
    {
        let mut inputs: Vec<_> = self
            .input_times
            .iter()
            .map(|pin_id| NodeInput::Time(pin_id.clone()))
            .chain(
                self.input_data
                    .iter()
                    .map(|(pin_id, data_spec)| NodeInput::Data(pin_id.clone(), *data_spec)),
            )
            .collect();

        inputs.sort_by_key(|val| self.input_order.get(&NodeInputPin::from(val.clone())));
        inputs
    }

    /// Sorted vector of inputs
    pub fn sorted_outputs(&self) -> Vec<NodeOutput> {
        let mut outputs: Vec<_> = self
            .output_time
            .then_some(NodeOutput::Time)
            .into_iter()
            .chain(
                self.output_data
                    .iter()
                    .map(|(pin_id, data_spec)| NodeOutput::Data(pin_id.clone(), *data_spec)),
            )
            .collect();

        outputs.sort_by_key(|val| self.output_order.get(&NodeOutputPin::from(val.clone())));
        outputs
    }

    pub fn len_input(&self) -> usize {
        self.input_order.len()
    }

    pub fn len_output(&self) -> usize {
        self.output_order.len()
    }

    pub fn shift_input_index(&mut self, key: &NodeInputPin<I>, idx_delta: i32)
    where
        I: Clone + Eq + std::hash::Hash,
    {
        let Some(&current_idx) = self.input_order.get(key) else {
            return;
        };

        let target_idx = current_idx + idx_delta;

        for idx in self.input_order.values_mut() {
            if *idx == target_idx {
                *idx = current_idx;
            }
        }

        let Some(idx) = self.input_order.get_mut(key) else {
            return;
        };
        *idx = target_idx;
        self.reset_ordering();
    }

    pub fn shift_output_index(&mut self, key: &NodeOutputPin, idx_delta: i32)
    where
        I: Clone + std::hash::Hash + Eq,
    {
        let Some(&current_idx) = self.output_order.get(key) else {
            return;
        };

        let target_idx = current_idx + idx_delta;

        for idx in self.output_order.values_mut() {
            if *idx == target_idx {
                *idx = current_idx;
            }
        }

        let Some(idx) = self.output_order.get_mut(key) else {
            return;
        };
        *idx = target_idx;
        self.reset_ordering();
    }

    /// Returns false if the update could not be completed (e.g. if the new key already
    /// exists!)
    pub fn update_input(&mut self, prev_key: &NodeInputPin<I>, new_input: NodeInput<I>) -> bool
    where
        I: Clone + Eq + std::hash::Hash,
    {
        let new_key = NodeInputPin::from(new_input.clone());

        if &new_key != prev_key && self.input_order.contains_key(&new_key) {
            return false;
        }

        let Some(idx) = self.input_order.remove(prev_key) else {
            return false;
        };

        match prev_key {
            NodeInputPin::Time(input) => {
                self.input_times.remove(input);
            }
            NodeInputPin::Data(input) => {
                self.input_data.remove(input);
            }
        }

        self.input_order.insert(new_key, idx);

        match new_input {
            NodeInput::Time(input) => {
                self.input_times.insert(input);
            }
            NodeInput::Data(input, data_spec) => {
                self.input_data.insert(input, data_spec);
            }
        }

        true
    }

    pub fn remove_input(&mut self, key: &NodeInputPin<I>)
    where
        I: Clone + Eq + std::hash::Hash,
    {
        self.input_order.remove(key);
        match key {
            NodeInputPin::Time(input) => {
                self.input_times.remove(input);
            }
            NodeInputPin::Data(input) => {
                self.input_data.remove(input);
            }
        }
        self.reset_ordering();
    }

    pub fn remove_output(&mut self, key: &NodeOutputPin)
    where
        I: Clone + Eq + std::hash::Hash,
    {
        self.output_order.remove(key);
        match key {
            NodeOutputPin::Time => {
                self.output_time = false;
            }
            NodeOutputPin::Data(output) => {
                self.output_data.remove(output);
            }
        }
        self.reset_ordering();
    }

    /// Returns false if the update could not be completed (e.g. if the new key already
    /// exists!)
    pub fn update_output(&mut self, prev_key: &NodeOutputPin, new_output: NodeOutput) -> bool {
        let new_key = NodeOutputPin::from(new_output.clone());

        if &new_key != prev_key && self.output_order.contains_key(&new_key) {
            return false;
        }

        let Some(idx) = self.output_order.remove(prev_key) else {
            return false;
        };

        self.output_order.insert(new_key, idx);

        match prev_key {
            NodeOutputPin::Time => {
                self.output_time = false;
            }
            NodeOutputPin::Data(output) => {
                self.output_data.remove(output);
            }
        }

        match new_output {
            NodeOutput::Time => {
                self.output_time = true;
            }
            NodeOutput::Data(output, data_spec) => {
                self.output_data.insert(output, data_spec);
            }
        }

        true
    }

    pub fn reset_ordering(&mut self)
    where
        I: Clone + std::hash::Hash + Eq,
    {
        let mut sorted_inputs: Vec<_> = self.input_order.iter().collect();
        sorted_inputs.sort_by_key(|(_, idx)| **idx);
        self.input_order = sorted_inputs
            .into_iter()
            .enumerate()
            .map(|(idx, (key, _))| (key.clone(), idx as i32))
            .collect();
        self.next_input_order = self.input_order.len() as i32;

        let mut sorted_outputs: Vec<_> = self.output_order.iter().collect();
        sorted_outputs.sort_by_key(|(_, idx)| **idx);
        self.output_order = sorted_outputs
            .into_iter()
            .enumerate()
            .map(|(idx, (key, _))| (key.clone(), idx as i32))
            .collect();
        self.next_output_order = self.output_order.len() as i32;
    }
}

impl<I: Clone + Serialize + Eq + std::hash::Hash> Serialize for IoSpec<I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        IoSpecSerial {
            input_times: self.input_times.clone(),
            input_data: self.input_data.clone(),
            output_time: self.output_time,
            output_data: self.output_data.clone(),
            input_order: self.input_order.clone(),
            output_order: self.output_order.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de, I: Deserialize<'de> + Eq + std::hash::Hash> Deserialize<'de> for IoSpec<I> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serial = IoSpecSerial::deserialize(deserializer)?;

        let next_input_order = serial.input_order.values().max().copied().unwrap_or(-1) + 1;
        let next_output_order = serial.output_order.values().max().copied().unwrap_or(-1) + 1;

        Ok(Self {
            input_times: serial.input_times,
            input_data: serial.input_data,
            output_time: serial.output_time,
            output_data: serial.output_data,
            input_order: serial.input_order,
            output_order: serial.output_order,
            next_input_order,
            next_output_order,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct IoSpecSerial<I: Eq + std::hash::Hash> {
    input_times: HashSet<I>,
    input_data: HashMap<I, DataSpec>,
    output_time: bool,
    output_data: HashMap<PinId, DataSpec>,

    input_order: HashMap<NodeInputPin<I>, i32>,
    output_order: HashMap<NodeOutputPin, i32>,
}

#[derive(Reflect, Debug, Clone)]
pub struct SpecEntry<K, V> {
    key: K,
    value: V,
}

#[derive(Reflect, Debug, Clone)]
pub struct DataOnlySpec<K, V> {
    entries: Vec<(K, V)>,
    index: HashMap<K, usize>,
}

impl<K, V> Default for DataOnlySpec<K, V> {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            index: Default::default(),
        }
    }
}

impl<K, V> DataOnlySpec<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    pub fn push(&mut self, key: K, value: V) -> bool {
        if self.index.contains_key(&key) {
            return false;
        }
        let entry = (key.clone(), value);
        self.entries.push(entry);
        self.index.insert(key, self.entries.len() - 1);
        true
    }

    pub fn input_compare_key(&self, key: &K) -> i32 {
        self.index.get(key).copied().map_or(i32::MAX, |i| i as i32)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).map(|i| &self.entries[*i].1)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|entry| &entry.0)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|entry| &entry.1)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn shift_key(&mut self, key: &K, idx_delta: i32)
    where
        K: Clone + Eq + std::hash::Hash,
    {
        let Some(&current_idx) = self.index.get(key) else {
            return;
        };

        self.shift_index(current_idx, idx_delta);
    }

    pub fn shift_index(&mut self, current_idx: usize, idx_delta: i32) -> usize {
        let mut target_idx = (current_idx as i32 + idx_delta) as usize;
        if target_idx > current_idx {
            target_idx -= 1;
        }

        let current_entry = self.entries.remove(current_idx);
        self.entries.insert(target_idx, current_entry);

        self.reindex();

        target_idx
    }

    /// Returns false if the update could not be completed (e.g. if the new key already
    /// exists!)
    pub fn update(&mut self, prev_key: &K, new_key: K, new_value: V) -> bool
    where
        K: Clone + Eq + std::hash::Hash,
    {
        let Some(&index) = self.index.get(prev_key) else {
            return false;
        };

        self.update_index(index, new_key, new_value)
    }

    /// Returns false if the update could not be completed (e.g. if the new key already
    /// exists!)
    pub fn update_index(&mut self, index: usize, new_key: K, new_value: V) -> bool
    where
        K: Clone + Eq + std::hash::Hash,
    {
        let prev_key = &self.entries[index].0;

        if &new_key != prev_key && self.index.contains_key(&new_key) {
            return false;
        }

        self.index.remove(prev_key);
        self.entries.remove(index);
        self.entries.insert(index, (new_key.clone(), new_value));
        self.index.insert(new_key, index);

        true
    }

    pub fn remove_key(&mut self, key: &K)
    where
        K: Clone + Eq + std::hash::Hash,
    {
        if let Some(i) = self.index.remove(key) {
            self.entries.remove(i);
        }

        self.reindex();
    }

    pub fn remove_index(&mut self, index: usize) {
        self.entries.remove(index);
        self.reindex();
    }

    fn reindex(&mut self) {
        self.index.clear();
        self.entries.iter().enumerate().for_each(|(i, (k, _))| {
            self.index.insert(k.clone(), i);
        });
    }
}

impl<I, V> Serialize for DataOnlySpec<I, V>
where
    I: Clone + Serialize + Eq + std::hash::Hash,
    V: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DataOnlySpecSerial {
            entries: self.entries.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de, K, V> Deserialize<'de> for DataOnlySpec<K, V>
where
    K: Deserialize<'de> + Eq + std::hash::Hash + Clone,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serial = DataOnlySpecSerial::deserialize(deserializer)?;

        let mut val = Self {
            entries: serial.entries,
            index: HashMap::new(),
        };

        val.reindex();

        Ok(val)
    }
}

#[derive(Serialize, Deserialize)]
struct DataOnlySpecSerial<K: Eq + std::hash::Hash, V> {
    entries: Vec<(K, V)>,
}
