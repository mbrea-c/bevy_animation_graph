use super::{
    serial::{AnimationGraphSerial, AnimationNodeTypeSerial},
    AnimationGraph,
};
use crate::{
    core::{animation_clip::GraphClip, errors::AssetLoaderError},
    nodes::{
        blend_node::BlendNode, chain_node::ChainNode, clip_node::ClipNode,
        flip_lr_node::FlipLRNode, loop_node::LoopNode, speed_node::SpeedNode, AbsF32, AddF32,
        ClampF32, DivF32, GraphNode, MulF32,
    },
    prelude::{
        DummyNode, ExtendSkeleton, IntoBoneSpaceNode, IntoCharacterSpaceNode, IntoGlobalSpaceNode,
        RotationArcNode, RotationNode, SubF32, TwoBoneIKNode,
    },
};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    gltf::Gltf,
    utils::BoxedFuture,
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
}

#[derive(Default)]
pub struct GraphClipLoader;

impl AssetLoader for GraphClipLoader {
    type Asset = GraphClip;
    type Settings = ();
    type Error = AssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await?;
            let serial: GraphClipSerial = ron::de::from_bytes(&bytes)?;

            let bevy_clip = match serial.source {
                GraphClipSource::GltfNamed {
                    path,
                    animation_name,
                } => {
                    let gltf_loaded_asset = load_context.load_direct(path).await?;
                    let gltf: &Gltf = gltf_loaded_asset.get().unwrap();

                    let Some(clip_handle) = gltf.named_animations.get(&animation_name) else {
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

            let clip_mine = GraphClip::from(bevy_clip);

            Ok(clip_mine)
        })
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

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
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
                    AnimationNodeTypeSerial::Clip(clip_name, override_duration) => {
                        ClipNode::new(load_context.load(clip_name), *override_duration)
                            .wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::Blend => BlendNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Chain => ChainNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::FlipLR { config } => {
                        FlipLRNode::new(config.clone()).wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::Loop => LoopNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Speed => SpeedNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Rotation => {
                        RotationNode::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::AddF32 => AddF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::SubF32 => SubF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::MulF32 => MulF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::DivF32 => DivF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::ClampF32 => ClampF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::AbsF32 => AbsF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::RotationArc => {
                        RotationArcNode::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::Graph(graph_name) => {
                        GraphNode::new(load_context.load(graph_name)).wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::IntoBoneSpace => {
                        IntoBoneSpaceNode::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::IntoCharacterSpace => {
                        IntoCharacterSpaceNode::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::ExtendSkeleton => {
                        ExtendSkeleton::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::TwoBoneIK => {
                        TwoBoneIKNode::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::IntoGlobalSpace => {
                        IntoGlobalSpaceNode::new().wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::Dummy => DummyNode::new().wrapped(&serial_node.name),
                };
                graph.add_node(node);
            }
            // ------------------------------------------------------------------------------------

            // --- Set up inputs and outputs
            // ------------------------------------------------------------------------------------
            for (param_name, param_value) in &serial.default_parameters {
                graph.set_default_parameter(param_name, param_value.clone());
            }
            for (pose_name, pose_spec) in &serial.input_poses {
                graph.add_input_pose(pose_name, *pose_spec);
            }
            for (param_name, param_spec) in &serial.output_parameters {
                graph.add_output_parameter(param_name, *param_spec);
            }
            if let Some(output_pose_spec) = serial.output_pose {
                graph.add_output_pose(output_pose_spec);
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
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animgraph.ron"]
    }
}
