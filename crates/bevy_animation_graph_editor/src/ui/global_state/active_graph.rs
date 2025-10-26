use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        event::Event,
        observer::Trigger,
        query::With,
        system::{Commands, Single},
        world::World,
    },
};
use bevy_animation_graph::prelude::AnimationGraph;

use crate::ui::global_state::{GlobalState, RegisterGlobalState};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveGraph {
    pub handle: Handle<AnimationGraph>,
}

impl RegisterGlobalState for ActiveGraph {
    fn register(world: &mut World, global_state_entity: Entity) {
        world.add_observer(SetActiveGraph::observe);
    }
}

#[derive(Event)]
pub struct SetActiveGraph {
    pub new: ActiveGraph,
}

impl SetActiveGraph {
    pub fn observe(
        new_state: Trigger<SetActiveGraph>,
        global_state: Single<(Entity, Option<&mut ActiveGraph>), With<GlobalState>>,
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
