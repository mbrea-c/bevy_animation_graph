use super::AnimationGraph;
use crate::{
    core::{animation_clip::GraphClip, frame::PoseSpec, parameters::ParamValueSerial},
    nodes::{
        blend_node::BlendNode, chain_node::ChainNode, clip_node::ClipNode,
        flip_lr_node::FlipLRNode, loop_node::LoopNode, speed_node::SpeedNode, AbsF32, AddF32,
        ClampF32, DivF32, GraphNode, MulF32,
    },
    prelude::{
        ExtendSkeleton, IntoBoneSpaceNode, IntoCharacterSpaceNode, ParamSpec, RotationArcNode,
        RotationNode, SubF32,
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
    input_parameters: HashMap<String, ParamValueSerial>,
    #[serde(default)]
    input_pose_spec: HashMap<String, PoseSpec>,
    #[serde(default)]
    output_parameter_spec: HashMap<String, ParamSpec>,
    #[serde(default)]
    output_pose_spec: Option<PoseSpec>,
    /// (from_node, from_out_edge) -> (to_node, to_in_edge)
    /// Note that this is the opposite as [`AnimationGraph`] in order to make
    /// construction easier, and hand-editing of graph files more natural.
    #[serde(default)]
    parameter_edges: Vec<((String, String), (String, String))>,
    #[serde(default)]
    pose_edges: Vec<(String, (String, String))>,

    /// parameter_name -> (to_node, to_in_edge)
    #[serde(default)]
    input_parameter_edges: Vec<(String, (String, String))>,
    #[serde(default)]
    output_parameter_edges: Vec<((String, String), String)>,
    #[serde(default)]
    input_pose_edges: Vec<(String, (String, String))>,
    #[serde(default)]
    output_pose_edge: Option<String>,
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
    Rotation,
    AddF32,
    SubF32,
    MulF32,
    DivF32,
    ClampF32,
    AbsF32,
    RotationArc,
    IntoBoneSpace,
    IntoCharacterSpace,
    ExtendSkeleton,
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
                    AnimationNodeTypeSerial::FlipLR => FlipLRNode::new().wrapped(&serial_node.name),
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
                };
                graph.add_node(node);
            }
            // ------------------------------------------------------------------------------------

            // --- Set up inputs and outputs
            // ------------------------------------------------------------------------------------
            for (param_name, param_value) in &serial.input_parameters {
                graph.set_default_parameter(param_name, param_value.clone().into());
            }
            for (pose_name, pose_spec) in &serial.input_pose_spec {
                graph.add_input_pose(pose_name, *pose_spec);
            }
            for (param_name, param_spec) in &serial.output_parameter_spec {
                graph.add_output_parameter(param_name, *param_spec);
            }

            if let Some(output_pose_spec) = serial.output_pose_spec {
                graph.add_output_pose(output_pose_spec);
            }
            // ------------------------------------------------------------------------------------

            // --- Set up edges
            // ------------------------------------------------------------------------------------
            for ((source_node, source_edge), (target_node, target_edge)) in &serial.parameter_edges
            {
                graph.add_node_parameter_edge(source_node, source_edge, target_node, target_edge);
            }

            for (source_node, (target_node, target_pin)) in &serial.pose_edges {
                graph.add_node_pose_edge(source_node, target_node, target_pin);
            }

            for (param_name, (target_node, target_edge)) in &serial.input_parameter_edges {
                graph.add_input_parameter_edge(param_name, target_node, target_edge);
            }

            for ((source_node, source_edge), output_name) in &serial.output_parameter_edges {
                graph.add_output_parameter_edge(source_node, source_edge, output_name);
            }

            for (param_name, (target_node, target_edge)) in &serial.input_pose_edges {
                graph.add_input_pose_edge(param_name, target_node, target_edge);
            }

            if let Some(node_name) = &serial.output_pose_edge {
                graph.add_output_pose_edge(node_name);
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
