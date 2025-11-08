use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::Event,
        observer::On,
        system::{Commands, Res},
        world::World,
    },
};
use bevy_animation_graph::prelude::AnimatedScene;

use crate::ui::global_state::{
    RegisterStateComponent, SetOrInsertEvent,
    active_ragdoll::{ActiveRagdoll, SetActiveRagdoll},
    active_skeleton::{ActiveSkeleton, SetActiveSkeleton},
    observe_clear_global_state, observe_set_or_insert_event,
};

#[derive(Debug, Component, Default, Clone)]
pub struct ActiveScene {
    pub handle: Handle<AnimatedScene>,
}

impl RegisterStateComponent for ActiveScene {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(observe_set_or_insert_event::<ActiveScene, SetActiveScene>);
        world.add_observer(observe_clear_global_state::<Self>);
        world.add_observer(set_skeleton_from_scene);
    }
}

#[derive(Event)]
pub struct SetActiveScene {
    pub new: ActiveScene,
}

impl SetOrInsertEvent for SetActiveScene {
    type Target = ActiveScene;

    fn get_component(&self) -> Self::Target {
        self.new.clone()
    }
}

fn set_skeleton_from_scene(
    event: On<SetActiveScene>,
    mut commands: Commands,
    animated_scene_asset: Res<Assets<AnimatedScene>>,
) {
    if let Some(scn) = animated_scene_asset.get(&event.new.handle) {
        commands.trigger(SetActiveSkeleton {
            new: ActiveSkeleton {
                handle: scn.skeleton.clone(),
            },
        });

        if let Some(ragdoll) = scn.ragdoll.clone() {
            commands.trigger(SetActiveRagdoll {
                new: ActiveRagdoll { handle: ragdoll },
            });
        }
    }
}
