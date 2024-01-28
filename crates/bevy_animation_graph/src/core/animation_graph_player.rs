use super::{
    animation_graph::{AnimationGraph, InputOverlay, TimeState, TimeUpdate},
    context::DeferredGizmos,
    parameters::ParamValue,
    pose::{BoneId, Pose},
};
use crate::prelude::{GraphContext, SystemResources};
use bevy::{asset::prelude::*, ecs::prelude::*, reflect::prelude::*, utils::HashMap};

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationGraphPlayer {
    pub(crate) paused: bool,
    pub(crate) animation: Option<Handle<AnimationGraph>>,
    pub(crate) elapsed: TimeState,
    pub(crate) pending_update: Option<TimeUpdate>,
    pub(crate) context: GraphContext,
    pub(crate) deferred_gizmos: DeferredGizmos,

    input_overlay: InputOverlay,
}

impl AnimationGraphPlayer {
    /// Create a new animation graph player, with no graph playing
    pub fn new() -> Self {
        Self {
            paused: false,
            animation: None,
            elapsed: TimeState::default(),
            pending_update: None,
            context: GraphContext::default(),
            deferred_gizmos: DeferredGizmos::default(),

            input_overlay: InputOverlay::default(),
        }
    }

    /// Set the animation graph to play
    pub fn with_graph(mut self, animation: Handle<AnimationGraph>) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Clear all input parameters for the animation graph
    pub fn clear_input_parameters(&mut self) {
        self.input_overlay.clear();
    }

    /// Configure an input parameter for the animation graph
    pub fn set_input_parameter(&mut self, parameter_name: impl Into<String>, value: ParamValue) {
        self.input_overlay
            .parameters
            .insert(parameter_name.into(), value);
    }

    /// Return an input parameter for the animation graph
    pub fn get_input_parameter(&self, parameter_name: &str) -> Option<ParamValue> {
        self.input_overlay.parameters.get(parameter_name).cloned()
    }

    /// Start playing an animation, resetting state of the player.
    /// This will use a linear blending between the previous and the new animation to make a smooth transition.
    pub fn start(&mut self, handle: Handle<AnimationGraph>) -> &mut Self {
        self.animation = Some(handle);
        self.elapsed = TimeState::default();
        self.paused = false;
        self
    }

    /// Query the animation graph with the latest time update and inputs
    pub(crate) fn query(
        &mut self,
        context_tmp: &SystemResources,
        root_entity: Entity,
        entity_map: &HashMap<BoneId, Entity>,
    ) -> Option<Pose> {
        let Some(graph_handle) = &self.animation else {
            return None;
        };

        let Some(graph) = context_tmp.animation_graph_assets.get(graph_handle) else {
            return None;
        };

        Some(graph.query_with_overlay(
            self.elapsed.update,
            &mut self.context,
            context_tmp,
            &self.input_overlay,
            root_entity,
            entity_map,
            &mut self.deferred_gizmos,
        ))
    }

    pub fn pause(&mut self) -> &mut Self {
        self.paused = true;
        self
    }

    pub fn resume(&mut self) -> &mut Self {
        self.paused = false;
        self
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn reset(&mut self) -> &mut Self {
        self.pending_update = Some(TimeUpdate::Absolute(0.));
        self
    }

    pub fn get_animation_graph(&self) -> Option<Handle<AnimationGraph>> {
        self.animation.clone()
    }
}
