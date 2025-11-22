use bevy::asset::Assets;

use crate::{animation_graph::AnimationGraph, state_machine::high_level::StateMachine};

#[derive(Clone, Copy)]
pub struct SpecContext<'a> {
    pub graph_assets: &'a Assets<AnimationGraph>,
    pub fsm_assets: &'a Assets<StateMachine>,
}
