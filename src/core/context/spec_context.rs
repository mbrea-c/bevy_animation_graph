use super::{GraphContext, SystemResources};

pub struct SpecContext<'a> {
    pub context: &'a mut GraphContext,
    pub context_tmp: &'a SystemResources<'a, 'a>,
}

impl<'a> SpecContext<'a> {
    pub fn new(context: &'a mut GraphContext, context_tmp: &'a SystemResources) -> Self {
        Self {
            context,
            context_tmp,
        }
    }
}
