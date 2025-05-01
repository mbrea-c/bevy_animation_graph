use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        system::{In, ResMut, SystemParam},
        world::World,
    },
    math::Vec2,
    reflect::Reflect,
};
use bevy_animation_graph::{
    core::{
        pin_map::PinMap,
        state_machine::high_level::{State, StateId, StateMachine, Transition, TransitionId},
    },
    prelude::DataValue,
};

use crate::fsm_show::{FsmIndicesMap, make_fsm_indices};

use super::{run_handler, saving::DirtyAssets};

pub enum FsmAction {
    MoveState(MoveState),
    UpdateState(UpdateState),
    UpdateTransition(UpdateTransition),
    CreateState(CreateState),
    CreateTransition(CreateTransition),
    RemoveState(RemoveState),
    RemoveTransition(RemoveTransition),
    UpdateProperties(UpdateProperties),
    GenerateIndices(GenerateIndices),
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
    pub transition_id: TransitionId,
    pub new_transition: Transition,
}

pub struct CreateState {
    pub fsm: Handle<StateMachine>,
    pub state: State,
}

pub struct CreateTransition {
    pub fsm: Handle<StateMachine>,
    pub transition: Transition,
}

pub struct RemoveState {
    pub fsm: Handle<StateMachine>,
    pub state_id: StateId,
}

pub struct RemoveTransition {
    pub fsm: Handle<StateMachine>,
    pub transition_id: TransitionId,
}

pub struct UpdateProperties {
    pub fsm: Handle<StateMachine>,
    pub new_properties: FsmProperties,
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
        FsmAction::CreateState(action) => {
            run_handler(world, FSM_ERR)(handle_create_state_system, action)
        }
        FsmAction::CreateTransition(action) => {
            run_handler(world, FSM_ERR)(handle_create_transition_system, action)
        }
        FsmAction::RemoveState(action) => {
            run_handler(world, FSM_ERR)(handle_remove_state_system, action)
        }
        FsmAction::RemoveTransition(action) => {
            run_handler(world, FSM_ERR)(handle_remove_transition_system, action)
        }
        FsmAction::UpdateProperties(action) => {
            run_handler(world, FSM_ERR)(handle_update_properties_system, action)
        }
        FsmAction::GenerateIndices(action) => {
            run_handler(world, FSM_ERR)(handle_generate_indices_system, action)
        }
    }
}

pub fn handle_move_state_system(In(action): In<MoveState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        fsm.extra.set_node_position(action.state_id, action.new_pos);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_update_state_system(In(action): In<UpdateState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.update_state(action.state_id, action.new_state);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_update_transition_system(In(action): In<UpdateTransition>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.update_transition(action.transition_id, action.new_transition);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_create_state_system(In(action): In<CreateState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        fsm.add_state(action.state);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_create_transition_system(In(action): In<CreateTransition>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.add_transition_from_ui(action.transition);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_remove_state_system(In(action): In<RemoveState>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.delete_state(action.state_id);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_remove_transition_system(In(action): In<RemoveTransition>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        let _ = fsm.delete_transition(action.transition_id);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_update_properties_system(In(action): In<UpdateProperties>, mut ctx: FsmContext) {
    ctx.provide_mut(&action.fsm, |fsm| {
        fsm.set_start_state(action.new_properties.start_state);
        fsm.set_input_data(action.new_properties.input_data);
    });

    ctx.generate_indices(&action.fsm);
}

pub fn handle_generate_indices_system(In(action): In<GenerateIndices>, mut ctx: FsmContext) {
    ctx.generate_indices(action.fsm);
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

/// Just a helper for using with the reflect based editor
#[derive(Debug, Clone, Reflect)]
pub struct FsmProperties {
    pub start_state: StateId,
    pub input_data: PinMap<DataValue>,
}

impl From<&StateMachine> for FsmProperties {
    fn from(value: &StateMachine) -> Self {
        Self {
            start_state: value.start_state.clone(),
            input_data: value.input_data.clone(),
        }
    }
}
