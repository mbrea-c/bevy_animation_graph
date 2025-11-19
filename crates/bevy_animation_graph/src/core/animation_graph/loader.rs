use crate::core::{
    animation_graph::{AnimationGraph, serial::AnimationGraphDeserializer},
    errors::AssetLoaderError,
};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    ecs::{
        reflect::AppTypeRegistry,
        world::{FromWorld, World},
    },
    reflect::TypeRegistryArc,
};

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

        let serial = ron::de::from_bytes::<AnimationGraphDeserializer>(&bytes)?;

        let mut graph = AnimationGraph::new();

        // Set up editor metadata
        // Needs to be done before adding nodes in case data is missing, so that it
        // gets properly initialized.
        graph.extra = serial.extra;

        // Add nodes
        for node_ron in serial.nodes {
            let node = node_ron.finish_deserialize(&self.type_registry.read(), load_context)?;
            graph.add_node(node);
        }

        // Set up inputs and outputs
        for (input_name, data_spec) in serial.input_data {
            graph.set_input_data(input_name, data_spec);
        }
        for (pose_name, _) in serial.input_times {
            graph.add_input_time(pose_name);
        }
        for (param_name, param_spec) in serial.output_parameters {
            graph.add_output_data(param_name, param_spec);
        }
        if serial.output_time.is_some() {
            graph.add_output_time();
        }

        // Set default data values
        for (param_name, param_value) in serial.default_data {
            graph.set_default_data(param_name, param_value.clone());
        }

        // Set up edges
        for (target_pin, source_pin) in serial.edges_inverted.clone().into_iter() {
            graph.add_edge(source_pin, target_pin);
        }

        // Static validation
        graph.validate()?;

        Ok(graph)
    }

    fn extensions(&self) -> &[&str] {
        &["animgraph.ron"]
    }
}
