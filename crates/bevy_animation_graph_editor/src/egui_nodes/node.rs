use super::pin::PinSpec;
use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;

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
