use crate::core::animation_graph::PinMap;
use crate::core::animation_node::NodeLike;
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct ConstBool {
    pub constant: bool,
}

impl ConstBool {
    pub const OUTPUT: &'static str = "out";

    pub fn new(constant: bool) -> Self {
        Self { constant }
    }
}

impl NodeLike for ConstBool {
    fn clone_value(&self) -> Box<dyn NodeLike> {
        Box::new(self.clone())
    }

    fn display_name(&self) -> String {
        "Bool".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, self.constant);
        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Bool)].into()
    }
}
