use crate::prelude::AnimationGraph;
use bevy::asset::Assets;

#[derive(Clone, Copy)]
pub struct SpecContext<'a> {
    pub graph_assets: &'a Assets<AnimationGraph>,
}

impl<'a> SpecContext<'a> {
    pub fn new(graph_assets: &'a Assets<AnimationGraph>) -> Self {
        Self { graph_assets }
    }
}
