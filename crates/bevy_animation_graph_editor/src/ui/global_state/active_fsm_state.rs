use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        event::Event,
        observer::Trigger,
        query::With,
        system::{Commands, Single},
    },
};
use bevy_animation_graph::core::state_machine::high_level::{StateId, StateMachine};

use crate::ui::global_state::GlobalState;

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveFsmState {
    pub handle: Handle<StateMachine>,
    pub state: StateId,
}

#[derive(Event)]
pub struct SetActiveFsmState {
    pub new: ActiveFsmState,
}

impl SetActiveFsmState {
    pub fn observe(
        new_state: Trigger<SetActiveFsmState>,
        global_state: Single<(Entity, Option<&mut ActiveFsmState>), With<GlobalState>>,
        mut commands: Commands,
    ) {
        let (entity, old_state) = global_state.into_inner();

        if let Some(mut old_state) = old_state {
            *old_state = new_state.event().new.clone();
        } else {
            commands
                .entity(entity)
                .insert(new_state.event().new.clone());
        }
    }
}
