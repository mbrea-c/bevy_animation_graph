use bevy::{asset::LoadContext, reflect::TypeRegistry};
use serde::{Deserialize, Serialize, de::DeserializeSeed};

use crate::core::{
    animation_graph::NodeId,
    animation_node::{
        AnimationNode,
        dyn_node_like::{
            DynNodeLike,
            serial::{DynNodeLikeDeserializer, DynNodeLikeSerializer},
        },
    },
    errors::AssetLoaderError,
};

#[derive(Deserialize)]
pub struct AnimationNodeDeserializer {
    pub id: NodeId,
    pub name: String,
    pub inner: Box<ron::value::RawValue>,
    pub should_debug: bool,
}

impl AnimationNodeDeserializer {
    pub fn finish_deserialize<'a, 'b>(
        &self,
        type_registry: &'a TypeRegistry,
        load_context: &'a mut LoadContext<'b>,
    ) -> Result<AnimationNode, AssetLoaderError> {
        let mut ron_deserializer = ron::de::Deserializer::from_str(self.inner.get_ron())?;
        let dyn_node_like_deserializer = DynNodeLikeDeserializer {
            type_registry,
            load_context,
        };

        let inner: DynNodeLike = dyn_node_like_deserializer
            .deserialize(&mut ron_deserializer)
            .map_err(|err| ron_deserializer.span_error(err))?;

        Ok(AnimationNode {
            id: self.id,
            name: self.name.clone(),
            inner: inner,
            should_debug: self.should_debug,
        })
    }
}

#[derive(Serialize)]
pub struct AnimationNodeSerializer<'a> {
    pub id: NodeId,
    pub name: String,
    pub inner: DynNodeLikeSerializer<'a>,
    pub should_debug: bool,
}

impl<'a> AnimationNodeSerializer<'a> {
    pub fn new(animation_node: &AnimationNode, type_registry: &'a TypeRegistry) -> Self {
        Self {
            id: animation_node.id,
            name: animation_node.name.clone(),
            inner: DynNodeLikeSerializer {
                type_registry,
                value: animation_node.inner.clone(),
            },
            should_debug: animation_node.should_debug,
        }
    }
}
