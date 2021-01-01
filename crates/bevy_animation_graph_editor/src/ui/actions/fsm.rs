use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        system::{In, ResMut, SystemParam},
        world::World,
    },
    math::Vec2,
    reflect::Reflect,
};
use bevy_animation_graph::core::{
    context::spec_context::NodeSpec,
    state_machine::high_level::{
        DirectTransition, DirectTransitionId, State, StateId, StateMachine,
    },
};

use super::{run_handler, saving::DirtyAssets};

pub enum FsmAction {
    MoveState(MoveState),
    UpdateState(UpdateState),
    UpdateTransition(UpdateTransition),
    RemoveState(RemoveState),
    RemoveTransition(RemoveTransition),
}

pub struct MoveState {
    pub fsm: Handle<StateMachine>,
    pub state_id: StateId,
    pub new_pos: Vec2,
}

/// Some of the properties of a state are changed.
/// This includes the state name, so this doubles as a rename action
pub struct UpdateState {
    pub fsm: Handle<StateMachine>,
    pub state_id: StateId,
    pub new_state: State,
}

pub struct UpdateTransition {
    pub fsm: Handle<StateMachine>,
    pub transition_id: DirectTransitionId,
    pub new_transition: DirectTransition,
}

pub struct RemoveState {
    pub fsm: Handle<StateMachine>,
    pub state_id: StateId,
}

pub struct RemoveTransition {
    pub fsm: Handle<StateMachine>,
    pub transition_id: DirectTransitionId,
}

pub struct GenerateIndices {
    pub fsm: AssetId<StateMachine>,
}

const FSM_ERR: &str = "Failed to run FSM action";

pub fn handle_fsm_action(world: &mut World, action: FsmAction) {
    match action {
        FsmAction::MoveState(action) => {
            run_handler(world, FSM_ERR)(handle_move_state_system, action)
        }
        FsmAction::UpdateState(action) => {
            run_handler(world, FSM_ERR)(handle_update_state_system, action)
        }
        FsmAction::UpdateTransition(action) => {
            run_handler(world, FSM_ERR)(handle_update_transition_system, action)
        }
        FsmAction::RemoveState(action) => {
            run_handler(world, FSM_ERR)(handle_remove_state_system, action)
        }
        FsmAction::RemoveTransition(action) => {
            run_handler(world, FSM_ERR)(handle_remove_transition_system, action)
        }
    }
}

pub fn handle_move_state_system(In(action): In<MoveState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        fsm.extra
            .set_state_position(action.state_id, action.new_pos);
    });
}

pub fn handle_update_state_system(In(action): In<UpdateState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.update_state(action.state_id, action.new_state);
    });
}

pub fn handle_update_transition_system(In(action): In<UpdateTransition>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.update_transition(action.transition_id, action.new_transition);
    });
}

pub fn handle_remove_state_system(In(action): In<RemoveState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.delete_state(action.state_id);
    });
}

pub fn handle_remove_transition_system(In(action): In<RemoveTransition>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.delete_transition(action.transition_id);
    });
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

/// Just a helper for using with the reflect based editor
#[derive(Debug, Clone, Reflect)]
pub struct FsmProperties {
    pub start_state: StateId,
    pub node_spec: NodeSpec,
}

impl From<&StateMachine> for FsmProperties {
    fn from(value: &StateMachine) -> Self {
        Self {
            start_state: value.start_state.clone(),
            node_spec: value.node_spec.clone(),
        }
    }
}
