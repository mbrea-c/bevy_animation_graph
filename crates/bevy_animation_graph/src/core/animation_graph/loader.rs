use super::{serial::AnimationGraphSerial, AnimationGraph};
use crate::{
    core::{animation_clip::GraphClip, errors::AssetLoaderError},
    prelude::{AnimationNode, ReflectNodeLike},
    utils::reflect_de::TypedReflectDeserializer,
};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    gltf::Gltf,
    prelude::*,
    reflect::TypeRegistryArc,
};
use serde::{de::DeserializeSeed, Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub struct AnimationGraphLoader {
    type_registry: TypeRegistryArc,
}

impl FromWorld for AnimationGraphLoader {
    fn from_world(world: &mut World) -> Self {
        let type_registry = world.resource::<AppTypeRegistry>();
        AnimationGraphLoader {
            type_registry: type_registry.0.clone(),
        }
    }
}

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
        let type_registry = self.type_registry.read();
        for node in serial.nodes {
            let type_path = &node.ty;
            let type_registration =
                type_registry.get_with_type_path(type_path).ok_or_else(|| {
                    ron::Error::Message(format!("No registration found for `{type_path}`"))
                })?;
            let node_like = type_registration.data::<ReflectNodeLike>().ok_or_else(|| {
                ron::Error::Message(format!("`{type_path}` is not an animation node"))
            })?;

            let inner =
                TypedReflectDeserializer::new(type_registration, &type_registry, load_context)
                    .deserialize(node.inner)?;
            let inner = node_like
                .get_boxed(inner)
                .expect("value with this type registration must be a NodeLike");

            graph.add_node(AnimationNode {
                name: node.name,
                inner,
                should_debug: false,
            });
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
