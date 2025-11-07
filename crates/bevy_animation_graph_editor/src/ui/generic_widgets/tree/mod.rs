pub mod paths;

pub struct TreeWidget<I, L, R> {
    pub tree: Tree<I, L>,
    pub id_hash: egui::Id,
    pub renderer: R,
}

impl<I, L, R> TreeWidget<I, L, R> {
    pub fn new_salted(tree: Tree<I, L>, renderer: R, salt: impl std::hash::Hash) -> Self {
        Self {
            tree,
            id_hash: egui::Id::new(salt),
            renderer,
        }
    }
}

impl<I, L, R> TreeWidget<I, L, R>
where
    R: TreeRenderer<I, L>,
    <R as TreeRenderer<I, L>>::Output: Default,
{
    pub fn show(self, ui: &mut egui::Ui, handle_result: impl FnOnce(R::Output)) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut output = R::Output::default();
            let response = self.tree.picker_selector(&mut output, ui, &self.renderer);
            handle_result(output);
            response
        })
        .inner
    }
}

pub struct Tree<I, L = I>(pub Vec<TreeInner<I, L>>);

pub enum TreeInner<I, L = I> {
    Leaf(L),
    Node(I, Vec<TreeInner<I, L>>),
}

pub trait TreeRenderer<I, L> {
    type Output;

    fn render_inner(
        &self,
        data: &I,
        children: &[TreeInner<I, L>],
        result: &mut Self::Output,
        ui: &mut egui::Ui,
        render_child: impl Fn(&TreeInner<I, L>, &mut Self::Output, &mut egui::Ui) -> egui::Response,
    ) -> egui::Response;

    fn render_leaf(&self, data: &L, result: &mut Self::Output, ui: &mut egui::Ui)
    -> egui::Response;
}

impl<I, L> Tree<I, L> {
    pub fn picker_selector<O, R>(
        &self,
        result: &mut O,
        ui: &mut egui::Ui,
        renderer: &R,
    ) -> egui::Response
    where
        R: TreeRenderer<I, L, Output = O>,
    {
        self.0
            .iter()
            .map(|subtree| subtree.picker_selector(result, ui, renderer))
            .reduce(|l, r| l | r)
            .unwrap_or_else(|| ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover()))
    }
}

impl<I, L> TreeInner<I, L> {
    pub fn picker_selector<O, R>(
        &self,
        result: &mut O,
        ui: &mut egui::Ui,
        renderer: &R,
    ) -> egui::Response
    where
        R: TreeRenderer<I, L, Output = O>,
    {
        match self {
            TreeInner::Leaf(data) => renderer.render_leaf(data, result, ui),
            TreeInner::Node(data, subtrees) => {
                renderer.render_inner(data, subtrees, result, ui, |subtree, result, ui| {
                    subtree.picker_selector(result, ui, renderer)
                })
            }
        }
    }
}
