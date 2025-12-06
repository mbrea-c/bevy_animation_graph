use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::Event,
        observer::On,
        system::{ResMut, SystemParam},
        world::World,
    },
};
use bevy_animation_graph::core::{
    context::spec_context::NodeSpec,
    state_machine::high_level::{StateId, StateMachine},
};

use crate::{
    fsm_show::{FsmIndicesMap, make_fsm_indices},
    ui::{actions::saving::DirtyAssets, state_management::global::RegisterStateComponent},
};

#[derive(Debug, Component, Default, Clone)]
pub struct FsmManager;

impl RegisterStateComponent for FsmManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(SetFsmNodeSpec::observe);
        world.add_observer(SetFsmStartState::observe);
    }
}

#[derive(Event)]
pub struct SetFsmNodeSpec {
    pub fsm: Handle<StateMachine>,
    pub new: NodeSpec,
}

impl SetFsmNodeSpec {
    pub fn observe(input: On<SetFsmNodeSpec>, mut fsm_context: FsmContext) {
        fsm_context.provide_mut(&input.fsm, |fsm| {
            fsm.set_input_spec(input.new.clone());
        });
    }
}

#[derive(Event)]
pub struct SetFsmStartState {
    pub fsm: Handle<StateMachine>,
    pub new: StateId,
}

impl SetFsmStartState {
    pub fn observe(input: On<SetFsmStartState>, mut fsm_context: FsmContext) {
        fsm_context.provide_mut(&input.fsm, |fsm| {
            fsm.set_start_state(input.new.clone());
        });
    }
}

#[derive(SystemParam)]
pub struct FsmContext<'w> {
    fsm_assets: ResMut<'w, Assets<StateMachine>>,
    dirty_asets: ResMut<'w, DirtyAssets>,
    fsm_indices: ResMut<'w, FsmIndicesMap>,
}

impl FsmContext<'_> {
    pub fn provide_mut<F>(&mut self, fsm_handle: &Handle<StateMachine>, f: F)
    where
        F: FnOnce(&mut StateMachine),
    {
        self.dirty_asets.add(fsm_handle.clone().untyped());

        let Some(fsm) = self.fsm_assets.get_mut(fsm_handle) else {
            return;
        };

        f(fsm)
    }

    pub fn generate_indices(&mut self, fsm_id: impl Into<AssetId<StateMachine>>) {
        let fsm_id = fsm_id.into();

        let Some(fsm) = self.fsm_assets.get(fsm_id) else {
            return;
        };

        let indices = make_fsm_indices(fsm);

        if let Ok(indices) = indices {
            self.fsm_indices.indices.insert(fsm_id, indices);
        }
    }
}
