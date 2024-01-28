use super::{
    animation_graph::{PinId, TimeState, TimeUpdate},
    parameters::ParamValue,
};
use bevy::{reflect::prelude::*, utils::HashMap};

#[derive(Reflect, Clone, Debug, Default)]
pub struct ParameterCache {
    pub upstream: HashMap<PinId, ParamValue>,
    pub downstream: HashMap<PinId, ParamValue>,
}

#[derive(Reflect, Clone, Debug)]
pub struct DurationCache {
    pub upstream: HashMap<PinId, Option<f32>>,
    pub downstream: Option<Option<f32>>,
}

#[derive(Reflect, Clone, Debug)]
pub struct TimeCache {
    pub upstream: HashMap<PinId, TimeUpdate>,
    pub downstream: TimeState,
}

#[derive(Reflect, Clone, Debug, Default)]
pub struct TimeDependentCache {
    pub upstream: HashMap<PinId, ParamValue>,
    pub downstream: Option<ParamValue>,
}

#[derive(Reflect, Clone, Debug, Default)]
pub struct AnimationCaches {
    pub parameter_cache: Option<ParameterCache>,
    pub duration_cache: Option<DurationCache>,
    pub time_caches: Option<TimeCache>,
    pub time_dependent_caches: Option<TimeDependentCache>,
}
