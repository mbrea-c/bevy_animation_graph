use super::{AnimationGraph, serial::AnimationGraphLoadDeserializer};
use crate::core::errors::AssetLoaderError;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypeRegistryArc,
};
use serde::de::DeserializeSeed;

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

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
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
            graph.add_edge(source_pin, target_pin);
        }
        // ------------------------------------------------------------------------------------

        graph.validate()?;

        Ok(graph)
    }

    fn extensions(&self) -> &[&str] {
        &["animgraph.ron"]
    }
}
