use crate::{core::state_machine::high_level::StateMachine, prelude::AnimationGraph};
use bevy::asset::{Assets, LoadedUntypedAsset};

#[derive(Clone, Copy)]
pub struct SpecContext<'a> {
    pub loaded_untyped_assets: &'a Assets<LoadedUntypedAsset>,
    pub graph_assets: &'a Assets<AnimationGraph>,
    pub fsm_assets: &'a Assets<StateMachine>,
}
