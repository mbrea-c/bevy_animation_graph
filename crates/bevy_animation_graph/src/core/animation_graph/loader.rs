use super::{
    serial::{AnimationGraphSerial, AnimationNodeTypeSerial},
    AnimationGraph,
};
use crate::{
    core::{animation_clip::GraphClip, errors::AssetLoaderError},
    nodes::{
        AbsF32, AddF32, BlendNode, ChainNode, ClampF32, ClipNode, CompareF32, ConstBool, DivF32,
        FSMNode, FireEventNode, FlipLRNode, GraphNode, LoopNode, MulF32, PaddingNode,
        RotationArcNode, RotationNode, SpeedNode, SubF32, TwoBoneIKNode,
    },
    prelude::DummyNode,
};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    gltf::Gltf,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum GraphClipSource {
    GltfNamed {
        path: String,
        animation_name: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphClipSerial {
    source: GraphClipSource,
    skeleton: String,
}

#[derive(Default)]
pub struct GraphClipLoader;

impl AssetLoader for GraphClipLoader {
    type Asset = GraphClip;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let serial: GraphClipSerial = ron::de::from_bytes(&bytes)?;

        let bevy_clip = match serial.source {
            GraphClipSource::GltfNamed {
                path,
                animation_name,
            } => {
                let gltf_loaded_asset = load_context.loader().direct().untyped().load(path).await?;
                let gltf: &Gltf = gltf_loaded_asset.get().unwrap();

                let Some(clip_handle) = gltf
                    .named_animations
                    .get(&animation_name.clone().into_boxed_str())
                else {
                    return Err(AssetLoaderError::GltfMissingLabel(animation_name));
                };

                let Some(clip_path) = clip_handle.path() else {
                    return Err(AssetLoaderError::GltfMissingLabel(animation_name));
                };

                let clip_bevy: bevy::animation::AnimationClip = gltf_loaded_asset
                    .get_labeled(clip_path.label_cow().unwrap())
                    .unwrap()
                    .get::<bevy::animation::AnimationClip>()
                    .unwrap()
                    .clone();

                clip_bevy
            }
        };

        let skeleton = load_context.loader().load(serial.skeleton);

        let clip_mine = GraphClip::from_bevy_clip(bevy_clip, skeleton);

        Ok(clip_mine)
    }

    fn extensions(&self) -> &[&str] {
        &["anim.ron"]
    }
}

#[derive(Default)]
pub struct AnimationGraphLoader;

impl AssetLoader for AnimationGraphLoader {
    type Asset = AnimationGraph;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let serial: AnimationGraphSerial = ron::de::from_bytes(&bytes)?;

        let mut graph = AnimationGraph::new();

        // --- Set up extra data
        // --- Needs to be done before adding nodes in case data is missing, so that it
        // --- gets properly initialized.
        // ------------------------------------------------------------------------------------
        graph.extra = serial.extra;
        // ------------------------------------------------------------------------------------

        // --- Add nodes
        // ------------------------------------------------------------------------------------
        for serial_node in &serial.nodes {
            let node = match &serial_node.node {
                AnimationNodeTypeSerial::Clip(
                    clip_name,
                    override_duration,
                    override_interpolation,
                ) => ClipNode::new(
                    load_context.load(clip_name),
                    *override_duration,
                    *override_interpolation,
                )
                .wrapped(&serial_node.name),
                AnimationNodeTypeSerial::Blend { mode } => {
                    BlendNode::new(*mode).wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::Chain {
                    interpolation_period,
                } => ChainNode::new(*interpolation_period).wrapped(&serial_node.name),
                AnimationNodeTypeSerial::FlipLR { config } => {
                    FlipLRNode::new(config.clone()).wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::Loop {
                    interpolation_period,
                } => LoopNode::new(*interpolation_period).wrapped(&serial_node.name),
                AnimationNodeTypeSerial::Speed => SpeedNode::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::Rotation(mode, space, decay, length, base_weight) => {
                    RotationNode::new(*mode, *space, *decay, *length, *base_weight)
                        .wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::FireEvent(ev) => {
                    FireEventNode::new(ev.clone()).wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::AddF32 => AddF32::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::SubF32 => SubF32::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::MulF32 => MulF32::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::DivF32 => DivF32::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::ClampF32 => ClampF32::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::CompareF32(op) => {
                    CompareF32::new(*op).wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::AbsF32 => AbsF32::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::ConstBool(b) => {
                    ConstBool::new(*b).wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::RotationArc => {
                    RotationArcNode::new().wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::Fsm(fsm_name) => {
                    FSMNode::new(load_context.load(fsm_name)).wrapped(&serial_node.name)
                }
                AnimationNodeTypeSerial::Graph(graph_name) => {
                    GraphNode::new(load_context.load(graph_name)).wrapped(&serial_node.name)
                }
                // AnimationNodeTypeSerial::IntoBoneSpace => {
                //     IntoBoneSpaceNode::new().wrapped(&serial_node.name)
                // }
                // AnimationNodeTypeSerial::IntoCharacterSpace => {
                //     IntoCharacterSpaceNode::new().wrapped(&serial_node.name)
                // }
                // AnimationNodeTypeSerial::ExtendSkeleton => {
                //     ExtendSkeleton::new().wrapped(&serial_node.name)
                // }
                AnimationNodeTypeSerial::TwoBoneIK => {
                    TwoBoneIKNode::new().wrapped(&serial_node.name)
                }
                // AnimationNodeTypeSerial::IntoGlobalSpace => {
                //     IntoGlobalSpaceNode::new().wrapped(&serial_node.name)
                // }
                AnimationNodeTypeSerial::Dummy => DummyNode::new().wrapped(&serial_node.name),
                AnimationNodeTypeSerial::Padding {
                    interpolation_period,
                } => PaddingNode::new(*interpolation_period).wrapped(&serial_node.name),
            };
            graph.add_node(node);
        }
        // ------------------------------------------------------------------------------------

        // --- Set up inputs and outputs
        // ------------------------------------------------------------------------------------
        for (param_name, param_value) in &serial.default_parameters {
            graph.set_default_parameter(param_name, param_value.clone());
        }
        for (pose_name, _) in &serial.input_times {
            graph.add_input_time(pose_name);
        }
        for (param_name, param_spec) in &serial.output_parameters {
            graph.add_output_parameter(param_name, *param_spec);
        }
        if serial.output_time.is_some() {
            graph.add_output_time();
        }
        // ------------------------------------------------------------------------------------

        // --- Set up edges
        // ------------------------------------------------------------------------------------
        for (target_pin, source_pin) in serial.edges_inverted.clone().into_iter() {
            graph.add_edge(source_pin.map_into(), target_pin.map_into());
        }
        // ------------------------------------------------------------------------------------

        graph.validate()?;

        Ok(graph)
    }

    fn extensions(&self) -> &[&str] {
        &["animgraph.ron"]
    }
}
