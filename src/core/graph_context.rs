use super::{
    animation_clip::GraphClip,
    animation_graph::{
        AnimationGraph, InputOverlay, NodeId, ParamValue, PinId, SourcePin, TargetPin, TimeUpdate,
    },
    duration_data::DurationData,
    frame::PoseFrame,
};
use bevy::{asset::Assets, reflect::prelude::*, utils::HashMap};

#[derive(Clone, Copy)]
pub struct NodeContext<'a> {
    pub node_id: &'a NodeId,
    pub graph: &'a AnimationGraph,
}

#[derive(Clone)]
struct PassContextRef<'a> {
    ctx: *const PassContext<'a>,
}

impl<'a> From<&'a PassContext<'a>> for PassContextRef<'a> {
    fn from(value: &'a PassContext) -> Self {
        Self { ctx: value }
    }
}

impl<'a> PassContextRef<'a> {
    pub fn as_ref(&self) -> PassContext<'a> {
        unsafe { self.ctx.as_ref().unwrap().clone() }
    }
}

#[derive(Clone)]
pub struct GraphParent<'a> {
    /// Identifies which node this graph represents
    pub node_context: NodeContext<'a>,
    pub context: GraphContextRef,
    pub overlay: &'a InputOverlay,
}

#[derive(Clone)]
pub struct PassContext<'a> {
    pub context: GraphContextRef,
    pub context_tmp: GraphContextTmp<'a>,
    pub overlay: &'a InputOverlay,
    pub node_context: Option<NodeContext<'a>>,
    parent: Option<PassContextRef<'a>>,
}

impl<'a> PassContext<'a> {
    pub fn new(
        context: GraphContextRef,
        context_tmp: GraphContextTmp<'a>,
        overlay: &'a InputOverlay,
    ) -> Self {
        Self {
            context,
            context_tmp,
            overlay,
            node_context: None,
            parent: None,
        }
    }

    pub fn with_node(&self, node_id: &'a NodeId, graph: &'a AnimationGraph) -> Self {
        Self {
            context: self.context.clone(),
            context_tmp: self.context_tmp,
            overlay: self.overlay,
            node_context: Some(NodeContext { node_id, graph }),
            parent: self.parent.clone(),
        }
    }

    pub fn without_node(&self) -> Self {
        Self {
            context: self.context.clone(),
            context_tmp: self.context_tmp,
            overlay: self.overlay,
            node_context: None,
            parent: self.parent.clone(),
        }
    }

    pub fn child(&'a self, overlay: &'a InputOverlay) -> Self {
        Self {
            context: self
                .context
                .as_mut()
                .context_for_subgraph_or_insert_default(&self.node_context.unwrap().node_id),
            context_tmp: self.context_tmp,
            overlay,
            node_context: self.node_context,
            parent: Some(self.into()),
        }
    }

    pub fn parent(&'a self) -> Self {
        self.parent.as_ref().unwrap().as_ref()
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    pub fn parameter_back(&mut self, pin_id: impl Into<PinId>) -> ParamValue {
        let node_ctx = self.node_context.unwrap();
        let target_pin = TargetPin::NodeParameter(node_ctx.node_id.clone(), pin_id.into());
        node_ctx
            .graph
            .get_parameter(target_pin, self.without_node())
            .unwrap()
    }

    pub fn parameter_back_opt(&mut self, pin_id: impl Into<PinId>) -> Option<ParamValue> {
        let node_ctx = self.node_context.unwrap();
        let target_pin = TargetPin::NodeParameter(node_ctx.node_id.clone(), pin_id.into());
        node_ctx
            .graph
            .get_parameter(target_pin, self.without_node())
    }

    pub fn duration_back(&mut self, pin_id: impl Into<PinId>) -> DurationData {
        let node_ctx = self.node_context.unwrap();
        let target_pin = TargetPin::NodePose(node_ctx.node_id.clone(), pin_id.into());
        node_ctx.graph.get_duration(target_pin, self.without_node())
    }

    pub fn pose_back(&mut self, pin_id: impl Into<PinId>, time_update: TimeUpdate) -> PoseFrame {
        let node_ctx = self.node_context.unwrap();
        let target_pin = TargetPin::NodePose(node_ctx.node_id.clone(), pin_id.into());
        node_ctx
            .graph
            .get_pose(time_update, target_pin, self.without_node())
    }

    pub fn time_update_fwd(&self) -> TimeUpdate {
        let node_ctx = self.node_context.unwrap();
        let source_pin = SourcePin::NodePose(node_ctx.node_id.clone());

        *self
            .context
            .as_mut()
            .get_time_update(&source_pin)
            .unwrap_or_else(|| panic!("Time update not cached at {source_pin:?}"))
    }

    pub fn prev_time_fwd(&self) -> f32 {
        let node_ctx = self.node_context.unwrap();
        let source_pin = SourcePin::NodePose(node_ctx.node_id.clone());
        self.context.as_mut().get_prev_time(&source_pin)
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

// Internal mutability lets goooooooooo
// May god have mercy on us
#[derive(Clone)]
pub struct GraphContextRef {
    context: *mut GraphContext,
}

impl From<&mut GraphContext> for GraphContextRef {
    fn from(value: &mut GraphContext) -> Self {
        Self { context: value }
    }
}

impl GraphContextRef {
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut GraphContext {
        unsafe { self.context.as_mut().unwrap() }
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
pub struct GraphContextTmp<'a> {
    pub graph_clip_assets: &'a Assets<GraphClip>,
    pub animation_graph_assets: &'a Assets<AnimationGraph>,
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

    pub fn context_for_subgraph_or_insert_default(&mut self, node: &str) -> GraphContextRef {
        if !self.subgraph_contexts.contains_key(node) {
            self.subgraph_contexts
                .insert(node.to_string(), GraphContext::default());
        }

        self.subgraph_contexts.get_mut(node).unwrap().into()
    }
}
