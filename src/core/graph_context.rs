use super::{
    animation_clip::GraphClip,
    animation_graph::{AnimationGraph, EdgePath},
    caches::{AnimationCaches, DurationCache, ParameterCache, TimeCache, TimeDependentCache},
};
use bevy::{asset::Assets, reflect::prelude::*, utils::HashMap};

#[derive(Reflect, Debug)]
pub struct GraphContext {
    /// Caches are double buffered
    caches: [HashMap<String, AnimationCaches>; 2],
    current_cache: usize,
    #[reflect(ignore)]
    subgraph_contexts: HashMap<String, GraphContext>,
}

impl Default for GraphContext {
    fn default() -> Self {
        Self {
            caches: [HashMap::default(), HashMap::default()],
            current_cache: 0,
            subgraph_contexts: HashMap::default(),
        }
    }
}

/// Contains temprary data such as references to assets, gizmos, etc.
pub struct GraphContextTmp<'a> {
    pub graph_clip_assets: &'a Assets<GraphClip>,
    pub animation_graph_assets: &'a Assets<AnimationGraph>,
}

impl GraphContext {
    pub fn get_cache(&self) -> &HashMap<String, AnimationCaches> {
        &self.caches[self.current_cache]
    }

    pub fn get_other_cache(&self) -> &HashMap<String, AnimationCaches> {
        &self.caches[self.other_cache()]
    }

    pub fn get_cache_mut(&mut self) -> &mut HashMap<String, AnimationCaches> {
        &mut self.caches[self.current_cache]
    }

    pub fn flip_caches(&mut self) {
        self.current_cache = self.other_cache();
    }

    pub fn other_cache(&self) -> usize {
        (self.current_cache + 1) % 2
    }

    pub fn push_caches(&mut self) {
        self.caches[self.other_cache()].clear();
        self.flip_caches();
    }

    pub fn get_node_cache(&self, node: &str) -> Option<&AnimationCaches> {
        self.get_cache().get(node)
    }

    pub fn get_node_other_cache(&self, node: &str) -> Option<&AnimationCaches> {
        self.get_other_cache().get(node)
    }

    pub fn get_parameters(&self, node: &str) -> Option<&ParameterCache> {
        self.get_node_cache(node)
            .and_then(|c| c.parameter_cache.as_ref())
    }

    pub fn get_durations(&self, node: &str) -> Option<&DurationCache> {
        self.get_node_cache(node)
            .and_then(|c| c.duration_cache.as_ref())
    }

    pub fn get_times(&self, node: &str, path: &EdgePath) -> Option<&TimeCache> {
        self.get_node_cache(node)
            .and_then(|c| c.time_caches.get(path))
    }

    pub fn get_time_dependent(&self, node: &str, path: &EdgePath) -> Option<&TimeDependentCache> {
        self.get_node_cache(node)
            .and_then(|c| c.time_dependent_caches.get(path))
    }

    pub fn get_other_parameters(&self, node: &str) -> Option<&ParameterCache> {
        self.get_node_other_cache(node)
            .map_or(None, |c| c.parameter_cache.as_ref())
    }

    pub fn get_other_durations(&self, node: &str) -> Option<&DurationCache> {
        self.get_node_other_cache(node)
            .map_or(None, |c| c.duration_cache.as_ref())
    }

    pub fn get_other_times(&self, node: &str, path: &EdgePath) -> Option<&TimeCache> {
        self.get_node_other_cache(node)
            .and_then(|c| c.time_caches.get(path))
    }

    pub fn get_other_time_dependent(
        &self,
        node: &str,
        path: &EdgePath,
    ) -> Option<&TimeDependentCache> {
        self.get_node_other_cache(node)
            .and_then(|c| c.time_dependent_caches.get(path))
    }

    pub fn get_node_cache_mut(&mut self, node: &str) -> Option<&mut AnimationCaches> {
        self.get_cache_mut().get_mut(node)
    }

    pub fn get_node_cache_or_insert_default(&mut self, node: &str) -> &mut AnimationCaches {
        let caches = self.get_cache_mut();
        if !caches.contains_key(node) {
            caches.insert(node.to_string(), AnimationCaches::default());
        }

        caches.get_mut(node).unwrap()
    }

    pub fn context_for_subgraph_or_insert_default(&mut self, node: &str) -> &mut GraphContext {
        if !self.subgraph_contexts.contains_key(node) {
            self.subgraph_contexts
                .insert(node.to_string(), GraphContext::default());
        }

        self.subgraph_contexts.get_mut(node).unwrap()
    }
}
