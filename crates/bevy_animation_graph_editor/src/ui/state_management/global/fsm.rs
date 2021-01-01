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
    state_machine::high_level::{
        DirectTransition, DirectTransitionId, State, StateId, StateMachine,
    },
};

use crate::ui::{actions::saving::DirtyAssets, state_management::global::RegisterStateComponent};

#[derive(Debug, Component, Default, Clone)]
pub struct FsmManager;

impl RegisterStateComponent for FsmManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(SetFsmNodeSpec::observe);
        world.add_observer(SetFsmStartState::observe);
        world.add_observer(MoveStates::observe);
        world.add_observer(CreateState::observe);
        world.add_observer(DeleteStates::observe);
        world.add_observer(CreateDirectTransition::observe);
        world.add_observer(DeleteDirectTransitions::observe);
        world.add_observer(UpdateState::observe);
        world.add_observer(UpdateDirectTransition::observe);
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

#[derive(Event)]
pub struct CreateState {
    pub fsm: Handle<StateMachine>,
    pub state: State,
}

impl CreateState {
    pub fn observe(create_state: On<CreateState>, mut ctx: FsmContext) {
        ctx.provide_mut(&create_state.fsm, |fsm| {
            fsm.add_state(create_state.state.clone());
        });
    }
}

#[derive(Event)]
pub struct CreateDirectTransition {
    pub fsm: Handle<StateMachine>,
    pub transition: DirectTransition,
}

impl CreateDirectTransition {
    pub fn observe(create_transition: On<CreateDirectTransition>, mut ctx: FsmContext) {
        ctx.provide_mut(&create_transition.fsm, |fsm| {
            // For now, nothing happens if creation fails
            // might want to notify or something eventually
            let _ = fsm.add_transition_from_ui(create_transition.transition.clone());
        });
    }
}

#[derive(Event)]
pub struct DeleteStates {
    pub fsm: Handle<StateMachine>,
    pub states: HashSet<StateId>,
}

impl DeleteStates {
    pub fn observe(delete_states: On<DeleteStates>, mut ctx: FsmContext) {
        ctx.provide_mut(&delete_states.fsm, |fsm| {
            for state_id in &delete_states.states {
                let _ = fsm.delete_state(*state_id);
            }
        });
    }
}

#[derive(Event)]
pub struct DeleteDirectTransitions {
    pub fsm: Handle<StateMachine>,
    pub transitions: HashSet<DirectTransitionId>,
}

impl DeleteDirectTransitions {
    pub fn observe(delete_states: On<DeleteDirectTransitions>, mut ctx: FsmContext) {
        ctx.provide_mut(&delete_states.fsm, |fsm| {
            for transition_id in &delete_states.transitions {
                let _ = fsm.delete_transition(*transition_id);
            }
        });
    }
}

#[derive(Event)]
pub struct UpdateState {
    pub fsm: Handle<StateMachine>,
    pub state: State,
}

impl UpdateState {
    pub fn observe(update_state: On<UpdateState>, mut ctx: FsmContext) {
        ctx.provide_mut(&update_state.fsm, |fsm| {
            let _ = fsm.update_state(update_state.state.id, update_state.state.clone());
        });
    }
}

#[derive(Event)]
pub struct UpdateDirectTransition {
    pub fsm: Handle<StateMachine>,
    pub transition: DirectTransition,
}

impl UpdateDirectTransition {
    pub fn observe(update_transition: On<UpdateDirectTransition>, mut ctx: FsmContext) {
        ctx.provide_mut(&update_transition.fsm, |fsm| {
            let _ = fsm.update_transition(
                update_transition.transition.id,
                update_transition.transition.clone(),
            );
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
