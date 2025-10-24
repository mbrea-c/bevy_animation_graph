use bevy::{
    asset::Handle,
    ecs::{component::Component, event::Event, observer::Trigger, query::With, system::Single},
};
use bevy_animation_graph::prelude::AnimatedScene;

use crate::ui::global_state::GlobalState;

#[derive(Debug, Component, Default)]
pub struct ActiveScene {
    pub handle: Option<Handle<AnimatedScene>>,
}

#[derive(Event)]
pub struct SetActiveScene {
    pub handle: Option<Handle<AnimatedScene>>,
}

impl SetActiveScene {
    pub fn observe(
        set_active_scene: Trigger<SetActiveScene>,
        global_state: Single<&mut ActiveScene, With<GlobalState>>,
    ) {
        global_state.into_inner().handle = set_active_scene.event().handle.clone();
    }
}
