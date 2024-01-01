use crate::{
    core::{
        animation_graph::{SourcePin, TimeUpdate},
        duration_data::DurationData,
        frame::PoseFrame,
    },
    prelude::{AnimationGraph, GraphClip, ParamValue},
};

use super::pass_context::GraphContextRef;
use bevy::{
    asset::Assets, ecs::system::Query, reflect::prelude::*, transform::prelude::*, utils::HashMap,
};

#[derive(Reflect, Debug, Default)]
pub struct OutputCache {
    pub parameters: HashMap<SourcePin, ParamValue>,
    pub durations: HashMap<SourcePin, DurationData>,
    pub time_updates: HashMap<SourcePin, TimeUpdate>,
    pub poses: HashMap<SourcePin, PoseFrame>,
}

#[derive(Reflect, Debug, Default)]
struct TimeCacheSingle {
    /// Only set after a pose query, cannot be assumed to exist
    current: Option<f32>,
    /// Should always exist
    prev: f32,
}

impl TimeCacheSingle {
    pub fn push(&mut self) {
        if let Some(current) = self.current {
            self.prev = current;
        }
    }

    pub fn get_prev(&self) -> f32 {
        self.prev
    }

    pub fn set_curr(&mut self, value: f32) {
        self.current = Some(value);
    }
}

#[derive(Reflect, Debug, Default)]
pub struct TimeCaches {
    caches: HashMap<SourcePin, TimeCacheSingle>,
}

impl TimeCaches {
    pub fn push(&mut self) {
        for (_, cache) in self.caches.iter_mut() {
            cache.push();
        }
    }

    pub fn get_prev(&self, source_pin: &SourcePin) -> f32 {
        self.caches.get(source_pin).map_or(0., |c| c.get_prev())
    }

    pub fn set_curr(&mut self, source_pin: SourcePin, value: f32) {
        if let Some(cache) = self.caches.get_mut(&source_pin) {
            cache.set_curr(value);
        } else {
            let mut new_cache = TimeCacheSingle::default();
            new_cache.set_curr(value);
            self.caches.insert(source_pin, new_cache);
        }
    }
}

impl OutputCache {
    pub fn clear(&mut self) {
        self.parameters.clear();
        self.durations.clear();
        self.time_updates.clear();
        self.poses.clear();
    }
}

#[derive(Reflect, Debug, Default)]
pub struct OutputCaches {
    /// Caches are double buffered
    caches: [OutputCache; 2],
    current_cache: usize,
}

impl OutputCaches {
    pub fn flip(&mut self) {
        self.current_cache = self.other_cache();
    }

    pub fn other_cache(&self) -> usize {
        (self.current_cache + 1) % 2
    }

    pub fn get_cache(&self) -> &OutputCache {
        &self.caches[self.current_cache]
    }

    pub fn get_other_cache(&self) -> &OutputCache {
        &self.caches[self.other_cache()]
    }

    pub fn get_cache_mut(&mut self) -> &mut OutputCache {
        &mut self.caches[self.current_cache]
    }

    pub fn push(&mut self) {
        self.caches[self.other_cache()].clear();
        self.flip();
    }

    pub fn get_paramereter(&self, source_pin: &SourcePin) -> Option<&ParamValue> {
        self.get_cache().parameters.get(source_pin)
    }

    pub fn set_parameter(
        &mut self,
        source_pin: SourcePin,
        value: ParamValue,
    ) -> Option<ParamValue> {
        self.get_cache_mut().parameters.insert(source_pin, value)
    }

    pub fn get_duration(&self, source_pin: &SourcePin) -> Option<DurationData> {
        self.get_cache().durations.get(source_pin).cloned()
    }

    pub fn set_duration(
        &mut self,
        source_pin: SourcePin,
        value: DurationData,
    ) -> Option<DurationData> {
        self.get_cache_mut().durations.insert(source_pin, value)
    }

    pub fn get_time_update(&self, source_pin: &SourcePin) -> Option<&TimeUpdate> {
        self.get_cache().time_updates.get(source_pin)
    }

    pub fn set_time_update(
        &mut self,
        source_pin: SourcePin,
        value: TimeUpdate,
    ) -> Option<TimeUpdate> {
        self.get_cache_mut().time_updates.insert(source_pin, value)
    }

    pub fn get_pose(&self, source_pin: &SourcePin) -> Option<&PoseFrame> {
        self.get_cache().poses.get(source_pin)
    }

    pub fn set_pose(&mut self, source_pin: SourcePin, value: PoseFrame) -> Option<PoseFrame> {
        self.get_cache_mut().poses.insert(source_pin, value)
    }
}

#[derive(Debug, Default, Reflect)]
pub struct GraphContext {
    outputs: OutputCaches,
    times: TimeCaches,
    #[reflect(ignore)]
    subgraph_contexts: HashMap<String, GraphContext>,
}

/// Contains temprary data such as references to assets, gizmos, etc.
#[derive(Clone, Copy)]
pub struct SystemResources<'s> {
    pub graph_clip_assets: &'s Assets<GraphClip>,
    pub animation_graph_assets: &'s Assets<AnimationGraph>,
}

impl GraphContext {
    pub fn push_caches(&mut self) {
        self.outputs.push();
        self.times.push();

        for (_, sub_ctx) in self.subgraph_contexts.iter_mut() {
            sub_ctx.push_caches();
        }
    }

    pub fn get_parameter(&self, source_pin: &SourcePin) -> Option<&ParamValue> {
        self.outputs.get_paramereter(source_pin)
    }

    pub fn set_parameter(
        &mut self,
        source_pin: SourcePin,
        value: ParamValue,
    ) -> Option<ParamValue> {
        self.outputs.set_parameter(source_pin, value)
    }

    pub fn get_duration(&self, source_pin: &SourcePin) -> Option<DurationData> {
        self.outputs.get_duration(source_pin)
    }

    pub fn set_duration(
        &mut self,
        source_pin: SourcePin,
        value: DurationData,
    ) -> Option<DurationData> {
        self.outputs.set_duration(source_pin, value)
    }

    pub fn get_time_update(&self, source_pin: &SourcePin) -> Option<&TimeUpdate> {
        self.outputs.get_time_update(source_pin)
    }

    pub fn set_time_update(
        &mut self,
        source_pin: SourcePin,
        value: TimeUpdate,
    ) -> Option<TimeUpdate> {
        self.outputs.set_time_update(source_pin, value)
    }

    pub fn get_prev_time(&self, source_pin: &SourcePin) -> f32 {
        self.times.get_prev(source_pin)
    }

    pub fn set_time(&mut self, source_pin: SourcePin, value: f32) {
        self.times.set_curr(source_pin, value);
    }

    pub fn get_pose(&self, source_pin: &SourcePin) -> Option<&PoseFrame> {
        self.outputs.get_pose(source_pin)
    }

    pub fn set_pose(&mut self, source_pin: SourcePin, value: PoseFrame) -> Option<PoseFrame> {
        self.outputs.set_pose(source_pin, value)
    }

    pub(super) fn context_for_subgraph_or_insert_default(&mut self, node: &str) -> GraphContextRef {
        if !self.subgraph_contexts.contains_key(node) {
            self.subgraph_contexts
                .insert(node.to_string(), GraphContext::default());
        }

        self.subgraph_contexts.get_mut(node).unwrap().into()
    }
}
