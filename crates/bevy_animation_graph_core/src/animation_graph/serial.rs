//! Deserialization happens in two steps:
//! 1. First, we deserialize into an [`AnimationGraphDeserializer`]. This does not yet try to
//!    deserialize the inner node types, it keeps them as [`ron::value::RawValue`]s. This is
//!    because these need to be deserialized using [`serde::DeserializeSeed`]: It is not possible
//!    to derive that trait, so the two-step process keeps us from having to manually impl
//!    [`serde::DeserializeSeed`] on the animation graph deserialization pod.
//! 2. Second, the loader iterates over animation nodes and completes deserialization using
//!    [`serde::DeserializeSeed`], providing references to type registry (to identify node types
//!    and deserialize via reflection) and to the load context (to load handles recursively).
//!
//! This process may seem complicated, but it is better than the alternative (manual
//! `DeserializeSeed` impl on animation graph).
use bevy::{platform::collections::HashMap, reflect::TypeRegistry};
use serde::{Deserialize, Serialize};

use crate::{
    animation_graph::{AnimationGraph, EditorMetadata, PinId, SourcePin, TargetPin},
    animation_node::serial::{AnimationNodeDeserializer, AnimationNodeSerializer},
    context::spec_context::NodeSpec,
    edge_data::DataValue,
};

#[derive(Deserialize)]
pub struct AnimationGraphDeserializer {
    pub nodes: Vec<AnimationNodeDeserializer>,
    pub edges_inverted: HashMap<TargetPin, SourcePin>,

    pub node_spec: NodeSpec,

    pub default_data: HashMap<PinId, DataValue>,

    pub extra: EditorMetadata,
}

#[derive(Serialize)]
pub struct AnimationGraphSerializer<'a> {
    pub nodes: Vec<AnimationNodeSerializer<'a>>,
    pub edges_inverted: HashMap<TargetPin, SourcePin>,

    pub node_spec: NodeSpec,

    pub default_data: HashMap<PinId, DataValue>,

    pub extra: EditorMetadata,
}

impl AnimationGraphSerializer<'_> {
    pub fn new<'a>(
        graph: &AnimationGraph,
        type_registry: &'a TypeRegistry,
    ) -> AnimationGraphSerializer<'a> {
        let mut serial = AnimationGraphSerializer {
            nodes: Vec::new(),
            edges_inverted: graph.edges_inverted.clone(),
            node_spec: graph.node_spec.clone(),
            default_data: graph.default_data.clone(),
            extra: graph.extra.clone(),
        };

        for node in graph.nodes.values() {
            serial
                .nodes
                .push(AnimationNodeSerializer::new(node, type_registry));
        }

        serial
    }
}
