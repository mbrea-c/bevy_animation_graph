use super::{
    animation_graph::{AnimationGraph, DEFAULT_OUTPUT_POSE, InputOverlay, PinId, TimeUpdate},
    context::{DeferredGizmos, PassContext},
    edge_data::{AnimationEvent, DataValue, EventQueue, SampledEvent},
    errors::GraphError,
    pose::{BoneId, Pose},
    prelude::GraphContextArena,
    skeleton::Skeleton,
};
use crate::{
    core::ragdoll::{bone_mapping::RagdollBoneMap, definition::Ragdoll, spawning::SpawnedRagdoll},
    prelude::{CustomRelativeDrawCommand, SystemResources},
};
use bevy::{
    asset::prelude::*,
    color::{Color, palettes::css::WHITE},
    ecs::prelude::*,
    platform::collections::HashMap,
    reflect::prelude::*,
};

#[derive(Default, Reflect, Clone, Copy)]
pub enum PlaybackState {
    Paused,
    #[default]
    Play,
    PlayOneFrame,
}

impl PlaybackState {
    pub fn is_paused(&self) -> bool {
        matches!(self, PlaybackState::Paused)
    }
}

#[derive(Reflect, Clone, Default)]
pub enum AnimationSource {
    Graph(Handle<AnimationGraph>),
    Pose(Pose),
    #[default]
    None,
}

impl AnimationSource {
    pub fn is_none(&self) -> bool {
        matches!(self, AnimationSource::None)
    }

    pub fn is_graph(&self) -> bool {
        matches!(self, AnimationSource::Graph(_))
    }

    pub fn is_pose(&self) -> bool {
        matches!(self, AnimationSource::Pose(_))
    }
}

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationGraphPlayer {
    pub(crate) playback_state: PlaybackState,
    pub(crate) animation: AnimationSource,
    pub(crate) skeleton: Handle<Skeleton>,
    pub(crate) ragdoll: Option<Handle<Ragdoll>>,
    pub(crate) ragdoll_bone_map: Option<Handle<RagdollBoneMap>>,
    pub(crate) spawned_ragdoll: Option<SpawnedRagdoll>,
    pub(crate) context_arena: Option<GraphContextArena>,
    pub(crate) elapsed: f32,
    pub(crate) pending_update: TimeUpdate,
    pub(crate) deferred_gizmos: DeferredGizmos,
    pub(crate) debug_draw_bones: Vec<(BoneId, Color)>,
    #[reflect(ignore)]
    pub(crate) debug_draw_custom: Vec<CustomRelativeDrawCommand>,
    pub(crate) entity_map: HashMap<BoneId, Entity>,
    pub(crate) queued_events: EventQueue,
    pub(crate) outputs: HashMap<PinId, DataValue>,

    input_overlay: InputOverlay,
    /// Error that ocurred during graph evaluation in the last frame
    #[reflect(ignore)]
    error: Option<GraphError>,
}

impl AnimationGraphPlayer {
    pub const USER_EVENTS: &'static str = "user events";

    /// Create a new animation graph player, with no graph playing
    pub fn new(skeleton: Handle<Skeleton>) -> Self {
        Self {
            skeleton,
            ..Default::default()
        }
    }

    pub fn get_context_arena(&self) -> Option<&GraphContextArena> {
        self.context_arena.as_ref()
    }

    pub fn set_animation(&mut self, animation: AnimationSource) {
        self.animation = animation;
    }

    pub fn skeleton(&self) -> &Handle<Skeleton> {
        &self.skeleton
    }

    /// Set the animation graph to play
    pub fn with_graph(mut self, animation: Handle<AnimationGraph>) -> Self {
        self.context_arena = Some(GraphContextArena::new(animation.id()));
        self.animation = AnimationSource::Graph(animation);
        self
    }

    /// Clear all input parameters for the animation graph
    pub fn clear_input_parameters(&mut self) {
        self.input_overlay.clear();
    }

    /// Configure an input parameter for the animation graph
    pub fn set_input_parameter(&mut self, parameter_name: impl Into<String>, value: DataValue) {
        self.input_overlay
            .parameters
            .insert(parameter_name.into(), value);
    }

    /// Return an input parameter for the animation graph
    pub fn get_input_parameter(&self, parameter_name: &str) -> Option<DataValue> {
        self.input_overlay.parameters.get(parameter_name).cloned()
    }

    /// Start playing an animation, resetting state of the player.
    /// This will use a linear blending between the previous and the new animation to make a smooth transition.
    pub fn start(&mut self, handle: Handle<AnimationGraph>) -> &mut Self {
        self.context_arena = Some(GraphContextArena::new(handle.id()));
        self.animation = AnimationSource::Graph(handle);
        self.elapsed = 0.;
        self.playback_state = PlaybackState::Play;
        self
    }

    /// Queue an event to make available to the animation graph this frame. This event queue is
    /// cleared every frame.
    pub fn send_event(&mut self, event: AnimationEvent) {
        self.queued_events.events.push(SampledEvent {
            event,
            weight: 1.,
            percentage: 1.,
            track: None,
        });
    }

