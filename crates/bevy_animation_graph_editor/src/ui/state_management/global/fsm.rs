use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::Event,
        observer::On,
        system::{ResMut, SystemParam},
        world::World,
    },
    math::Vec2,
    platform::collections::HashSet,
};
use bevy_animation_graph::core::{
    context::spec_context::NodeSpec,
    state_machine::high_level::{StateId, StateMachine},
};

use crate::ui::{actions::saving::DirtyAssets, state_management::global::RegisterStateComponent};

#[derive(Debug, Component, Default, Clone)]
pub struct FsmManager;

impl RegisterStateComponent for FsmManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(SetFsmNodeSpec::observe);
        world.add_observer(SetFsmStartState::observe);
        world.add_observer(MoveStates::observe);
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

#[derive(Event)]
pub struct MoveStates {
    pub fsm: Handle<StateMachine>,
    pub states: HashSet<StateId>,
    pub delta: Vec2,
}

impl MoveStates {
    pub fn observe(move_states: On<MoveStates>, mut ctx: FsmContext) {
        ctx.provide_mut(&move_states.fsm, |fsm| {
            for state_id in &move_states.states {
                fsm.extra.move_state(*state_id, move_states.delta);
            }
        });
    }
}

#[derive(SystemParam)]
pub struct FsmContext<'w> {
    fsm_assets: ResMut<'w, Assets<StateMachine>>,
    dirty_asets: ResMut<'w, DirtyAssets>,
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
}
