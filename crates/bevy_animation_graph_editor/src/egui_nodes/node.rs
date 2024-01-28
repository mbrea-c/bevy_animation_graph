use super::{
    pin::{PinSpec, PinType},
    *,
};
use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;
use pin::PinArgs;

#[derive(Default, Debug, Clone)]
/// The Style of a Node. If feilds are None then the Context style is used
pub struct NodeArgs {
    pub background: Option<egui::Color32>,
    pub background_hovered: Option<egui::Color32>,
    pub background_selected: Option<egui::Color32>,
    pub outline: Option<egui::Color32>,
    pub titlebar: Option<egui::Color32>,
    pub titlebar_hovered: Option<egui::Color32>,
    pub titlebar_selected: Option<egui::Color32>,
    pub corner_rounding: Option<f32>,
    pub padding: Option<egui::Vec2>,
    pub border_thickness: Option<f32>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct NodeDataColorStyle {
    pub background: egui::Color32,
    pub background_hovered: egui::Color32,
    pub background_selected: egui::Color32,
    pub outline: egui::Color32,
    pub titlebar: egui::Color32,
    pub titlebar_hovered: egui::Color32,
    pub titlebar_selected: egui::Color32,
}

#[derive(Default, Debug, Clone)]
pub struct NodeDataLayoutStyle {
    pub corner_rounding: f32,
    pub padding: egui::Vec2,
    pub border_thickness: f32,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct NodeData {
    pub id: usize,
    pub origin: egui::Pos2,
    pub size: egui::Vec2,
    pub title_bar_content_rect: egui::Rect,
    pub rect: egui::Rect,
    #[derivative(Debug = "ignore")]
    pub color_style: NodeDataColorStyle,
    pub layout_style: NodeDataLayoutStyle,
    pub pin_indices: Vec<usize>,
    pub draggable: bool,
    #[derivative(Debug = "ignore")]
    pub titlebar_shape: Option<egui::layers::ShapeIdx>,
    #[derivative(Debug = "ignore")]
    pub background_shape: Option<egui::layers::ShapeIdx>,
    #[derivative(Debug = "ignore")]
    pub outline_shape: Option<egui::layers::ShapeIdx>,
}

impl NodeData {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            origin: [100.0; 2].into(),
            size: [100.0; 2].into(),
            title_bar_content_rect: [[0.0; 2].into(); 2].into(),
            rect: [[0.0; 2].into(); 2].into(),
            color_style: Default::default(),
            layout_style: Default::default(),
            pin_indices: Default::default(),
            draggable: true,
            titlebar_shape: None,
            background_shape: None,
            outline_shape: None,
        }
    }

    #[inline]
    pub fn get_node_title_rect(&self) -> egui::Rect {
        let expanded_title_rect = self
            .title_bar_content_rect
            .expand2(self.layout_style.padding);
        egui::Rect::from_min_max(
            expanded_title_rect.min,
            expanded_title_rect.min + egui::vec2(self.rect.width(), expanded_title_rect.height()),
        )
    }
}

impl Default for NodeData {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct NodeSpec {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) subtitle: String,
    pub(crate) origin: egui::Pos2,
    pub(crate) attributes: Vec<PinSpec>,
    pub(crate) args: NodeArgs,
}

#[derive(Derivative, Clone)]
#[derivative(Debug, Default)]
pub(crate) struct NodeState {
    #[derivative(Default(value = "[100.;2].into()"))]
    pub size: egui::Vec2,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    pub title_bar_content_rect: egui::Rect,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    pub rect: egui::Rect,
    #[derivative(Debug = "ignore")]
    pub color_style: NodeDataColorStyle,
    pub layout_style: NodeDataLayoutStyle,
    pub pin_indices: Vec<usize>,
    #[derivative(Default(value = "true"))]
    pub draggable: bool,
    #[derivative(Debug = "ignore")]
    pub titlebar_shape: Option<egui::layers::ShapeIdx>,
    #[derivative(Debug = "ignore")]
    pub background_shape: Option<egui::layers::ShapeIdx>,
    #[derivative(Debug = "ignore")]
    pub outline_shape: Option<egui::layers::ShapeIdx>,
}
impl NodeState {
    #[inline]
    pub fn get_node_title_rect(&self) -> egui::Rect {
        let expanded_title_rect = self
            .title_bar_content_rect
            .expand2(self.layout_style.padding);
        egui::Rect::from_min_max(
            expanded_title_rect.min,
            expanded_title_rect.min + egui::vec2(self.rect.width(), expanded_title_rect.height()),
        )
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Node {
    pub spec: NodeSpec,
    pub state: NodeState,
}

/// Used to construct a node and stores the relevant ui code for its title and attributes
/// This is used so that the nodes can be rendered in the context depth order
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct NodeConstructor<'a> {
    //node: &'a mut NodeData,
    pub(crate) id: usize,
    #[derivative(Debug = "ignore")]
    pub(crate) title: Option<Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + 'a>>,
    #[derivative(Debug = "ignore")]
    pub(crate) attributes: Vec<PinSpec>,
    pub(crate) pos: Option<egui::Pos2>,
    pub(crate) args: NodeArgs,
}

impl<'a, 'b> NodeConstructor<'a> {
    /// Create a new node to be displayed in a Context.
    /// id should be the same accross frames and should not be the same as any other currently used nodes
    pub fn new(id: usize, args: NodeArgs) -> Self {
        Self {
            id,
            args,
            ..Default::default()
        }
    }

    /// Add a title to a node
    pub fn with_title(mut self, title: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.title.replace(Box::new(title));
        self
    }

    /// Add an input attibute to a node, this attribute can be connected to output attributes of other nodes
    /// id should be the same accross frames and should not be the same as any other currently used attributes
    /// the attribute should return a egui::Response to be checked for interaction
    pub fn with_attribute(mut self, pin_spec: PinSpec) -> Self {
        self.attributes.push(pin_spec);
        self
    }

    /// Set the position of the node in screen space when it is first created.
    /// To modify it after creation use one of the set_node_pos methods of the Context
    pub fn with_origin(mut self, origin: egui::Pos2) -> Self {
        self.pos.replace(origin);
        self
    }

    /// Get the id of this NodeConstructor
    pub fn id(&self) -> usize {
        self.id
    }
}
