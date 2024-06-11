use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;

use super::{
    lib::*,
    link::{LinkStyle, LinkStyleArgs},
    node::{StateDataColorStyle, StateDataLayoutStyle},
};

/// Represents different color style values used by a Context
#[derive(Debug, Clone, Copy)]
pub enum ColorStyle {
    NodeBackground = 0,
    NodeBackgroundHovered,
    NodeBackgroundSelected,
    NodeOutline,
    ActiveNodeOutline,
    TitleBar,
    TitleBarHovered,
    TitleBarSelected,
    StartTitleBar,
    StartTitleBarHovered,
    StartTitleBarSelected,
    Link,
    LinkHovered,
    LinkSelected,
    Pin,
    PinHovered,
    BoxSelector,
    BoxSelectorOutline,
    GridBackground,
    GridLine,
    Count,
}

/// Controls some style aspects
#[derive(Debug)]
#[allow(dead_code)]
pub enum StyleFlags {
    None = 0,
    NodeOutline = 1 << 0,
    GridLines = 1 << 2,
}

impl ColorStyle {
    /// dark color style
    pub fn colors_dark() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];
        colors[ColorStyle::NodeBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(50, 50, 50, 255);
        colors[ColorStyle::NodeBackgroundHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeBackgroundSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 255);
        colors[ColorStyle::ActiveNodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 200, 255);
        colors[ColorStyle::TitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(41, 74, 122, 255);
        colors[ColorStyle::TitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::TitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::StartTitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(122, 74, 41, 255);
        colors[ColorStyle::StartTitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(250, 150, 66, 255);
        colors[ColorStyle::StartTitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(250, 150, 66, 255);
        colors[ColorStyle::Link as usize] =
            egui::Color32::from_rgba_unmultiplied(61, 133, 224, 200);
        colors[ColorStyle::LinkHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::LinkSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::Pin as usize] = egui::Color32::from_rgba_unmultiplied(53, 150, 250, 180);
        colors[ColorStyle::PinHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(53, 150, 250, 255);
        colors[ColorStyle::BoxSelector as usize] =
            egui::Color32::from_rgba_unmultiplied(61, 133, 224, 30);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(61, 133, 224, 150);
        colors[ColorStyle::GridBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(40, 40, 50, 200);
        colors[ColorStyle::GridLine as usize] =
            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 40);
        colors
    }

    /// classic color style
    #[allow(dead_code)]
    pub fn colors_classic() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];
        colors[ColorStyle::NodeBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(50, 50, 50, 255);
        colors[ColorStyle::NodeBackgroundHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeBackgroundSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 255);
        colors[ColorStyle::TitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(69, 69, 138, 255);
        colors[ColorStyle::TitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 255);
        colors[ColorStyle::TitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 255);
        colors[ColorStyle::Link as usize] =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100);
        colors[ColorStyle::LinkHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(105, 99, 204, 153);
        colors[ColorStyle::LinkSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(105, 99, 204, 153);
        colors[ColorStyle::Pin as usize] = egui::Color32::from_rgba_unmultiplied(89, 102, 156, 170);
        colors[ColorStyle::PinHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(102, 122, 179, 200);
        colors[ColorStyle::BoxSelector as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 100);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 255);
        colors[ColorStyle::GridBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(40, 40, 50, 200);
        colors[ColorStyle::GridLine as usize] =
            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 40);
        colors
    }

    /// light color style
    #[allow(dead_code)]
    pub fn colors_light() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];
        colors[ColorStyle::NodeBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(240, 240, 240, 255);
        colors[ColorStyle::NodeBackgroundHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(240, 240, 240, 255);
        colors[ColorStyle::NodeBackgroundSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(240, 240, 240, 255);
        colors[ColorStyle::NodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 255);
        colors[ColorStyle::TitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(248, 248, 248, 255);
        colors[ColorStyle::TitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(209, 209, 209, 255);
        colors[ColorStyle::TitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(209, 209, 209, 255);
        colors[ColorStyle::Link as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 100);
        colors[ColorStyle::LinkHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 242);
        colors[ColorStyle::LinkSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 242);
        colors[ColorStyle::Pin as usize] = egui::Color32::from_rgba_unmultiplied(66, 150, 250, 160);
        colors[ColorStyle::PinHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::BoxSelector as usize] =
            egui::Color32::from_rgba_unmultiplied(90, 170, 250, 30);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(90, 170, 250, 150);
        colors[ColorStyle::GridBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(225, 225, 225, 255);
        colors[ColorStyle::GridLine as usize] =
            egui::Color32::from_rgba_unmultiplied(180, 180, 180, 100);
        colors
    }
}

/// The style used by a context
/// Example:
/// ``` rust
/// # use egui_nodes::{Context, Style, ColorStyle};
/// let mut ctx = Context::default();
/// let style = Style { colors: ColorStyle::colors_classic(), ..Default::default() };
/// ctx.style = style;
/// ```
#[derive(Debug)]
pub struct Style {
    pub grid_spacing: f32,
    pub node_corner_rounding: f32,
    pub node_padding_horizontal: f32,
    pub node_padding_vertical: f32,
    pub node_border_thickness: f32,

    pub link_thickness: f32,
    pub link_line_segments_per_length: f32,
    pub link_hover_distance: f32,

    pub pin_circle_radius: f32,
    pub pin_quad_side_length: f32,
    pub pin_triangle_side_length: f32,
    pub pin_line_thickness: f32,
    pub pin_hover_radius: f32,
    pub pin_offset: f32,

    pub flags: usize,
    pub colors: [egui::Color32; ColorStyle::Count as usize],
}

impl Default for Style {
    fn default() -> Self {
        Self {
            grid_spacing: 32.0,
            node_corner_rounding: 4.0,
            node_padding_horizontal: 8.0,
            node_padding_vertical: 8.0,
            node_border_thickness: 1.0,
            link_thickness: 3.0,
            link_line_segments_per_length: 0.1,
            link_hover_distance: 10.0,
            pin_circle_radius: 4.0,
            pin_quad_side_length: 7.0,
            pin_triangle_side_length: 9.5,
            pin_line_thickness: 1.0,
            pin_hover_radius: 10.0,
            pin_offset: 0.0,
            flags: StyleFlags::NodeOutline as usize | StyleFlags::GridLines as usize,
            colors: ColorStyle::colors_dark(),
        }
    }
}

impl Style {
    pub(crate) fn format_node(
        &self,
        args: StateArgs,
    ) -> (StateDataColorStyle, StateDataLayoutStyle) {
        let mut color = StateDataColorStyle::default();
        let mut layout = StateDataLayoutStyle::default();

        color.background = args
            .background
            .unwrap_or(self.colors[ColorStyle::NodeBackground as usize]);
        color.background_hovered = args
            .background_hovered
            .unwrap_or(self.colors[ColorStyle::NodeBackgroundHovered as usize]);
        color.background_selected = args
            .background_selected
            .unwrap_or(self.colors[ColorStyle::NodeBackgroundSelected as usize]);
        color.outline = args
            .outline
            .unwrap_or(self.colors[ColorStyle::NodeOutline as usize]);
        color.titlebar = args
            .titlebar
            .unwrap_or(self.colors[ColorStyle::TitleBar as usize]);
        color.titlebar_hovered = args
            .titlebar_hovered
            .unwrap_or(self.colors[ColorStyle::TitleBarHovered as usize]);
        color.titlebar_selected = args
            .titlebar_selected
            .unwrap_or(self.colors[ColorStyle::TitleBarSelected as usize]);
        color.start_titlebar = args
            .start_titlebar
            .unwrap_or(self.colors[ColorStyle::StartTitleBar as usize]);
        color.start_titlebar_hovered = args
            .start_titlebar_hovered
            .unwrap_or(self.colors[ColorStyle::StartTitleBarHovered as usize]);
        color.start_titlebar_selected = args
            .start_titlebar_selected
            .unwrap_or(self.colors[ColorStyle::StartTitleBarSelected as usize]);
        layout.corner_rounding = args.corner_rounding.unwrap_or(self.node_corner_rounding);
        layout.padding = args.padding.unwrap_or_else(|| {
            egui::vec2(self.node_padding_horizontal, self.node_padding_vertical)
        });
        layout.border_thickness = args.border_thickness.unwrap_or(self.node_border_thickness);

        (color, layout)
    }

    pub(crate) fn format_link(&self, args: LinkStyleArgs) -> LinkStyle {
        LinkStyle {
            base: args.base.unwrap_or(self.colors[ColorStyle::Link as usize]),
            hovered: args
                .hovered
                .unwrap_or(self.colors[ColorStyle::LinkHovered as usize]),
            selected: args
                .selected
                .unwrap_or(self.colors[ColorStyle::LinkSelected as usize]),
            thickness: args.thickness.unwrap_or(self.link_thickness),
            ..Default::default()
        }
    }
}
