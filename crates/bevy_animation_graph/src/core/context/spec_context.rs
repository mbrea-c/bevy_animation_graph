use crate::{core::state_machine::high_level::StateMachine, prelude::AnimationGraph};
use bevy::asset::Assets;

#[derive(Clone, Copy)]
pub struct SpecContext<'a> {
    pub graph_assets: &'a Assets<AnimationGraph>,
    pub fsm_assets: &'a Assets<StateMachine>,
}
