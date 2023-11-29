use super::caches::AnimationCaches;
use bevy::{reflect::prelude::*, utils::HashMap};

#[derive(Reflect)]
pub struct GraphContext {
    /// Caches are double buffered
    caches: [HashMap<String, AnimationCaches>; 2],
    current_cache: usize,
}

impl Default for GraphContext {
    fn default() -> Self {
        Self {
            caches: [HashMap::default(), HashMap::default()],
            current_cache: 0,
        }
    }
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
}