    pub fn queue_time_update(&mut self, update: TimeUpdate) {
        self.pending_update = self.pending_update.combine(&update);
    }

    /// Query the animation graph with the latest time update and inputs
    pub(crate) fn update(&mut self, system_resources: &SystemResources, root_entity: Entity) {
        self.outputs.clear();
        self.input_overlay.parameters.insert(
            Self::USER_EVENTS.into(),
            std::mem::take(&mut self.queued_events).into(),
        );

        let AnimationSource::Graph(graph_handle) = &self.animation else {
            return;
        };

        let Some(graph) = system_resources.animation_graph_assets.get(graph_handle) else {
            return;
        };

        match graph.query_with_overlay(
            self.pending_update.clone(),
            self.context_arena.as_mut().unwrap(),
            system_resources,
            &self.input_overlay,
            root_entity,
            &self.entity_map,
            &mut self.deferred_gizmos,
        ) {
            Ok(outputs) => {
                self.error = None;
                self.outputs = outputs;
            }
            Err(error) => {
                self.error = Some(error);
            }
        };

        if let Some(pose) = self.outputs.get(DEFAULT_OUTPUT_POSE) {
            let _ = pose.as_pose().map(|p| self.elapsed = p.timestamp);
        }

        self.pending_update = TimeUpdate::Delta(0.);
    }

    pub fn get_pass_context<'a>(
        &'a mut self,
        system_resources: &'a SystemResources,
        root_entity: Entity,
    ) -> PassContext<'a> {
        let context_arena = self.context_arena.as_mut().unwrap();

        PassContext::new(
            context_arena.get_toplevel_id(),
            context_arena,
            system_resources,
            &self.input_overlay,
            root_entity,
            &self.entity_map,
            &mut self.deferred_gizmos,
        )
    }

    pub fn gizmo_for_bones(&mut self, bones: impl IntoIterator<Item = BoneId>) {
        self.debug_draw_bones
            .extend(bones.into_iter().map(|b| (b, WHITE.into())));
    }

    pub fn gizmo_for_bones_with_color(&mut self, bones: impl IntoIterator<Item = (BoneId, Color)>) {
        self.debug_draw_bones.extend(bones);
    }

    pub fn custom_relative_gizmo(&mut self, gizmo: CustomRelativeDrawCommand) {
        self.debug_draw_custom.push(gizmo);
    }

    pub(crate) fn debug_draw_bones(
        &mut self,
        system_resources: &SystemResources,
        root_entity: Entity,
    ) {
        if self.debug_draw_bones.is_empty() && self.debug_draw_custom.is_empty() {
            return;
        }

        let mut bones = std::mem::take(&mut self.debug_draw_bones);
        let mut custom_gizmos = std::mem::take(&mut self.debug_draw_custom);

        let skeleton_handle = self.skeleton.clone();

        let ctx = self
            .get_pass_context(system_resources, root_entity)
            .with_debugging(true);

        let Some(skeleton) = ctx.resources.skeleton_assets.get(&skeleton_handle) else {
            return;
        };

        for (bone_id, color) in bones.drain(..) {
            ctx.gizmos()
                .bone_gizmo(bone_id, color.into(), skeleton, None);
        }

        for custom_cmd in custom_gizmos.drain(..) {
            ctx.gizmos()
                .relative_custom_gizmo(custom_cmd, skeleton, None);
        }
    }

    pub fn pause(&mut self) {
        self.playback_state = PlaybackState::Paused;
    }

    pub fn resume(&mut self) {
        self.playback_state = PlaybackState::Play;
    }

    pub fn is_paused(&self) -> bool {
        self.playback_state.is_paused()
    }

    pub fn playback_state(&self) -> PlaybackState {
        self.playback_state
    }

    pub fn play_one_frame(&mut self) {
        self.playback_state = PlaybackState::PlayOneFrame;
    }

    pub fn reset(&mut self) {
        self.pending_update = TimeUpdate::Absolute(0.);
    }

    pub fn seek(&mut self, time: f32) {
        self.pending_update = TimeUpdate::Absolute(time);
    }

    pub fn get_animation_source(&self) -> &AnimationSource {
        &self.animation
    }

    /// If graph evaluation produced an error in the last frame return the error, otherwise return
    /// `None`.
    pub fn get_error(&self) -> Option<GraphError> {
        self.error.clone()
    }

    pub fn get_outputs(&self) -> &HashMap<PinId, DataValue> {
        &self.outputs
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Gets the default output pose from the graph, or the forced pose if set.
    pub fn get_default_output_pose(&self) -> Option<&Pose> {
        match &self.animation {
            AnimationSource::Graph(_) => self.outputs.get(DEFAULT_OUTPUT_POSE)?.as_pose().ok(),
            AnimationSource::Pose(pose) => Some(pose),
            AnimationSource::None => None,
        }
    }
}
