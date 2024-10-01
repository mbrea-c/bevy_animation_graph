use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;

#[derive(Default, Debug, Clone)]
/// The Style of a Node. If feilds are None then the Context style is used
pub struct StateArgs {
    pub background: Option<egui::Color32>,
    pub background_hovered: Option<egui::Color32>,
    pub background_selected: Option<egui::Color32>,
    pub outline: Option<egui::Color32>,
    pub titlebar: Option<egui::Color32>,
    pub titlebar_hovered: Option<egui::Color32>,
    pub titlebar_selected: Option<egui::Color32>,
    pub start_titlebar: Option<egui::Color32>,
    pub start_titlebar_hovered: Option<egui::Color32>,
    pub start_titlebar_selected: Option<egui::Color32>,
    pub corner_rounding: Option<f32>,
    pub padding: Option<egui::Vec2>,
    pub border_thickness: Option<f32>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct StateDataColorStyle {
    pub background: egui::Color32,
    pub background_hovered: egui::Color32,
    pub background_selected: egui::Color32,
    pub outline: egui::Color32,
    pub titlebar: egui::Color32,
    pub titlebar_hovered: egui::Color32,
    pub titlebar_selected: egui::Color32,
    pub start_titlebar: egui::Color32,
    pub start_titlebar_hovered: egui::Color32,
    pub start_titlebar_selected: egui::Color32,
}

#[derive(Default, Debug, Clone)]
pub struct StateDataLayoutStyle {
    pub corner_rounding: f32,
    pub padding: egui::Vec2,
    pub border_thickness: f32,
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct StateSpec {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) subtitle: String,
    pub(crate) origin: egui::Pos2,
    pub(crate) args: StateArgs,
    pub(crate) time: Option<f32>,
    pub(crate) duration: Option<f32>,
    pub(crate) active: bool,
    pub(crate) is_start_state: bool,
    pub(crate) has_global_transition: bool,
}

#[derive(Derivative, Clone)]
#[derivative(Debug, Default)]
pub(crate) struct StateState {
    #[derivative(Default(value = "egui::vec2(100., 100.)"))]
    pub size: egui::Vec2,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    pub title_bar_content_rect: egui::Rect,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    pub rect: egui::Rect,
    #[derivative(Debug = "ignore")]
    pub color_style: StateDataColorStyle,
    pub layout_style: StateDataLayoutStyle,
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

impl StateState {
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
    pub spec: StateSpec,
    pub state: StateState,
}

impl Node {
    pub fn center(&self) -> egui::Pos2 {
        self.spec.origin + 0.5 * self.state.size
    }
}
