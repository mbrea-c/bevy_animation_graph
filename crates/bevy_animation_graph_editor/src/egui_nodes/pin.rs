use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;

#[derive(Default, Debug, Clone)]
/// The Visual Style of a Link.
/// If feilds are None then the Context style is used.
/// shape defualts to CircleFilled
pub struct PinStyleArgs {
    pub background: Option<egui::Color32>,
    pub hovered: Option<egui::Color32>,
    pub shape: Option<PinShape>,
}

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub(crate) enum PinType {
    #[default]
    None,
    Input,
    Output,
}

/// Controls the shape of an attribut pin.
/// Triangle and TriangleFilled are not currently implemented and will not be drawn
#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)]
pub enum PinShape {
    Circle,
    #[default]
    CircleFilled,
    Triangle,
    TriangleFilled,
    Quad,
    QuadFilled,
}

/// Controls the way that attribute pins behave
#[derive(Debug)]
pub enum AttributeFlags {
    None = 0,

    /// If there is a link on the node then it will detatch instead of creating a new one.
    /// Requires handling of deleted links via Context::link_destroyed
    EnableLinkDetachWithDragClick = 1 << 0,

    /// Visual snapping will trigger link creation / destruction
    EnableLinkCreationOnSnap = 1 << 1,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct PinStyle {
    pub background: egui::Color32,
    pub hovered: egui::Color32,
    pub shape: PinShape,
}

#[derive(Derivative, Clone)]
#[derivative(Debug, Default)]
pub struct PinSpec {
    pub id: usize,
    pub kind: PinType,
    pub name: String,
    pub style_args: PinStyleArgs,
    pub flags: usize,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub(crate) struct PinState {
    pub parent_node_idx: usize,
    pub attribute_rect: egui::Rect,
    pub pos: egui::Pos2,
    #[derivative(Debug = "ignore")]
    pub color_style: PinStyle,
    #[derivative(Debug = "ignore")]
    pub shape_gui: Option<egui::layers::ShapeIdx>,
}

impl Default for PinState {
    fn default() -> Self {
        Self {
            parent_node_idx: Default::default(),
            attribute_rect: egui::Rect::ZERO,
            pos: Default::default(),
            color_style: Default::default(),
            shape_gui: Default::default(),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Pin {
    pub spec: PinSpec,
    pub state: PinState,
}
