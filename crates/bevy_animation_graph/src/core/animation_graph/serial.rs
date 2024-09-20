use super::{pin, AnimationGraph, Extra};
use crate::{
    core::{animation_clip::Interpolation, edge_data::AnimationEvent},
    flipping::config::FlipConfig,
    nodes::{BlendMode, ChainDecay, CompareOp, RotationMode, RotationSpace},
    prelude::{AnimationNode, AnimationNodeType, DataSpec, DataValue},
    utils::ordered_map::OrderedMap,
};
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
//     pub nodes: HashMap<String, AnimationNode>,
//     /// Inverted, indexed by output node name.
//     pub edges: HashMap<TargetPin, SourcePin>,
//     pub default_output: Option<String>,
//
//     pub default_parameters: HashMap<PinId, ParamValue>,
//     pub input_poses: HashMap<PinId, PoseSpec>,
//     pub output_parameters: HashMap<PinId, ParamSpec>,
//     pub output_pose: Option<PoseSpec>,
//
//     pub extra: Extra,

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
    pub node: AnimationNodeTypeSerial,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AnimationNodeTypeSerial {
    Clip(String, Option<f32>, #[serde(default)] Option<Interpolation>),
    Chain {
        #[serde(default)]
        interpolation_period: f32,
    },
    Blend {
        #[serde(default)]
        mode: BlendMode,
    },
    FlipLR {
        #[serde(default)]
        config: FlipConfig,
    },
    Loop {
        #[serde(default)]
        interpolation_period: f32,
    },
    Padding {
        interpolation_period: f32,
    },
    Speed,
    Rotation(
        RotationMode,
        RotationSpace,
        ChainDecay,
        usize,
        #[serde(default)] f32,
    ),
    AddF32,
    SubF32,
    MulF32,
    DivF32,
    ClampF32,
    CompareF32(CompareOp),
    AbsF32,
    ConstBool(bool),
    RotationArc,
    FireEvent(AnimationEvent),
    // IntoBoneSpace,
    // IntoCharacterSpace,
    // IntoGlobalSpace,
    // ExtendSkeleton,
    TwoBoneIK,
    Dummy,
    Fsm(String),
    Graph(String),
}

impl From<&AnimationGraph> for AnimationGraphSerial {
    fn from(value: &AnimationGraph) -> Self {
        Self {
            nodes: value.nodes.values().map(|v| v.into()).collect(),
            edges_inverted: value
                .edges
                .clone()
                .into_iter()
                .map(|(k, v)| (k.map_into(), v.map_into()))
                .collect(),
            default_parameters: value
                .default_parameters
                .iter()
                .map(|(k, v)| (k.into(), v.clone()))
                .collect(),
            input_times: value
                .input_times
                .iter()
                .map(|(k, v)| (k.into(), *v))
                .collect(),
            output_parameters: value
                .output_parameters
                .iter()
                .map(|(k, v)| (k.into(), *v))
                .collect(),
            output_time: value.output_time,
            extra: value.extra.clone(),
        }
    }
}

impl From<&AnimationNode> for AnimationNodeSerial {
    fn from(value: &AnimationNode) -> Self {
        Self {
            name: value.name.clone(),
            node: (&value.node).into(),
        }
    }
}

impl From<&AnimationNodeType> for AnimationNodeTypeSerial {
    fn from(value: &AnimationNodeType) -> Self {
        match value {
            AnimationNodeType::Clip(n) => AnimationNodeTypeSerial::Clip(
                n.clip.path().unwrap().to_string(),
                n.override_duration,
                n.override_interpolation,
            ),
            AnimationNodeType::Dummy(_) => AnimationNodeTypeSerial::Dummy,
            AnimationNodeType::Chain(n) => AnimationNodeTypeSerial::Chain {
                interpolation_period: n.interpolation_period,
            },
            AnimationNodeType::Blend(n) => AnimationNodeTypeSerial::Blend { mode: n.mode },
            AnimationNodeType::FlipLR(n) => AnimationNodeTypeSerial::FlipLR {
                config: n.config.clone(),
            },
            AnimationNodeType::Loop(n) => AnimationNodeTypeSerial::Loop {
                interpolation_period: n.interpolation_period,
            },
            AnimationNodeType::Speed(_) => AnimationNodeTypeSerial::Speed,
            AnimationNodeType::Rotation(n) => AnimationNodeTypeSerial::Rotation(
                n.application_mode,
                n.rotation_space,
                n.chain_decay,
                n.chain_length,
                n.base_weight,
            ),
            // AnimationNodeType::IntoBoneSpace(_) => AnimationNodeTypeSerial::IntoBoneSpace,
            // AnimationNodeType::IntoCharacterSpace(_) => AnimationNodeTypeSerial::IntoCharacterSpace,
            // AnimationNodeType::IntoGlobalSpace(_) => AnimationNodeTypeSerial::IntoGlobalSpace,
            // AnimationNodeType::ExtendSkeleton(_) => AnimationNodeTypeSerial::ExtendSkeleton,
            AnimationNodeType::TwoBoneIK(_) => AnimationNodeTypeSerial::TwoBoneIK,
            AnimationNodeType::AddF32(_) => AnimationNodeTypeSerial::AddF32,
            AnimationNodeType::MulF32(_) => AnimationNodeTypeSerial::MulF32,
            AnimationNodeType::DivF32(_) => AnimationNodeTypeSerial::DivF32,
            AnimationNodeType::SubF32(_) => AnimationNodeTypeSerial::SubF32,
            AnimationNodeType::ClampF32(_) => AnimationNodeTypeSerial::ClampF32,
            AnimationNodeType::CompareF32(n) => AnimationNodeTypeSerial::CompareF32(n.op),
            AnimationNodeType::AbsF32(_) => AnimationNodeTypeSerial::AbsF32,
            AnimationNodeType::ConstBool(n) => AnimationNodeTypeSerial::ConstBool(n.constant),
            AnimationNodeType::RotationArc(_) => AnimationNodeTypeSerial::RotationArc,
            AnimationNodeType::Fsm(n) => {
                AnimationNodeTypeSerial::Fsm(n.fsm.path().unwrap().to_string())
            }
            AnimationNodeType::FireEvent(n) => AnimationNodeTypeSerial::FireEvent(n.event.clone()),
            AnimationNodeType::Graph(n) => {
                AnimationNodeTypeSerial::Graph(n.graph.path().unwrap().to_string())
            }
            AnimationNodeType::Custom(_) => panic!("Cannot serialize custom node!"),
            AnimationNodeType::Padding(n) => AnimationNodeTypeSerial::Padding {
                interpolation_period: n.interpolation_period,
            },
        }
    }
}
