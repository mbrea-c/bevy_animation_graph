use super::{AnimationGraph, EdgeSpec, EdgeValue};
use crate::{
    core::animation_clip::GraphClip,
    nodes::{
        blend_node::BlendNode, chain_node::ChainNode, clamp_f32::ClampF32, clip_node::ClipNode,
        flip_lr_node::FlipLRNode, loop_node::LoopNode, speed_node::SpeedNode, sub_f32::SubF32,
        AddF32, DivF32, GraphNode, MulF32,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationGraphSerial {
    nodes: Vec<AnimationNodeSerial>,
    #[serde(default)]
    input_parameters: HashMap<String, EdgeValue>,
    #[serde(default)]
    input_time_dependent_spec: HashMap<String, EdgeSpec>,
    #[serde(default)]
    output_parameter_spec: HashMap<String, EdgeSpec>,
    #[serde(default)]
    output_time_dependent_spec: HashMap<String, EdgeSpec>,
    /// (from_node, from_out_edge) -> (to_node, to_in_edge)
    /// Note that this is the opposite as [`AnimationGraph`] in order to make
    /// construction easier, and hand-editing of graph files more natural.
    edges: Vec<((String, String), (String, String))>,
    /// parameter_name -> (to_node, to_in_edge)
    input_edges: Vec<(String, (String, String))>,
    output_edges: Vec<((String, String), String)>,

    #[serde(default)]
    default_output: Option<String>,
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
    AddF32,
    SubF32,
    MulF32,
    DivF32,
    ClampF32,
    Graph(String),
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
                    AnimationNodeTypeSerial::AddF32 => AddF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::SubF32 => SubF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::MulF32 => MulF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::DivF32 => DivF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::ClampF32 => ClampF32::new().wrapped(&serial_node.name),
                    AnimationNodeTypeSerial::Graph(graph_name) => {
                        GraphNode::new(load_context.load(graph_name)).wrapped(&serial_node.name)
                    }
                };
                graph.add_node(node);
            }

            for ((source_node, source_edge), (target_node, target_edge)) in &serial.edges {
                graph.add_edge(source_node, source_edge, target_node, target_edge);
            }

            for (parameter_name, parameter_value) in &serial.input_parameters {
                graph.set_input_parameter(parameter_name, parameter_value.clone());
            }

            for (td_name, td_spec) in &serial.input_time_dependent_spec {
                graph.register_input_td(td_name, *td_spec);
            }

            for (p_name, p_spec) in &serial.output_parameter_spec {
                graph.register_output_parameter(p_name, *p_spec);
            }

            for (td_name, td_spec) in &serial.output_time_dependent_spec {
                graph.register_output_td(td_name, *td_spec);
            }

            for (parameter_name, (target_node, target_edge)) in &serial.input_edges {
                graph.add_input_edge(parameter_name, target_node, target_edge);
            }

            for ((source_node, source_edge), output_name) in &serial.output_edges {
                graph.add_output_edge(source_node, source_edge, output_name);
            }

            if let Some(def_output) = &serial.default_output {
                graph.set_default_output(def_output);
            }

            Ok(graph)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animgraph.ron"]
    }
}
