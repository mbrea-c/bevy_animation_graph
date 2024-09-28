use crate::prelude::*;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct Dummy;

impl NodeLike for Dummy {
    fn display_name(&self) -> String {
        "Dummy".into()
    }
}
