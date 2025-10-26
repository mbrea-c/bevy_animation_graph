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
use bevy_animation_graph::core::state_machine::high_level::{StateMachine, TransitionId};

use crate::ui::global_state::GlobalState;

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveFsmTransition {
    pub handle: Handle<StateMachine>,
    pub transition: TransitionId,
}

#[derive(Event)]
pub struct SetActiveFsmTransition {
    pub new: ActiveFsmTransition,
}

impl SetActiveFsmTransition {
    pub fn observe(
        new_state: Trigger<SetActiveFsmTransition>,
        global_state: Single<(Entity, Option<&mut ActiveFsmTransition>), With<GlobalState>>,
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
