use super::animation_graph::{EdgePath, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate};
use bevy::{reflect::prelude::*, utils::HashMap};

#[derive(Reflect, Clone, Debug, Default)]
pub struct ParameterCache {
    pub upstream: HashMap<NodeInput, EdgeValue>,
    pub downstream: HashMap<NodeOutput, EdgeValue>,
}

#[derive(Reflect, Clone, Debug)]
pub struct DurationCache {
    pub upstream: HashMap<NodeInput, Option<f32>>,
    pub downstream: HashMap<NodeOutput, Option<f32>>,
}

#[derive(Reflect, Clone, Debug)]
pub struct TimeCache {
    pub upstream: HashMap<NodeOutput, TimeUpdate>,
    pub downstream: TimeState,
}

#[derive(Reflect, Clone, Debug, Default)]
pub struct TimeDependentCache {
    pub upstream: HashMap<NodeInput, EdgeValue>,
    pub downstream: HashMap<NodeOutput, EdgeValue>,
}

#[derive(Reflect, Clone, Debug)]
pub struct AnimationCaches {
    pub parameter_cache: Option<ParameterCache>,
    pub duration_cache: Option<DurationCache>,
    pub time_caches: HashMap<EdgePath, TimeCache>,
    pub time_dependent_caches: HashMap<EdgePath, TimeDependentCache>,
}

#[derive(Reflect, Clone, Debug)]
pub struct EdgePathCache<'a> {
    pub parameter_cache: &'a ParameterCache,
    pub duration_cache: &'a DurationCache,
    pub time_cache: &'a TimeCache,
    pub time_dependent_cache: &'a TimeDependentCache,
}

impl AnimationCaches {
    pub fn get<'a>(&'a self, key: &EdgePath) -> Option<EdgePathCache<'a>> {
        let parameter_cache = self.parameter_cache.as_ref()?;
        let duration_cache = self.duration_cache.as_ref()?;
        let time_cache = self.time_caches.get(key)?;
        let time_dependent_cache = self.time_dependent_caches.get(key)?;

        Some(EdgePathCache {
            parameter_cache,
            duration_cache,
            time_cache,
            time_dependent_cache,
        })
    }
}

impl Default for AnimationCaches {
    fn default() -> Self {
        Self {
            parameter_cache: None,
            duration_cache: None,
            time_caches: HashMap::new(),
            time_dependent_caches: HashMap::new(),
        }
    }
}
