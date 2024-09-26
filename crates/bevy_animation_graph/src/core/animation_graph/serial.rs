use super::{pin, Extra};
use crate::prelude::{DataSpec, DataValue, OrderedMap};
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

pub type NodeIdSerial = String;
pub type PinIdSerial = String;
pub type TargetPinSerial = pin::TargetPin<NodeIdSerial, PinIdSerial>;
pub type SourcePinSerial = pin::SourcePin<NodeIdSerial, PinIdSerial>;

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationGraphSerial {
    #[serde(default)]
    pub nodes: Vec<AnimationNodeSerial>,
    #[serde(default)]
    pub edges_inverted: HashMap<TargetPinSerial, SourcePinSerial>,

    #[serde(default)]
    pub default_parameters: OrderedMap<PinIdSerial, DataValue>,
    #[serde(default)]
    pub input_times: OrderedMap<PinIdSerial, ()>,
    #[serde(default)]
    pub output_parameters: OrderedMap<PinIdSerial, DataSpec>,
    #[serde(default)]
    pub output_time: Option<()>,

    // for editor
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationNodeSerial {
    pub name: String,
    #[serde(flatten)]
    pub inner: ron::Value,
}
