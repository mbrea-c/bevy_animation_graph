use super::graph_context::{GraphContext, SystemResources};

pub struct SpecContext<'a> {
    pub context: &'a mut GraphContext,
    pub context_tmp: SystemResources<'a>,
}

impl<'a> SpecContext<'a> {
    pub fn new(context: &'a mut GraphContext, context_tmp: SystemResources<'a>) -> Self {
        Self {
            context,
            context_tmp,
        }
    }
}
