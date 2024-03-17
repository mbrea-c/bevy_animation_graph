use super::pass_context::GraphContextRef;
use crate::{
    core::{
        animation_graph::{SourcePin, TimeUpdate},
        duration_data::DurationData,
        pose::Pose,
    },
    prelude::ParamValue,
};
use bevy::{reflect::prelude::*, utils::HashMap};

#[derive(Reflect, Debug, Default)]
pub struct OutputCache {
    pub parameters: HashMap<SourcePin, ParamValue>,
    pub durations: HashMap<SourcePin, DurationData>,
    pub time_updates: HashMap<SourcePin, TimeUpdate>,
    pub poses: HashMap<SourcePin, Pose>,
    pub times: HashMap<SourcePin, f32>,
}

impl OutputCache {
    pub fn clear(&mut self, current_times: HashMap<SourcePin, f32>) {
        self.parameters.clear();
        self.durations.clear();
        self.time_updates.clear();
        self.poses.clear();
        self.times = current_times;
    }

    pub fn clear_for(&mut self, source_pin: &SourcePin) {
        self.parameters.remove(source_pin);
        self.durations.remove(source_pin);
        self.time_updates.remove(source_pin);
        self.poses.remove(source_pin);
        self.times.remove(source_pin);
    }

    pub fn get_parameter(&self, source_pin: &SourcePin) -> Option<&ParamValue> {
        self.parameters.get(source_pin)
    }

    pub fn set_parameter(
        &mut self,
        source_pin: SourcePin,
        value: ParamValue,
    ) -> Option<ParamValue> {
        self.parameters.insert(source_pin, value)
    }

    pub fn get_duration(&self, source_pin: &SourcePin) -> Option<DurationData> {
        self.durations.get(source_pin).cloned()
    }

    pub fn set_duration(
        &mut self,
        source_pin: SourcePin,
        value: DurationData,
    ) -> Option<DurationData> {
        self.durations.insert(source_pin, value)
    }

    pub fn get_time_update(&self, source_pin: &SourcePin) -> Option<&TimeUpdate> {
        self.time_updates.get(source_pin)
    }

    pub fn set_time_update(
        &mut self,
        source_pin: SourcePin,
        value: TimeUpdate,
    ) -> Option<TimeUpdate> {
        self.time_updates.insert(source_pin, value)
    }

    pub fn get_pose(&self, source_pin: &SourcePin) -> Option<&Pose> {
        self.poses.get(source_pin)
    }

    pub fn set_pose(&mut self, source_pin: SourcePin, value: Pose) -> Option<Pose> {
        self.poses.insert(source_pin, value)
    }

    pub fn get_time(&self, source_pin: &SourcePin) -> Option<f32> {
        self.times.get(source_pin).copied()
    }

    pub fn set_time(&mut self, source_pin: SourcePin, value: f32) -> Option<f32> {
        self.times.insert(source_pin, value)
    }
}

#[derive(Reflect, Debug, Default)]
pub struct OutputCaches {
    /// Caches are double buffered
    primary_caches: [OutputCache; 2],
    temp_cache: OutputCache,
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
        &self.primary_caches[self.current_cache]
    }

    pub fn get_other_cache(&self) -> &OutputCache {
        &self.primary_caches[self.other_cache()]
    }

    pub fn get_cache_mut(&mut self) -> &mut OutputCache {
        &mut self.primary_caches[self.current_cache]
    }

    pub fn get_temp_cache(&self) -> &OutputCache {
        &self.temp_cache
    }

    pub fn get_temp_cache_mut(&mut self) -> &mut OutputCache {
        &mut self.temp_cache
    }

    pub fn push(&mut self) {
        let current_times = self.primary_caches[self.current_cache].times.clone();
        self.primary_caches[self.other_cache()].clear(current_times);
        self.temp_cache.clear(HashMap::default());
        self.flip();
    }
}

#[derive(Debug, Default, Reflect)]
pub struct GraphContext {
    pub caches: OutputCaches,
    #[reflect(ignore)]
    subgraph_contexts: HashMap<String, GraphContext>,
}

impl GraphContext {
    pub fn push_caches(&mut self) {
        self.caches.push();

        for (_, sub_ctx) in self.subgraph_contexts.iter_mut() {
            sub_ctx.push_caches();
        }
    }

    pub fn get_parameter(&self, source_pin: &SourcePin) -> Option<&ParamValue> {
        self.caches
            .get_temp_cache()
            .get_parameter(source_pin)
            .or_else(|| self.caches.get_cache().get_parameter(source_pin))
    }

    pub fn get_duration(&self, source_pin: &SourcePin) -> Option<DurationData> {
        self.caches
            .get_temp_cache()
            .get_duration(source_pin)
            .or_else(|| self.caches.get_cache().get_duration(source_pin))
    }

    pub fn get_time_update(&self, source_pin: &SourcePin) -> Option<&TimeUpdate> {
        self.caches
            .get_temp_cache()
            .get_time_update(source_pin)
            .or_else(|| self.caches.get_cache().get_time_update(source_pin))
    }

    pub fn get_prev_time(&self, source_pin: &SourcePin) -> f32 {
        self.caches
            .get_other_cache()
            .get_time(source_pin)
            .unwrap_or(0.)
    }

    pub fn get_time(&self, source_pin: &SourcePin) -> f32 {
        self.caches
            .get_temp_cache()
            .get_time(source_pin)
            .or_else(|| self.caches.get_cache().get_time(source_pin))
            .unwrap_or(0.)
    }

    pub fn get_pose(&self, source_pin: &SourcePin) -> Option<&Pose> {
        self.caches
            .get_temp_cache()
            .get_pose(source_pin)
            .or_else(|| self.caches.get_cache().get_pose(source_pin))
    }

    pub fn clear_temp_cache_for(&mut self, source_pin: &SourcePin) {
        self.caches.get_temp_cache_mut().clear_for(source_pin);
    }

    pub(super) fn context_for_subgraph_or_insert_default(&mut self, node: &str) -> GraphContextRef {
        if !self.subgraph_contexts.contains_key(node) {
            self.subgraph_contexts
                .insert(node.to_string(), GraphContext::default());
        }

        self.subgraph_contexts.get_mut(node).unwrap().into()
    }
}
