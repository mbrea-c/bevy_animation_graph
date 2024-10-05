use crate::{
    core::{animation_clip::EntityPath, animation_graph::PinMap, errors::GraphError},
    prelude::*,
};
use bevy::prelude::*;

use super::math::impl_const;

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Const>();
}

impl_const!(Const, EntityPath, DataSpec::EntityPath, "Entity Path");
