use super::{
    animation_clip::GraphClip,
    animation_graph::{AnimationGraph, NodeId, ParamValue, PinId, SourcePin, TargetPin, TimeState},
    frame::PoseFrame,
};
use crate::prelude::DurationData;
use bevy::{asset::Assets, reflect::prelude::*, utils::HashMap};

pub struct PassContext<'a> {
    pub node_id: &'a NodeId,
    pub context: &'a mut GraphContext,
    pub context_tmp: GraphContextTmp<'a>,
    pub edges: &'a HashMap<TargetPin, SourcePin>,
}

impl<'a> PassContext<'a> {
    pub fn new(
        node_id: &'a NodeId,
        context: &'a mut GraphContext,
        context_tmp: GraphContextTmp<'a>,
        edges: &'a HashMap<TargetPin, SourcePin>,
    ) -> Self {
        Self {
            node_id,
            context,
            context_tmp,
            edges,
        }
    }

    pub fn parameter_back(&self, pin_id: impl Into<PinId>) -> ParamValue {
        let target_pin = TargetPin::NodeParameter(self.node_id.clone(), pin_id.into());
        let source_pin = self
            .edges
            .get(&target_pin)
            .unwrap_or_else(|| panic!("Pin {target_pin:?} is disconnected!"));

        self.context
            .get_cached_parameter(source_pin)
            .unwrap_or_else(|| panic!("Parameter not cached at {source_pin:?}"))
            .clone()
    }

    pub fn duration_back(&self, pin_id: impl Into<PinId>) -> DurationData {
        let target_pin = TargetPin::NodePose(self.node_id.clone(), pin_id.into());
        let source_pin = self
            .edges
            .get(&target_pin)
            .unwrap_or_else(|| panic!("Pin {target_pin:?} is disconnected!"));

        self.context
            .get_cached_duration(source_pin)
            .unwrap_or_else(|| panic!("Duration not cached at {source_pin:?}"))
            .clone()
    }

    pub fn time_fwd(&self) -> TimeState {
        let source_pin = SourcePin::NodePose(self.node_id.clone());

        self.context
            .get_cached_time(&source_pin)
            .unwrap_or_else(|| panic!("Time not cached at {source_pin:?}"))
            .clone()
    }

    pub fn prev_time_fwd_opt(&self) -> Option<TimeState> {
        let source_pin = SourcePin::NodePose(self.node_id.clone());
        self.context.old_cached_time(&source_pin).cloned()
    }
}

pub struct SpecContext<'a> {
    pub context: &'a mut GraphContext,
    pub context_tmp: GraphContextTmp<'a>,
}

impl<'a> SpecContext<'a> {
    pub fn new(context: &'a mut GraphContext, context_tmp: GraphContextTmp<'a>) -> Self {
        Self {
            context,
            context_tmp,
        }
    }
}

#[derive(Reflect, Debug, Default)]
pub struct GraphCache {
    pub parameters: HashMap<SourcePin, ParamValue>,
    pub durations: HashMap<SourcePin, DurationData>,
    pub times: HashMap<SourcePin, TimeState>,
    pub poses: HashMap<SourcePin, PoseFrame>,
}

impl GraphCache {
    pub fn clear(&mut self) {
        self.parameters.clear();
        self.durations.clear();
        self.times.clear();
        self.poses.clear();
    }
}

#[derive(Reflect, Debug, Default)]
pub struct GraphContext {
    /// Caches are double buffered
    caches: [GraphCache; 2],
    current_cache: usize,
    #[reflect(ignore)]
    subgraph_contexts: HashMap<String, GraphContext>,
}

/// Contains temprary data such as references to assets, gizmos, etc.
#[derive(Clone, Copy)]
pub struct GraphContextTmp<'a> {
    pub graph_clip_assets: &'a Assets<GraphClip>,
    pub animation_graph_assets: &'a Assets<AnimationGraph>,
}

impl GraphContext {
    pub fn get_cache(&self) -> &GraphCache {
        &self.caches[self.current_cache]
    }

    pub fn get_other_cache(&self) -> &GraphCache {
        &self.caches[self.other_cache()]
    }

    pub fn get_cache_mut(&mut self) -> &mut GraphCache {
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

        for (_, sub_ctx) in self.subgraph_contexts.iter_mut() {
            sub_ctx.push_caches();
        }
    }

    pub fn get_cached_parameter(&self, source_pin: &SourcePin) -> Option<&ParamValue> {
        self.get_cache().parameters.get(source_pin)
    }

    pub fn insert_cached_parameter(
        &mut self,
        source_pin: SourcePin,
        value: ParamValue,
    ) -> Option<ParamValue> {
        self.get_cache_mut().parameters.insert(source_pin, value)
    }

    pub fn get_cached_duration(&self, source_pin: &SourcePin) -> Option<Option<f32>> {
        self.get_cache().durations.get(source_pin).copied()
    }

    pub fn insert_cached_duration(
        &mut self,
        source_pin: SourcePin,
        value: Option<f32>,
    ) -> Option<Option<f32>> {
        self.get_cache_mut().durations.insert(source_pin, value)
    }

    pub fn get_cached_time(&self, source_pin: &SourcePin) -> Option<&TimeState> {
        self.get_cache().times.get(source_pin)
    }

    pub fn old_cached_time(&self, source_pin: &SourcePin) -> Option<&TimeState> {
        self.get_other_cache().times.get(source_pin)
    }

    pub fn insert_cached_time(
        &mut self,
        source_pin: SourcePin,
        value: TimeState,
    ) -> Option<TimeState> {
        self.get_cache_mut().times.insert(source_pin, value)
    }

    pub fn get_cached_pose(&self, source_pin: &SourcePin) -> Option<&PoseFrame> {
        self.get_cache().poses.get(source_pin)
    }

    pub fn insert_cached_pose(
        &mut self,
        source_pin: SourcePin,
        value: PoseFrame,
    ) -> Option<PoseFrame> {
        self.get_cache_mut().poses.insert(source_pin, value)
    }

    pub fn context_for_subgraph_or_insert_default(&mut self, node: &str) -> &mut GraphContext {
        if !self.subgraph_contexts.contains_key(node) {
            self.subgraph_contexts
                .insert(node.to_string(), GraphContext::default());
        }

        self.subgraph_contexts.get_mut(node).unwrap()
    }
}
