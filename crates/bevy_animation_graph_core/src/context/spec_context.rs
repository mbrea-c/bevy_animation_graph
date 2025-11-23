use bevy::{
    asset::Assets,
    platform::collections::{HashMap, HashSet},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation_graph::{AnimationGraph, PinId},
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

    pub fn set_from_node_spec(&mut self, node_spec: &NodeSpec) -> &mut Self {
        self.node_spec.set_from_node_spec(node_spec);
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

#[derive(Reflect, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeInputPin {
    Time(PinId),
    Data(PinId),
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeOutputPin {
    Time,
    Data(PinId),
}

#[derive(Reflect, Debug, Clone, Default)]
pub struct NodeSpec {
    input_times: HashSet<PinId>,
    input_data: HashMap<PinId, DataSpec>,
    output_time: bool,
    output_data: HashMap<PinId, DataSpec>,

    input_order: HashMap<NodeInputPin, i32>,
    output_order: HashMap<NodeOutputPin, i32>,

    next_input_order: i32,
    next_output_order: i32,
}

impl NodeSpec {
    pub fn add_input_time(&mut self, pin_id: PinId) {
        self.input_times.insert(pin_id.clone());
        self.input_order
            .insert(NodeInputPin::Time(pin_id), self.next_input_order);
        self.next_input_order += 1;
    }

    pub fn add_input_data(&mut self, pin_id: PinId, data_spec: DataSpec) {
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

    pub fn input_compare_key(&self, input: &NodeInputPin) -> i32 {
        self.input_order.get(input).copied().unwrap_or(i32::MAX)
    }

    pub fn output_compare_key(&self, output: &NodeOutputPin) -> i32 {
        self.output_order.get(output).copied().unwrap_or(i32::MAX)
    }

    pub fn get_input_data(&self, pin_id: &PinId) -> Option<DataSpec> {
        self.input_data.get(pin_id).copied()
    }

    pub fn has_input_time(&self, pin_id: &PinId) -> bool {
        self.input_times.contains(pin_id)
    }

    pub fn get_output_data(&self, pin_id: &PinId) -> Option<DataSpec> {
        self.output_data.get(pin_id).copied()
    }

    pub fn has_output_time(&self) -> bool {
        self.output_time
    }

    pub fn set_from_node_spec(&mut self, other: &NodeSpec) {
        *self = other.clone();
    }

    pub fn iter_output_data(&self) -> impl Iterator<Item = (&PinId, &DataSpec)> {
        self.output_data.iter()
    }
}

impl Serialize for NodeSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NodeSpecSerial {
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

impl<'de> Deserialize<'de> for NodeSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serial = NodeSpecSerial::deserialize(deserializer)?;

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
struct NodeSpecSerial {
    input_times: HashSet<PinId>,
    input_data: HashMap<PinId, DataSpec>,
    output_time: bool,
    output_data: HashMap<PinId, DataSpec>,

    input_order: HashMap<NodeInputPin, i32>,
    output_order: HashMap<NodeOutputPin, i32>,
}
