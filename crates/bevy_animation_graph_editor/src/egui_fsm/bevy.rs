use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct NodesContext(pub super::lib::FsmUiContext);
