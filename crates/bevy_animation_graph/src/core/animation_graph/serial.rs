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
use bevy::{platform::collections::HashMap, prelude::*, reflect::TypeRegistry};
use serde::{Deserialize, Serialize};

use super::{AnimationGraph, EditorMetadata};
use crate::core::{
    animation_graph::{GraphInputPin, PinId, SourcePin, TargetPin},
    animation_node::serial::{AnimationNodeDeserializer, AnimationNodeSerializer},
    edge_data::{DataSpec, DataValue},
};

#[derive(Deserialize)]
pub struct AnimationGraphDeserializer {
    pub nodes: Vec<AnimationNodeDeserializer>,
    pub edges_inverted: HashMap<TargetPin, SourcePin>,

    pub input_data: HashMap<GraphInputPin, DataSpec>,
    pub input_times: HashMap<GraphInputPin, ()>,
    pub output_parameters: HashMap<PinId, DataSpec>,
    pub output_time: Option<()>,

    pub default_data: HashMap<GraphInputPin, DataValue>,

    pub extra: EditorMetadata,
}

#[derive(Serialize)]
pub struct AnimationGraphSerializer<'a> {
    pub nodes: Vec<AnimationNodeSerializer<'a>>,
    pub edges_inverted: HashMap<TargetPin, SourcePin>,

    pub input_data: HashMap<GraphInputPin, DataSpec>,
    pub input_times: HashMap<GraphInputPin, ()>,
    pub output_data: HashMap<PinId, DataSpec>,
    pub output_time: Option<()>,

    pub default_data: HashMap<GraphInputPin, DataValue>,

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
            input_data: graph.input_data.clone(),
            input_times: graph.input_times.clone(),
            output_data: graph.output_parameters.clone(),
            output_time: graph.output_time,
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
