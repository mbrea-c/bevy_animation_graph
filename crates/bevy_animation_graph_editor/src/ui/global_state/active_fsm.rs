use bevy::{
    asset::Handle,
    ecs::{component::Component, event::Event, observer::Trigger, query::With, system::Single},
};
use bevy_animation_graph::core::state_machine::high_level::StateMachine;

use crate::ui::global_state::GlobalState;

#[derive(Debug, Component, Default)]
pub struct ActiveFsm {
    pub handle: Option<Handle<StateMachine>>,
}

#[derive(Event)]
pub struct SetActiveFsm {
    pub handle: Option<Handle<StateMachine>>,
}

impl SetActiveFsm {
    pub fn observe(
        set_active_fsm: Trigger<SetActiveFsm>,
        global_state: Single<&mut ActiveFsm, With<GlobalState>>,
    ) {
        global_state.into_inner().handle = set_active_fsm.event().handle.clone();
    }
}
