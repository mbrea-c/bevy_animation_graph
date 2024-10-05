use super::{serial::AnimationGraphLoadDeserializer, AnimationGraph};
use crate::core::{animation_clip::GraphClip, errors::AssetLoaderError};
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

        // TODO: what's stopping us from removing the AnimationGraphSerial
        // intermediary, and just deserializing to an AnimationGraph?

        let mut ron_deserializer = ron::de::Deserializer::from_bytes(&bytes)?;
        let graph_deserializer = AnimationGraphLoadDeserializer {
            type_registry: &self.type_registry.read(),
            load_context,
        };
        let serial = graph_deserializer
            .deserialize(&mut ron_deserializer)
            .map_err(|err| ron_deserializer.span_error(err))?;

        let mut graph = AnimationGraph::new();

        // --- Set up extra data
        // --- Needs to be done before adding nodes in case data is missing, so that it
        // --- gets properly initialized.
        // ------------------------------------------------------------------------------------
        graph.extra = serial.extra;
        // ------------------------------------------------------------------------------------

        // --- Add nodes
        // ------------------------------------------------------------------------------------
        for node in serial.nodes {
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
