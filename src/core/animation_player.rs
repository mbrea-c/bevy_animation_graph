use super::{
    animation_graph::{AnimationGraph, TimeState, TimeUpdate},
    graph_context::GraphContext,
};
use bevy::{asset::prelude::*, ecs::prelude::*, reflect::prelude::*};

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationPlayer {
    pub(crate) paused: bool,
    pub(crate) animation: Option<Handle<AnimationGraph>>,
    pub(crate) elapsed: TimeState,
    pub(crate) pending_update: Option<TimeUpdate>,
    pub(crate) context: GraphContext,
}

impl AnimationPlayer {
    /// Start playing an animation, resetting state of the player.
    /// This will use a linear blending between the previous and the new animation to make a smooth transition.
    pub fn start(&mut self, handle: Handle<AnimationGraph>) -> &mut Self {
        self.animation = Some(handle);
        self.elapsed = TimeState::default();
        self.paused = false;
        self
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
