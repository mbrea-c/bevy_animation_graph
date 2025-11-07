use std::path::PathBuf;

use crate::ui::generic_widgets::tree::{Tree, TreeInner, TreeRenderer};

impl<T> Tree<PathBuf, (PathBuf, T)> {
    pub fn from_paths(mut paths: Vec<(PathBuf, T)>) -> Self {
        let mut tree = Tree(Vec::new());
        paths.sort_by_key(|(p, _)| p.clone());

        for (path, val) in paths {
            insert_to_tree(&mut tree, path.clone(), val, path);
        }

        tree
    }
}

fn insert_to_subtrees<T>(
    subtrees: &mut Vec<TreeInner<PathBuf, (PathBuf, T)>>,
    path: PathBuf,
    value: T,
    remaining_path: PathBuf,
) -> Option<()> {
    if remaining_path.components().count() == 1 {
        subtrees.push(TreeInner::Leaf((path, value)));
        return Some(());
    }

    for subtree in subtrees.iter_mut() {
        let p = match subtree {
            TreeInner::Leaf(_) => continue,
            TreeInner::Node(p, _) => p.clone(),
        };
        if p.file_name().map(|s| s.to_string_lossy().to_string()) == first(remaining_path.clone()) {
            return insert_to_inner(subtree, path, value, without_first(remaining_path));
        }
    }

    let mut new_subtree = TreeInner::Node(
        up_to_including(path.clone(), first(remaining_path.clone()).unwrap()),
        vec![],
    );
    let result = insert_to_inner(&mut new_subtree, path, value, without_first(remaining_path));
    subtrees.push(new_subtree);
    result
}

fn insert_to_tree<T>(
    tree: &mut Tree<PathBuf, (PathBuf, T)>,
    path: PathBuf,
    value: T,
    remaining_path: PathBuf,
) -> Option<()> {
    insert_to_subtrees(&mut tree.0, path, value, remaining_path)
}

fn insert_to_inner<T>(
    tree: &mut TreeInner<PathBuf, (PathBuf, T)>,
    path: PathBuf,
    value: T,
    remaining_path: PathBuf,
) -> Option<()> {
    match tree {
        TreeInner::Leaf(_) => None,
        TreeInner::Node(_, subtrees) => insert_to_subtrees(subtrees, path, value, remaining_path),
    }
}

fn first(path: PathBuf) -> Option<String> {
    path.components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
}

fn without_first(path: PathBuf) -> PathBuf {
    let mut components = path.components();
    components.next();
    PathBuf::from_iter(components)
}

fn up_to_including(full_path: PathBuf, component: String) -> PathBuf {
    let mut components = Vec::new();
    for c in full_path.components() {
        components.push(c);
        if c.as_os_str().to_string_lossy() == component {
            break;
        }
    }

    PathBuf::from_iter(components.iter())
}

pub struct PathTreeRenderer<T> {
    #[allow(clippy::type_complexity)]
    pub is_selected: Box<dyn Fn(&PathBuf, &T) -> bool>,
}

#[derive(Default, Clone)]
pub struct PathTreeResult<T> {
    pub clicked: Option<T>,
}

impl<T: Clone> TreeRenderer<PathBuf, (PathBuf, T)> for PathTreeRenderer<T> {
    type Output = PathTreeResult<T>;

    fn render_inner(
        &self,
        data: &PathBuf,
        children: &[TreeInner<PathBuf, (PathBuf, T)>],
        result: &mut Self::Output,
        ui: &mut egui::Ui,
        render_child: impl Fn(
            &TreeInner<PathBuf, (PathBuf, T)>,
            &mut Self::Output,
            &mut egui::Ui,
        ) -> egui::Response,
    ) -> egui::Response {
        let label = data
            .components()
            .next_back()
            .map(|s| match s {
                std::path::Component::RootDir => "/".to_string(),
                std::path::Component::Prefix(prefix_component) => {
                    prefix_component.as_os_str().to_string_lossy().to_string()
                }
                std::path::Component::CurDir => ".".to_string(),
                std::path::Component::ParentDir => "..".to_string(),
                std::path::Component::Normal(os_str) => os_str.to_string_lossy().to_string(),
            })
            .unwrap_or("".into());

        let response = egui::CollapsingHeader::new(label)
            .default_open(true)
            .show(ui, |ui| {
                children
                    .iter()
                    .map(|c| render_child(c, result, ui))
                    .reduce(|l, r| l | r)
            });

        response.header_response.clicked();

        let mut r = response.header_response;
        if let Some(body_response) = response.body_returned.flatten() {
            r |= body_response;
        }

        r
    }

    fn render_leaf(
        &self,
        (path, value): &(PathBuf, T),
        result: &mut Self::Output,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let mut response = ui.selectable_label(
            (self.is_selected)(path, value),
            path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or("".into()),
        );
        if response.clicked() {
            response.mark_changed();
            result.clicked = Some(value.clone());
        }

        response
    }
}
