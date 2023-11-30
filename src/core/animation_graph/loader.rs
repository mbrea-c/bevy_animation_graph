use super::{AnimationGraph, EdgeValue};
use crate::{
    core::animation_clip::GraphClip,
    nodes::{
        blend_node::BlendNode, chain_node::ChainNode, clip_node::ClipNode,
        flip_lr_node::FlipLRNode, loop_node::LoopNode, speed_node::SpeedNode,
    },
    utils::asset_loader_error::AssetLoaderError,
};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    gltf::Gltf,
    utils::{BoxedFuture, HashMap},
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
                        return Err(AssetLoaderError::GltfMissingLabel(animation_name.into()));
                    };

                    let Some(clip_path) = clip_handle.path() else {
                        return Err(AssetLoaderError::GltfMissingLabel(animation_name.into()));
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

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationGraphSerial {
    nodes: Vec<AnimationNodeSerial>,
    /// (from_node, from_out_edge) -> (to_node, to_in_edge)
    /// Note that this is the opposite as [`AnimationGraph`] in order to make
    /// construction easier, and hand-editing of graph files more natural.
    edges: Vec<((String, String), (String, String))>,
    parameters: HashMap<String, EdgeValue>,
    /// parameter_name -> (to_node, to_in_edge)
    parameter_edges: HashMap<String, (String, String)>,
    output: (String, String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationNodeSerial {
    name: String,
    node: AnimationNodeTypeSerial,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AnimationNodeTypeSerial {
    Clip(String, Option<f32>),
    Blend,
    Chain,
    FlipLR,
    Loop,
    Speed,
    // Graph(#[reflect(ignore)] AnimationGraph),
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

            for serial_node in &serial.nodes {
                let node = match &serial_node.node {
                    AnimationNodeTypeSerial::Clip(clip_name, override_duration) => {
                        ClipNode::new(load_context.load(clip_name), *override_duration)
                            .wrapped(&serial_node.name)
                    }
                    AnimationNodeTypeSerial::Blend => BlendNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Chain => ChainNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::FlipLR => FlipLRNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Loop => LoopNode::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Speed => SpeedNode::new().wrapped(&serial_node.name),
                };
                graph.add_node(node);
            }

            for ((source_node, source_edge), (target_node, target_edge)) in &serial.edges {
                graph.add_edge(source_node, source_edge, target_node, target_edge);
            }

            for (parameter_name, parameter_value) in &serial.parameters {
                graph.set_parameter(parameter_name, parameter_value.clone());
            }

            for (parameter_name, (target_node, target_edge)) in &serial.parameter_edges {
                graph.add_parameter_edge(parameter_name, target_node, target_edge);
            }

            let (output_node, output_edge) = &serial.output;
            graph.set_out_edge(output_node, output_edge);

            Ok(graph)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animgraph.ron"]
    }
}
