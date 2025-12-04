use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    ecs::{
        reflect::AppTypeRegistry,
        world::{FromWorld, World},
    },
    reflect::TypeRegistryArc,
};

use crate::{
    animation_graph::{AnimationGraph, serial::AnimationGraphDeserializer},
    errors::AssetLoaderError,
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
        graph.editor_metadata = serial.editor_metadata;

        // Add nodes
        for node_ron in serial.nodes {
            let node = node_ron.finish_deserialize(&self.type_registry.read(), load_context)?;
            graph.add_node(node);
        }

        graph.io_spec = serial.io_spec;

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
