use crate::{
    core::{
        animation_graph::{NodeId, SourcePin, TargetPin, TimeUpdate},
        duration_data::DurationData,
        pose::Pose,
        prelude::AnimationGraph,
        state_machine::FSMState,
    },
    prelude::DataValue,
};
use bevy::{
    asset::AssetId,
    reflect::prelude::*,
    utils::{HashMap, HashSet},
};

#[derive(Reflect, Debug, Default, Clone)]
pub struct CacheReadFilter {
    pub allow_primary: bool,
    pub allow_temp: bool,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum CacheWriteFilter {
    Primary,
    Temp,
}

impl CacheReadFilter {
    pub const TEMP: Self = Self {
        allow_primary: false,
        allow_temp: true,
    };
    pub const PRIMARY: Self = Self {
        allow_primary: true,
        allow_temp: false,
    };
    pub const FULL: Self = Self {
        allow_primary: true,
        allow_temp: true,
    };

    pub fn for_temp(is_temp: bool) -> Self {
        if is_temp {
            Self::TEMP
        } else {
            Self::PRIMARY
        }
    }
}

impl CacheWriteFilter {
    pub fn for_temp(is_temp: bool) -> Self {
        if is_temp {
            Self::Temp
        } else {
            Self::Primary
        }
    }
}

#[derive(Reflect, Debug, Default, Clone)]
pub struct TimeCache {
    current: HashMap<SourcePin, f32>,
    previous: HashMap<SourcePin, f32>,
}

impl TimeCache {
    pub fn next_frame(&mut self) {
        self.previous = self.current.clone();
    }

    pub fn get(&self, source_pin: &SourcePin) -> Option<f32> {
        self.current.get(source_pin).copied()
    }

    pub fn get_prev(&self, source_pin: &SourcePin) -> Option<f32> {
        self.previous.get(source_pin).copied()
    }

    pub fn save(&mut self, source_pin: SourcePin, time: f32) -> Option<f32> {
        self.current.insert(source_pin, time)
    }
}

#[derive(Reflect, Debug, Default)]
pub struct GraphState {
    pub parameters: HashMap<SourcePin, DataValue>,
    pub durations: HashMap<SourcePin, DurationData>,
    pub time_updates: HashMap<SourcePin, TimeUpdate>,
    pub time_updates_back: HashMap<TargetPin, TimeUpdate>,
    pub poses: HashMap<SourcePin, Pose>,
    pub times: TimeCache,
    pub updated: HashSet<NodeId>,
    pub fsm_state: HashMap<NodeId, FSMState>,
}

impl GraphState {
    pub fn next_frame(&mut self) {
        self.times.next_frame();

        self.parameters.clear();
        self.durations.clear();
        self.time_updates.clear();
        self.time_updates_back.clear();
        self.poses.clear();
        self.updated.clear();
    }

    pub fn get_parameter(&self, source_pin: &SourcePin) -> Option<&DataValue> {
        self.parameters.get(source_pin)
    }

    pub fn set_parameter(&mut self, source_pin: SourcePin, value: DataValue) -> Option<DataValue> {
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

    pub fn get_time_update_back(&self, target_pin: &TargetPin) -> Option<&TimeUpdate> {
        self.time_updates_back.get(target_pin)
    }

    pub fn set_time_update_back(
        &mut self,
        target_pin: TargetPin,
        value: TimeUpdate,
    ) -> Option<TimeUpdate> {
        self.time_updates_back.insert(target_pin, value)
    }

    pub fn get_pose(&self, source_pin: &SourcePin) -> Option<&Pose> {
        self.poses.get(source_pin)
    }

    pub fn set_pose(&mut self, source_pin: SourcePin, value: Pose) -> Option<Pose> {
        self.poses.insert(source_pin, value)
    }

    pub fn get_time(&self, source_pin: &SourcePin) -> Option<f32> {
        self.times.get(source_pin)
    }

    pub fn get_prev_time(&self, source_pin: &SourcePin) -> Option<f32> {
        self.times.get_prev(source_pin)
    }

    pub fn set_time(&mut self, source_pin: SourcePin, value: f32) -> Option<f32> {
        self.times.save(source_pin, value)
    }

    pub fn is_updated(&self, node_id: &NodeId) -> bool {
        self.updated.contains(node_id)
    }

    pub fn set_updated(&mut self, node_id: NodeId) {
        self.updated.insert(node_id);
    }

    pub fn get_fsm_state(&self, node_id: &NodeId) -> Option<&FSMState> {
        self.fsm_state.get(node_id)
    }

    pub fn set_fsm_state(&mut self, node_id: NodeId, state: FSMState) -> Option<FSMState> {
        self.fsm_state.insert(node_id, state)
    }
}

// TODO: Maybe we should consider the multiple caches to be a stack of overlays?
// Might reduce the amount of cloning between frames.
#[derive(Reflect, Debug, Default)]
pub struct GraphStateStack {
    /// Caches are double buffered
    primary_cache: GraphState,
    temp_cache: GraphState,
}

impl GraphStateStack {
    pub fn next_frame(&mut self) {
        self.primary_cache.next_frame();
        self.temp_cache = GraphState::default();
    }

    pub fn get<T>(&self, f: impl Fn(&GraphState) -> Option<T>, opts: CacheReadFilter) -> Option<T> {
        opts.allow_temp
            .then(|| f(&self.temp_cache))
            .flatten()
            .or_else(|| opts.allow_primary.then(|| f(&self.primary_cache)).flatten())
    }

    pub fn set<T>(&mut self, f: impl FnOnce(&mut GraphState) -> T, opts: CacheWriteFilter) -> T {
        match opts {
            CacheWriteFilter::Primary => f(&mut self.primary_cache),
            CacheWriteFilter::Temp => f(&mut self.temp_cache),
        }
    }
}

#[derive(Debug, Reflect)]
pub struct GraphContext {
    pub caches: GraphStateStack,
    graph_id: AssetId<AnimationGraph>,
}

impl GraphContext {
    pub fn new(graph_id: AssetId<AnimationGraph>) -> Self {
        Self {
            caches: GraphStateStack::default(),
            graph_id,
        }
    }

    pub fn next_frame(&mut self) {
        self.caches.next_frame();
    }

    pub fn get_graph_id(&self) -> AssetId<AnimationGraph> {
        self.graph_id
    }
}
