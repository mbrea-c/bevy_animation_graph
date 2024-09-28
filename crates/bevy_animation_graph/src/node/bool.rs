use bevy::prelude::*;

use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::node::math::{impl_const, impl_for_1};
use crate::prelude::{PassContext, SpecContext};

use super::*;

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Const>().register_type::<Neg>();
}

impl_const!(Const, bool, DataSpec::Bool, "bool");
impl_for_1!(Neg, bool, DataSpec::Bool, "! Neg Bool", |v: bool| !v);
