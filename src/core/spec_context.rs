use super::graph_context::{GraphContext, GraphContextTmp};

pub struct SpecContext<'a> {
    pub context: &'a mut GraphContext,
    pub context_tmp: GraphContextTmp<'a>,
}

impl<'a> SpecContext<'a> {
    pub fn new(context: &'a mut GraphContext, context_tmp: GraphContextTmp<'a>) -> Self {
        Self {
            context,
            context_tmp,
        }
    }
}
