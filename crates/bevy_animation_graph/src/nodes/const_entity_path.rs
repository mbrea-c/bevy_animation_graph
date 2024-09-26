use crate::core::animation_clip::EntityPath;
use crate::core::animation_graph::PinMap;
use crate::core::animation_node::NodeLike;
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{DataValue, PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct ConstEntityPath {
    pub path: EntityPath,
}

impl ConstEntityPath {
    pub const OUTPUT: &'static str = "out";

    pub fn new(path: EntityPath) -> Self {
        Self { path }
    }
}

impl NodeLike for ConstEntityPath {
    fn clone_value(&self) -> Box<dyn NodeLike> {
        Box::new(self.clone())
    }

    fn display_name(&self) -> String {
        "Entity Path".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, DataValue::EntityPath(self.path.clone()));
        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::EntityPath)].into()
    }
}
