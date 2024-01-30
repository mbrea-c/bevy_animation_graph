use bevy::math::Vec2;
use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;
use std::collections::HashMap;

use super::link::*;
use super::node::*;
use super::pin::*;

pub use {
    super::node::NodeArgs,
    super::pin::{AttributeFlags, PinArgs, PinShape},
    super::style::{ColorStyle, Style, StyleFlags},
};

#[derive(Debug, Clone)]
pub enum GraphChange {
    LinkCreated(usize, usize),
    LinkRemoved(usize),
    NodeMoved(usize, Vec2),
    NodeRemoved(usize),
}

/// Keeps track of interactions that need to be stored cross-frame.
/// E.g. mouse drag
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct InteractionState {
    mouse_pos: egui::Pos2,
    mouse_delta: egui::Vec2,

    left_mouse_clicked: bool,
    left_mouse_released: bool,
    alt_mouse_clicked: bool,
    left_mouse_dragging: bool,
    alt_mouse_dragging: bool,
    mouse_in_canvas: bool,
    link_detatch_with_modifier_click: bool,

    delete_pressed: bool,
}

/// The stateful part of the node editor context that is persisted
/// between frames
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct PersistentState {
    interaction_state: InteractionState,

    selected_node_indices: Vec<usize>,
    selected_link_indices: Vec<usize>,

    node_depth_order: Vec<usize>,

    panning: egui::Vec2,

    #[derivative(Default(value = "ClickInteractionType::None"))]
    click_interaction_type: ClickInteractionType,
    click_interaction_state: ClickInteractionState,
}

/// The part of the node editor context state that is not reset every frame
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct FrameState {
    #[derivative(Default(value = "[[0.0; 2].into(); 2].into()"))]
    canvas_rect_screen_space: egui::Rect,
    node_indices_overlapping_with_mouse: Vec<usize>,
    occluded_pin_indices: Vec<usize>,
    hovered_node_index: Option<usize>,
    interactive_node_index: Option<usize>,
    hovered_link_idx: Option<usize>,
    hovered_pin_index: Option<usize>,
    hovered_pin_flags: usize,
    deleted_link_idx: Option<usize>,
    snap_link_idx: Option<usize>,

    element_state_change: ElementStateChange,
    active_pin: Option<usize>,
    graph_changes: Vec<GraphChange>,

    pins_tmp: HashMap<usize, Pin>,
    nodes_tmp: HashMap<usize, Node>,
}

/// The settings that are used by the node editor context
/// These are meant to be set by the user
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct NodesSettings {
    #[derivative(Debug = "ignore")]
    pub io: IO,
    #[derivative(Debug = "ignore")]
    pub style: Style,
}

impl FrameState {
    pub fn reset(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        self.canvas_rect_screen_space = rect;
        self.node_indices_overlapping_with_mouse.clear();
        self.occluded_pin_indices.clear();
        Option::take(&mut self.hovered_node_index);
        Option::take(&mut self.interactive_node_index);
        Option::take(&mut self.hovered_link_idx);
        Option::take(&mut self.hovered_pin_index);
        self.hovered_pin_flags = AttributeFlags::None as usize;
        Option::take(&mut self.deleted_link_idx);
        Option::take(&mut self.snap_link_idx);
        self.element_state_change.reset();
        Option::take(&mut self.active_pin);

        self.graph_changes.clear();
    }

    pub fn canvas_origin_screen_space(&self) -> egui::Vec2 {
        self.canvas_rect_screen_space.min.to_vec2()
    }
}

impl InteractionState {
    pub fn update(
        &self,
        io: &egui::InputState,
        opt_hover_pos: Option<egui::Pos2>,
        emulate_three_button_mouse: Modifier,
        link_detatch_with_modifier_click: Modifier,
        alt_mouse_button: Option<egui::PointerButton>,
    ) -> Self {
        let mut new_state = Self::default();

        if let Some(mouse_pos) = opt_hover_pos {
            new_state.mouse_in_canvas = true;
            new_state.mouse_pos = mouse_pos;
        } else {
            new_state.mouse_in_canvas = false;
            new_state.mouse_pos = self.mouse_pos;
        };

        new_state.mouse_delta = new_state.mouse_pos - self.mouse_pos;

        let left_mouse_clicked = io.pointer.button_down(egui::PointerButton::Primary);
        new_state.left_mouse_released =
            (self.left_mouse_clicked || self.left_mouse_dragging) && !left_mouse_clicked;
        new_state.left_mouse_dragging =
            (self.left_mouse_clicked || self.left_mouse_dragging) && left_mouse_clicked;
        new_state.left_mouse_clicked =
            left_mouse_clicked && !(self.left_mouse_clicked || self.left_mouse_dragging);

        let alt_mouse_clicked = emulate_three_button_mouse.is_active(&io.modifiers)
            || alt_mouse_button.map_or(false, |x| io.pointer.button_down(x));

        new_state.alt_mouse_dragging =
            (self.alt_mouse_clicked || self.alt_mouse_dragging) && alt_mouse_clicked;
        new_state.alt_mouse_clicked =
            alt_mouse_clicked && !(self.alt_mouse_clicked || new_state.alt_mouse_dragging);
        new_state.link_detatch_with_modifier_click =
            link_detatch_with_modifier_click.is_active(&io.modifiers);

        new_state.delete_pressed = io.key_pressed(egui::Key::Delete);

        new_state
    }
}

/// The Context that tracks the state of the node editor
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct NodesContext {
    state: PersistentState,
    frame_state: FrameState,
    settings: NodesSettings,

    nodes: HashMap<usize, Node>,
    pins: HashMap<usize, Pin>,
    links: HashMap<usize, Link>,
}

impl NodesContext {
    /// Displays the current state of the editor on a give Egui Ui as well as updating user input to the context
    pub fn show(
        &mut self,
        nodes: impl IntoIterator<Item = NodeSpec>,
        links: impl IntoIterator<Item = LinkSpec>,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        // Reset frame state
        self.frame_state.reset(ui);
        //self.nodes.reset();
        self.links.clear();

        ui.set_min_size(self.frame_state.canvas_rect_screen_space.size());
        let mut ui = ui.child_ui(
            self.frame_state.canvas_rect_screen_space,
            egui::Layout::top_down(egui::Align::Center),
        );
        // Setup and draw canvas, add links and nodes
        // This also draws text for attributes
        {
            let ui = &mut ui;
            let screen_rect = ui.ctx().input(|input| input.screen_rect());
            ui.set_clip_rect(
                self.frame_state
                    .canvas_rect_screen_space
                    .intersect(screen_rect),
            );
            ui.painter().rect_filled(
                self.frame_state.canvas_rect_screen_space,
                0.0,
                self.settings.style.colors[ColorStyle::GridBackground as usize],
            );

            if (self.settings.style.flags & StyleFlags::GridLines as usize) != 0 {
                self.draw_grid(self.frame_state.canvas_rect_screen_space.size(), ui);
            }

            let mut nodes = nodes
                .into_iter()
                .map(|n| (n.id, n))
                .collect::<HashMap<usize, NodeSpec>>();

            // Update node_depth_order
            let mut node_depth_order = nodes.keys().copied().collect::<Vec<_>>();
            node_depth_order.sort_by_key(|id| {
                self.state
                    .node_depth_order
                    .iter()
                    .position(|x| x == id)
                    .map(|x| x as i32)
                    .unwrap_or(-1)
            });
            self.state.node_depth_order = node_depth_order;

            for link_spec in links.into_iter() {
                self.add_link(link_spec, ui);
            }

            for node_id in self.state.node_depth_order.clone().iter() {
                let node_spec = nodes.remove(node_id).unwrap();
                self.add_node(node_spec, ui);
            }

            self.nodes = std::mem::take(&mut self.frame_state.nodes_tmp);
            self.pins = std::mem::take(&mut self.frame_state.pins_tmp);
        }

        let response = ui.interact(
            self.frame_state.canvas_rect_screen_space,
            ui.id().with("Input"),
            egui::Sense::click_and_drag(),
        );
        let hover_pos = response.hover_pos();

        ui.ctx().input(|io| {
            self.state.interaction_state = self.state.interaction_state.update(
                io,
                hover_pos,
                self.settings.io.emulate_three_button_mouse,
                self.settings.io.link_detatch_with_modifier_click,
                self.settings.io.alt_mouse_button,
            );
        });
        {
            // --- Delete all selected nodes and edges if delete pressed
            // -----------------------------------------------------------
            if self.state.interaction_state.delete_pressed {
                for node_id in self.state.selected_node_indices.drain(..) {
                    self.frame_state
                        .graph_changes
                        .push(GraphChange::NodeRemoved(node_id));
                }
                for edge_id in self.state.selected_link_indices.drain(..) {
                    self.frame_state
                        .graph_changes
                        .push(GraphChange::LinkRemoved(edge_id));
                }
            }
            // -----------------------------------------------------------

            let ui = &mut ui;
            if self.state.interaction_state.mouse_in_canvas {
                self.resolve_occluded_pins();
                self.resolve_hovered_pin();

                if self.frame_state.hovered_pin_index.is_none() {
                    self.resolve_hovered_node();
                }

                if self.frame_state.hovered_node_index.is_none() {
                    self.resolve_hovered_link();
                }
            }

            for node_idx in self.state.node_depth_order.clone() {
                self.draw_node(node_idx, ui);
            }

            let link_ids: Vec<usize> = self.links.keys().copied().collect();
            for link_id in link_ids {
                self.draw_link(link_id, ui);
            }

            if self.state.interaction_state.left_mouse_clicked
                || self.state.interaction_state.alt_mouse_clicked
            {
                self.begin_canvas_interaction();
            }

            self.click_interaction_update(ui);

            if let Some((source, target, _)) = self.link_created() {
                self.frame_state
                    .graph_changes
                    .push(GraphChange::LinkCreated(source, target));
            }
        }
        ui.painter().rect_stroke(
            self.frame_state.canvas_rect_screen_space,
            0.0,
            (
                1.0,
                self.settings.style.colors[ColorStyle::GridLine as usize],
            ),
        );
        response
    }

    #[allow(dead_code)]
    pub fn set_node_draggable(&mut self, node_id: usize, draggable: bool) {
        self.nodes.get_mut(&node_id).unwrap().state.draggable = draggable;
    }

    /// Check if there is a node that is hovered by the pointer
    #[allow(dead_code)]
    pub fn node_hovered(&self) -> Option<usize> {
        self.frame_state.hovered_node_index
    }

    /// Check if there is a link that is hovered by the pointer
    #[allow(dead_code)]
    pub fn link_hovered(&self) -> Option<usize> {
        self.frame_state.hovered_link_idx
    }

    /// Check if there is a pin that is hovered by the pointer
    #[allow(dead_code)]
    pub fn pin_hovered(&self) -> Option<usize> {
        self.frame_state.hovered_pin_index
    }

    #[allow(dead_code)]
    pub fn num_selected_nodes(&self) -> usize {
        self.state.selected_link_indices.len()
    }

    pub fn get_selected_nodes(&self) -> Vec<usize> {
        self.state.selected_node_indices.clone()
    }

    #[allow(dead_code)]
    pub fn get_selected_links(&self) -> Vec<usize> {
        self.state.selected_link_indices.clone()
    }

    #[allow(dead_code)]
    pub fn clear_node_selection(&mut self) {
        self.state.selected_node_indices.clear()
    }

    #[allow(dead_code)]
    pub fn clear_link_selection(&mut self) {
        self.state.selected_link_indices.clear()
    }

    /// Check if an attribute is currently being interacted with
    #[allow(dead_code)]
    pub fn active_attribute(&self) -> Option<usize> {
        self.frame_state.active_pin
    }

    /// Has a new link been created from a pin?
    #[allow(dead_code)]
    pub fn link_started(&self) -> Option<usize> {
        if self.frame_state.element_state_change.link_started {
            Some(
                self.state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx,
            )
        } else {
            None
        }
    }

    /// Has a link been dropped? if including_detached_links then links that were detached then dropped are included
    #[allow(dead_code)]
    pub fn link_dropped(&self, including_detached_links: bool) -> Option<usize> {
        if self.frame_state.element_state_change.link_dropped
            && (including_detached_links
                || self
                    .state
                    .click_interaction_state
                    .link_creation
                    .link_creation_type
                    != LinkCreationType::FromDetach)
        {
            Some(
                self.state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx,
            )
        } else {
            None
        }
    }

    /// Has a new link been created?
    /// -> Option<start_pin, end_pin created_from_snap>
    pub fn link_created(&self) -> Option<(usize, usize, bool)> {
        if self.frame_state.element_state_change.link_created {
            let (start_pin_id, end_pin_id) = {
                let start_pin = &self.pins[&self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx];
                let end_pin = &self.pins[&self
                    .state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .unwrap()];
                if start_pin.spec.kind == PinType::Output {
                    (start_pin.spec.id, end_pin.spec.id)
                } else {
                    (end_pin.spec.id, start_pin.spec.id)
                }
            };
            let created_from_snap =
                self.state.click_interaction_type == ClickInteractionType::LinkCreation;
            Some((start_pin_id, end_pin_id, created_from_snap))
        } else {
            None
        }
    }

    /// Has a new link been created? Includes start and end node
    /// -> Option<start_pin, start_node, end_pin, end_node created_from_snap>
    #[allow(dead_code)]
    pub fn link_created_node(&self) -> Option<(usize, usize, usize, usize, bool)> {
        if self.frame_state.element_state_change.link_created {
            let (start_pin_id, start_node_id, end_pin_id, end_node_id) = {
                let start_pin = &self.pins[&self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx];
                let end_pin = &self.pins[&self
                    .state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .unwrap()];
                let start_node_id = start_pin.state.parent_node_idx;
                let end_node_id = end_pin.state.parent_node_idx;
                if start_pin.spec.kind == PinType::Output {
                    (
                        start_pin.spec.id,
                        start_node_id,
                        end_pin.spec.id,
                        end_node_id,
                    )
                } else {
                    (
                        end_pin.spec.id,
                        end_node_id,
                        start_pin.spec.id,
                        start_node_id,
                    )
                }
            };
            let created_from_snap =
                self.state.click_interaction_type == ClickInteractionType::LinkCreation;
            Some((
                start_pin_id,
                start_node_id,
                end_pin_id,
                end_node_id,
                created_from_snap,
            ))
        } else {
            None
        }
    }

    /// If an existing link has been detached
    #[allow(dead_code)]
    pub fn link_destroyed(&self) -> Option<usize> {
        self.frame_state.deleted_link_idx
    }

    /// List of changes that occurred to the graph during frame
    pub fn get_changes(&self) -> &Vec<GraphChange> {
        &self.frame_state.graph_changes
    }

    #[allow(dead_code)]
    pub fn get_panning(&self) -> egui::Vec2 {
        self.state.panning
    }

    #[allow(dead_code)]
    pub fn reset_panniing(&mut self, panning: egui::Vec2) {
        self.state.panning = panning;
    }

    #[allow(dead_code)]
    pub fn get_node_dimensions(&self, id: usize) -> Option<egui::Vec2> {
        self.nodes.get(&id).map(|n| n.state.rect.size())
    }
}

impl NodesContext {
    fn add_node(&mut self, node_spec: NodeSpec, ui: &mut egui::Ui) {
        let node_state = if let Some(node) = self.nodes.get(&node_spec.id) {
            let mut state = node.state.clone();
            state.pin_indices.clear();
            state
        } else {
            NodeState::default()
        };
        let mut node = Node {
            spec: node_spec,
            state: node_state,
        };
        let (color_style, layout_style) = self.settings.style.format_node(node.spec.args.clone());
        node.state.color_style = color_style;
        node.state.layout_style = layout_style;

        node.state
            .background_shape
            .replace(ui.painter().add(egui::Shape::Noop));

        let node_origin = node.spec.origin;
        let node_size = node.state.size;
        let title_space = node.state.layout_style.padding.y;

        let response = ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(self.grid_space_to_screen_space(node_origin), node_size),
            |ui| {
                let mut title_info = None;
                let titlebar_shape = ui.painter().add(egui::Shape::Noop);
                let node_title = node.spec.name.clone();
                let node_subtitle = node.spec.subtitle.clone();
                let response = ui.allocate_ui(ui.available_size(), |ui| {
                    let r1 = ui.label(node_title);
                    let r2 = ui.separator();
                    let r3 = ui.label(node_subtitle);

                    r1.union(r2).union(r3)
                });
                let title_bar_content_rect = response.response.rect;
                title_info.replace((titlebar_shape, title_bar_content_rect));
                ui.add_space(title_space);
                let outline_shape = ui.painter().add(egui::Shape::Noop);
                for pin_spec in node.spec.attributes.clone().iter() {
                    self.add_pin(pin_spec.clone(), &mut node, ui);
                }
                (title_info, outline_shape)
            },
        );
        let (title_info, outline_shape) = response.inner;
        if let Some((titlebar_shape, title_bar_content_rect)) = title_info {
            node.state.titlebar_shape.replace(titlebar_shape);
            node.state.title_bar_content_rect = title_bar_content_rect;
        }
        node.state.outline_shape.replace(outline_shape);
        node.state.rect = response
            .response
            .rect
            .expand2(node.state.layout_style.padding);
        if response.response.hovered() {
            self.frame_state
                .node_indices_overlapping_with_mouse
                .push(node.spec.id);
        }

        self.frame_state.nodes_tmp.insert(node.spec.id, node);
    }

    fn add_pin(&mut self, pin_spec: PinSpec, node: &mut Node, ui: &mut egui::Ui) {
        let response = ui.allocate_ui(ui.available_size(), {
            let pin_name = pin_spec.name.clone();
            |ui| ui.label(pin_name)
        });
        let shape = ui.painter().add(egui::Shape::Noop);
        let response = response.response.union(response.inner);

        let mut pin_state = if let Some(pin) = self.pins.get(&pin_spec.id) {
            pin.state.clone()
        } else {
            PinState::default()
        };

        pin_state.shape_gui.replace(shape);
        pin_state.attribute_rect = response.rect;
        pin_state.color_style = self.settings.style.format_pin(pin_spec.style_args.clone());
        pin_state.parent_node_idx = node.spec.id;
        node.state.pin_indices.push(pin_spec.id);

        if response.is_pointer_button_down_on() {
            self.frame_state.active_pin = Some(pin_spec.id);
            self.frame_state
                .interactive_node_index
                .replace(node.spec.id);
        }

        self.frame_state.pins_tmp.insert(
            pin_spec.id,
            Pin {
                spec: pin_spec,
                state: pin_state,
            },
        );
    }

    fn add_link(&mut self, link_spec: LinkSpec, ui: &mut egui::Ui) {
        let link_state = LinkState {
            shape: Some(ui.painter().add(egui::Shape::Noop)),
            color_style: self
                .settings
                .style
                .format_link(link_spec.color_style.clone()),
        };
        self.links.insert(
            link_spec.id,
            Link {
                spec: link_spec,
                state: link_state,
            },
        );
    }

    fn draw_grid(&self, canvas_size: egui::Vec2, ui: &mut egui::Ui) {
        let mut x = self
            .state
            .panning
            .x
            .rem_euclid(self.settings.style.grid_spacing);
        while x < canvas_size.x {
            ui.painter().line_segment(
                [
                    self.editor_space_to_screen_space([x, 0.0].into()),
                    self.editor_space_to_screen_space([x, canvas_size.y].into()),
                ],
                (
                    1.0,
                    self.settings.style.colors[ColorStyle::GridLine as usize],
                ),
            );
            x += self.settings.style.grid_spacing;
        }

        let mut y = self
            .state
            .panning
            .y
            .rem_euclid(self.settings.style.grid_spacing);
        while y < canvas_size.y {
            ui.painter().line_segment(
                [
                    self.editor_space_to_screen_space([0.0, y].into()),
                    self.editor_space_to_screen_space([canvas_size.x, y].into()),
                ],
                (
                    1.0,
                    self.settings.style.colors[ColorStyle::GridLine as usize],
                ),
            );
            y += self.settings.style.grid_spacing;
        }
    }

    #[allow(dead_code)]
    fn screen_space_to_grid_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v - self.frame_state.canvas_origin_screen_space() - self.state.panning
    }

    fn grid_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.frame_state.canvas_origin_screen_space() + self.state.panning
    }

    #[allow(dead_code)]
    fn grid_space_to_editor_spcae(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.state.panning
    }

    #[allow(dead_code)]
    fn editor_space_to_grid_spcae(&self, v: egui::Pos2) -> egui::Pos2 {
        v - self.state.panning
    }

    fn editor_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.frame_state.canvas_origin_screen_space()
    }

    fn get_screen_space_pin_coordinates(&self, pin: &Pin) -> egui::Pos2 {
        let parent_node_rect = self.nodes[&pin.state.parent_node_idx].state.rect;
        self.settings.style.get_screen_space_pin_coordinates(
            &parent_node_rect,
            &pin.state.attribute_rect,
            pin.spec.kind,
        )
    }

    fn resolve_occluded_pins(&mut self) {
        let depth_stack = &self.state.node_depth_order;
        if depth_stack.len() < 2 {
            return;
        }

        for depth_idx in 0..(depth_stack.len() - 1) {
            let node_below = &self.nodes[&depth_stack[depth_idx]];
            for next_depth in &depth_stack[(depth_idx + 1)..(depth_stack.len())] {
                let rect_above = self.nodes[next_depth].state.rect;
                for idx in node_below.state.pin_indices.iter() {
                    let pin_pos = self.pins[idx].state.pos;
                    if rect_above.contains(pin_pos) {
                        self.frame_state.occluded_pin_indices.push(*idx);
                    }
                }
            }
        }
    }

    fn resolve_hovered_pin(&mut self) {
        let mut smallest_distance = f32::MAX;
        self.frame_state.hovered_pin_index.take();

        let hover_radius_sqr = self.settings.style.pin_hover_radius.powi(2);
        for idx in self.pins.keys() {
            if self.frame_state.occluded_pin_indices.contains(idx) {
                continue;
            }

            let pin_pos = self.pins[idx].state.pos;
            let distance_sqr = (pin_pos - self.state.interaction_state.mouse_pos).length_sq();
            if distance_sqr < hover_radius_sqr && distance_sqr < smallest_distance {
                smallest_distance = distance_sqr;
                self.frame_state.hovered_pin_index.replace(*idx);
            }
        }
    }

    fn resolve_hovered_node(&mut self) {
        match self.frame_state.node_indices_overlapping_with_mouse.len() {
            0 => {
                self.frame_state.hovered_node_index.take();
            }
            1 => {
                self.frame_state
                    .hovered_node_index
                    .replace(self.frame_state.node_indices_overlapping_with_mouse[0]);
            }
            _ => {
                let mut largest_depth_idx = -1;

                for node_idx in self.frame_state.node_indices_overlapping_with_mouse.iter() {
                    for (depth_idx, depth_node_idx) in
                        self.state.node_depth_order.iter().enumerate()
                    {
                        if *depth_node_idx == *node_idx && depth_idx as isize > largest_depth_idx {
                            largest_depth_idx = depth_idx as isize;
                            self.frame_state.hovered_node_index.replace(*node_idx);
                        }
                    }
                }
            }
        }
    }

    fn resolve_hovered_link(&mut self) {
        let mut smallest_distance = f32::MAX;
        self.frame_state.hovered_link_idx.take();

        for idx in self.links.keys() {
            let link = &self.links[idx];
            if self.frame_state.hovered_pin_index == Some(link.spec.start_pin_index)
                || self.frame_state.hovered_pin_index == Some(link.spec.end_pin_index)
            {
                self.frame_state.hovered_link_idx.replace(*idx);
                return;
            }

            let start_pin = &self.pins[&link.spec.start_pin_index];
            let end_pin = &self.pins[&link.spec.end_pin_index];

            let link_data = LinkBezierData::get_link_renderable(
                start_pin.state.pos,
                end_pin.state.pos,
                start_pin.spec.kind,
                self.settings.style.link_line_segments_per_length,
            );
            let link_rect = link_data
                .bezier
                .get_containing_rect_for_bezier_curve(self.settings.style.link_hover_distance);

            if link_rect.contains(self.state.interaction_state.mouse_pos) {
                let distance =
                    link_data.get_distance_to_cubic_bezier(&self.state.interaction_state.mouse_pos);
                if distance < self.settings.style.link_hover_distance
                    && distance < smallest_distance
                {
                    smallest_distance = distance;
                    self.frame_state.hovered_link_idx.replace(*idx);
                }
            }
        }
    }

    fn draw_link(&mut self, link_idx: usize, ui: &mut egui::Ui) {
        let link_hovered = self.frame_state.hovered_link_idx == Some(link_idx)
            && self.state.click_interaction_type != ClickInteractionType::BoxSelection;
        if link_hovered && self.state.interaction_state.left_mouse_clicked {
            self.begin_link_interaction(link_idx);
        }
        let link = self.links.get_mut(&link_idx).unwrap();
        let start_pin = &self.pins[&link.spec.start_pin_index];
        let end_pin = &self.pins[&link.spec.end_pin_index];
        let link_data = LinkBezierData::get_link_renderable(
            start_pin.state.pos,
            end_pin.state.pos,
            start_pin.spec.kind,
            self.settings.style.link_line_segments_per_length,
        );
        let link_shape = link.state.shape.take().unwrap();

        if self.frame_state.deleted_link_idx == Some(link_idx) {
            return;
        }

        let mut link_color = link.state.color_style.base;
        if self.state.selected_link_indices.contains(&link_idx) {
            link_color = link.state.color_style.selected;
        } else if link_hovered {
            link_color = link.state.color_style.hovered;
        }

        ui.painter().set(
            link_shape,
            link_data.draw((
                link.spec
                    .thickness
                    .unwrap_or(self.settings.style.link_thickness),
                link_color,
            )),
        );
    }

    fn draw_node(&mut self, node_idx: usize, ui: &mut egui::Ui) {
        let node = self.nodes.get_mut(&node_idx).unwrap();

        let node_hovered = self.frame_state.hovered_node_index == Some(node_idx)
            && self.state.click_interaction_type != ClickInteractionType::BoxSelection;

        let mut node_background = node.state.color_style.background;
        let mut titlebar_background = node.state.color_style.titlebar;

        if self.state.selected_node_indices.contains(&node_idx) {
            node_background = node.state.color_style.background_selected;
            titlebar_background = node.state.color_style.titlebar_selected;
        } else if node_hovered {
            node_background = node.state.color_style.background_hovered;
            titlebar_background = node.state.color_style.titlebar_hovered;
        }

        let painter = ui.painter();

        painter.set(
            node.state.background_shape.take().unwrap(),
            egui::Shape::rect_filled(
                node.state.rect,
                node.state.layout_style.corner_rounding,
                node_background,
            ),
        );
        if node.state.title_bar_content_rect.height() > 0.0 {
            painter.set(
                node.state.titlebar_shape.take().unwrap(),
                egui::Shape::rect_filled(
                    node.state.get_node_title_rect(),
                    node.state.layout_style.corner_rounding,
                    titlebar_background,
                ),
            );
        }
        if (self.settings.style.flags & StyleFlags::NodeOutline as usize) != 0 {
            painter.set(
                node.state.outline_shape.take().unwrap(),
                egui::Shape::rect_stroke(
                    node.state.rect,
                    node.state.layout_style.corner_rounding,
                    (
                        node.state.layout_style.border_thickness,
                        node.state.color_style.outline,
                    ),
                ),
            );
        }

        for pin_idx in node.state.pin_indices.clone() {
            self.draw_pin(pin_idx, ui);
        }

        if node_hovered
            && self.state.interaction_state.left_mouse_clicked
            && self.frame_state.interactive_node_index != Some(node_idx)
        {
            self.begin_node_selection(node_idx);
        }
    }

    fn draw_pin(&mut self, pin_idx: usize, ui: &mut egui::Ui) {
        let pin = self.pins.get_mut(&pin_idx).unwrap();
        let parent_node_rect = self.nodes[&pin.state.parent_node_idx].state.rect;

        pin.state.pos = self.settings.style.get_screen_space_pin_coordinates(
            &parent_node_rect,
            &pin.state.attribute_rect,
            pin.spec.kind,
        );

        let mut pin_color = pin.state.color_style.background;

        let pin_hovered = self.frame_state.hovered_pin_index == Some(pin_idx)
            && self.state.click_interaction_type != ClickInteractionType::BoxSelection;
        let pin_shape = pin.spec.shape;
        let pin_pos = pin.state.pos;
        let pin_shape_gui = pin
            .state
            .shape_gui
            .take()
            .expect("Unable to take pin shape. Perhaps your pin id is not unique?");

        if pin_hovered {
            self.frame_state.hovered_pin_flags = pin.spec.flags;
            pin_color = pin.state.color_style.hovered;

            if self.state.interaction_state.left_mouse_clicked {
                self.begin_link_creation(pin_idx);
            }
        }

        self.settings
            .style
            .draw_pin_shape(pin_pos, pin_shape, pin_color, pin_shape_gui, ui);
    }

    fn begin_canvas_interaction(&mut self) {
        let any_ui_element_hovered = self.frame_state.hovered_node_index.is_some()
            || self.frame_state.hovered_link_idx.is_some()
            || self.frame_state.hovered_pin_index.is_some();

        let mouse_not_in_canvas = !self.state.interaction_state.mouse_in_canvas;

        if self.state.click_interaction_type != ClickInteractionType::None
            || any_ui_element_hovered
            || mouse_not_in_canvas
        {
            return;
        }

        if self.state.interaction_state.alt_mouse_clicked {
            self.state.click_interaction_type = ClickInteractionType::Panning;
        } else {
            self.state.click_interaction_type = ClickInteractionType::BoxSelection;
            self.state.click_interaction_state.box_selection.min =
                self.state.interaction_state.mouse_pos;
        }
    }

    fn translate_selected_nodes(&mut self) {
        if self.state.interaction_state.left_mouse_dragging {
            let delta = self.state.interaction_state.mouse_delta;
            for idx in self.state.selected_node_indices.iter() {
                let node = self.nodes.get_mut(idx).unwrap();
                if node.state.draggable {
                    node.spec.origin += delta;
                    self.frame_state.graph_changes.push(GraphChange::NodeMoved(
                        *idx,
                        Vec2::new(node.spec.origin.x, node.spec.origin.y),
                    ));
                }
            }
        }
    }

    fn should_link_snap_to_pin(
        &self,
        start_pin: &Pin,
        hovered_pin_idx: usize,
        duplicate_link: Option<usize>,
    ) -> bool {
        let end_pin = &self.pins[&hovered_pin_idx];
        if start_pin.state.parent_node_idx == end_pin.state.parent_node_idx {
            return false;
        }

        if start_pin.spec.kind == end_pin.spec.kind {
            return false;
        }

        if duplicate_link.map_or(false, |x| Some(x) != self.frame_state.snap_link_idx) {
            return false;
        }
        true
    }

    fn box_selector_update_selection(&mut self) -> egui::Rect {
        let mut box_rect = self.state.click_interaction_state.box_selection;
        if box_rect.min.x > box_rect.max.x {
            std::mem::swap(&mut box_rect.min.x, &mut box_rect.max.x);
        }

        if box_rect.min.y > box_rect.max.y {
            std::mem::swap(&mut box_rect.min.y, &mut box_rect.max.y);
        }

        let old_selected_node_indices = self.state.selected_node_indices.clone();
        self.state.selected_node_indices.clear();
        for (idx, node) in self.nodes.iter() {
            if box_rect.intersects(node.state.rect) {
                self.state.selected_node_indices.push(*idx);
            }
        }
        // Force stability
        self.state.selected_node_indices.sort_by_key(|idx| {
            old_selected_node_indices
                .iter()
                .position(|x| x == idx)
                .map(|u| u as i32)
                .unwrap_or(i32::MAX)
        });

        self.state.selected_link_indices.clear();
        for (idx, link) in self.links.iter() {
            let pin_start = &self.pins[&link.spec.start_pin_index];
            let pin_end = &self.pins[&link.spec.end_pin_index];
            let node_start_rect = self.nodes[&pin_start.state.parent_node_idx].state.rect;
            let node_end_rect = self.nodes[&pin_end.state.parent_node_idx].state.rect;
            let start = self.settings.style.get_screen_space_pin_coordinates(
                &node_start_rect,
                &pin_start.state.attribute_rect,
                pin_start.spec.kind,
            );
            let end = self.settings.style.get_screen_space_pin_coordinates(
                &node_end_rect,
                &pin_end.state.attribute_rect,
                pin_end.spec.kind,
            );

            if self.rectangle_overlaps_link(&box_rect, &start, &end, pin_start.spec.kind) {
                self.state.selected_link_indices.push(*idx);
            }
        }
        box_rect
    }

    #[inline]
    fn rectangle_overlaps_link(
        &self,
        rect: &egui::Rect,
        start: &egui::Pos2,
        end: &egui::Pos2,
        start_type: PinType,
    ) -> bool {
        let mut lrect = egui::Rect::from_min_max(*start, *end);
        if lrect.min.x > lrect.max.x {
            std::mem::swap(&mut lrect.min.x, &mut lrect.max.x);
        }

        if lrect.min.y > lrect.max.y {
            std::mem::swap(&mut lrect.min.y, &mut lrect.max.y);
        }

        if rect.intersects(lrect) {
            if rect.contains(*start) || rect.contains(*end) {
                return true;
            }

            let link_data = LinkBezierData::get_link_renderable(
                *start,
                *end,
                start_type,
                self.settings.style.link_line_segments_per_length,
            );
            return link_data.rectangle_overlaps_bezier(rect);
        }
        false
    }

    fn click_interaction_update(&mut self, ui: &mut egui::Ui) {
        match self.state.click_interaction_type {
            ClickInteractionType::BoxSelection => {
                self.state.click_interaction_state.box_selection.max =
                    self.state.interaction_state.mouse_pos;
                let rect = self.box_selector_update_selection();

                let box_selector_color =
                    self.settings.style.colors[ColorStyle::BoxSelector as usize];
                let box_selector_outline =
                    self.settings.style.colors[ColorStyle::BoxSelectorOutline as usize];
                ui.painter()
                    .rect(rect, 0.0, box_selector_color, (1.0, box_selector_outline));

                if self.state.interaction_state.left_mouse_released {
                    let selected_nodes = &self.state.selected_node_indices;
                    for id in selected_nodes {
                        if let Some(depth_idx) =
                            self.state.node_depth_order.iter().position(|x| x == id)
                        {
                            let id = self.state.node_depth_order.remove(depth_idx);
                            self.state.node_depth_order.push(id);
                        }
                    }
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Node => {
                self.translate_selected_nodes();
                if self.state.interaction_state.left_mouse_released {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Link => {
                if self.state.interaction_state.left_mouse_released {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::LinkCreation => {
                let maybe_duplicate_link_idx = self.frame_state.hovered_pin_index.and_then(|idx| {
                    self.find_duplicate_link(
                        self.state
                            .click_interaction_state
                            .link_creation
                            .start_pin_idx,
                        idx,
                    )
                });

                let should_snap = self.frame_state.hovered_pin_index.map_or(false, |idx| {
                    let start_pin = &self.pins[&self
                        .state
                        .click_interaction_state
                        .link_creation
                        .start_pin_idx];
                    self.should_link_snap_to_pin(start_pin, idx, maybe_duplicate_link_idx)
                });

                let snapping_pin_changed = self
                    .state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .map_or(false, |idx| self.frame_state.hovered_pin_index != Some(idx));

                if snapping_pin_changed && self.frame_state.snap_link_idx.is_some() {
                    self.begin_link_detach(
                        self.frame_state.snap_link_idx.unwrap(),
                        self.state
                            .click_interaction_state
                            .link_creation
                            .end_pin_index
                            .unwrap(),
                    );
                }

                let start_pin = &self.pins[&self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx];
                let start_pos = self.get_screen_space_pin_coordinates(start_pin);

                let end_pos = if should_snap {
                    self.get_screen_space_pin_coordinates(
                        &self.pins[&self.frame_state.hovered_pin_index.unwrap()],
                    )
                } else {
                    self.state.interaction_state.mouse_pos
                };

                let link_data = LinkBezierData::get_link_renderable(
                    start_pos,
                    end_pos,
                    start_pin.spec.kind,
                    self.settings.style.link_line_segments_per_length,
                );
                ui.painter().add(link_data.draw((
                    self.settings.style.link_thickness,
                    self.settings.style.colors[ColorStyle::Link as usize],
                )));

                let link_creation_on_snap =
                    self.frame_state.hovered_pin_index.map_or(false, |idx| {
                        (self.pins[&idx].spec.flags
                            & AttributeFlags::EnableLinkCreationOnSnap as usize)
                            != 0
                    });

                if !should_snap {
                    self.state
                        .click_interaction_state
                        .link_creation
                        .end_pin_index
                        .take();
                }

                let create_link = should_snap
                    && (self.state.interaction_state.left_mouse_released || link_creation_on_snap);

                if create_link && maybe_duplicate_link_idx.is_none() {
                    if !self.state.interaction_state.left_mouse_released
                        && self
                            .state
                            .click_interaction_state
                            .link_creation
                            .end_pin_index
                            == self.frame_state.hovered_pin_index
                    {
                        return;
                    }
                    self.frame_state.element_state_change.link_created = true;
                    self.state
                        .click_interaction_state
                        .link_creation
                        .end_pin_index = self.frame_state.hovered_pin_index;
                }

                if self.state.interaction_state.left_mouse_released {
                    self.state.click_interaction_type = ClickInteractionType::None;
                    if !create_link {
                        self.frame_state.element_state_change.link_dropped = true;
                    }
                }
            }
            ClickInteractionType::Panning => {
                if self.state.interaction_state.alt_mouse_dragging
                    || self.state.interaction_state.alt_mouse_clicked
                {
                    self.state.panning += self.state.interaction_state.mouse_delta;
                } else {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::None => (),
        }
    }

    fn begin_link_detach(&mut self, idx: usize, detach_idx: usize) {
        self.state
            .click_interaction_state
            .link_creation
            .end_pin_index
            .take();
        let link = &self.links[&idx];
        self.state
            .click_interaction_state
            .link_creation
            .start_pin_idx = if detach_idx == link.spec.start_pin_index {
            link.spec.end_pin_index
        } else {
            link.spec.start_pin_index
        };
        self.frame_state.deleted_link_idx.replace(idx);
        self.frame_state
            .graph_changes
            .push(GraphChange::LinkRemoved(idx));
    }

    fn begin_link_interaction(&mut self, idx: usize) {
        if self.state.click_interaction_type == ClickInteractionType::LinkCreation {
            if (self.frame_state.hovered_pin_flags
                & AttributeFlags::EnableLinkDetachWithDragClick as usize)
                != 0
            {
                self.begin_link_detach(idx, self.frame_state.hovered_pin_index.unwrap());
                self.state
                    .click_interaction_state
                    .link_creation
                    .link_creation_type = LinkCreationType::FromDetach;
            }
        } else if self
            .state
            .interaction_state
            .link_detatch_with_modifier_click
        {
            let link = &self.links[&idx];
            let start_pin = &self.pins[&link.spec.start_pin_index];
            let end_pin = &self.pins[&link.spec.end_pin_index];
            let dist_to_start = start_pin
                .state
                .pos
                .distance(self.state.interaction_state.mouse_pos);
            let dist_to_end = end_pin
                .state
                .pos
                .distance(self.state.interaction_state.mouse_pos);
            let closest_pin_idx = if dist_to_start < dist_to_end {
                link.spec.start_pin_index
            } else {
                link.spec.end_pin_index
            };
            self.state.click_interaction_type = ClickInteractionType::LinkCreation;
            self.begin_link_detach(idx, closest_pin_idx);
        } else {
            self.begin_link_selection(idx);
        }
    }

    fn begin_link_creation(&mut self, hovered_pin_idx: usize) {
        self.state.click_interaction_type = ClickInteractionType::LinkCreation;
        self.state
            .click_interaction_state
            .link_creation
            .start_pin_idx = hovered_pin_idx;
        self.state
            .click_interaction_state
            .link_creation
            .end_pin_index
            .take();
        self.state
            .click_interaction_state
            .link_creation
            .link_creation_type = LinkCreationType::Standard;
        self.frame_state.element_state_change.link_started = true;
    }

    fn begin_link_selection(&mut self, idx: usize) {
        self.state.click_interaction_type = ClickInteractionType::Link;
        self.state.selected_node_indices.clear();
        self.state.selected_link_indices.clear();
        self.state.selected_link_indices.push(idx);
    }

    fn find_duplicate_link(&self, start_pin_idx: usize, end_pin_idx: usize) -> Option<usize> {
        let mut test_link = Link::default();
        test_link.spec.start_pin_index = start_pin_idx;
        test_link.spec.end_pin_index = end_pin_idx;
        for (idx, link) in self.links.iter() {
            if *link == test_link {
                return Some(*idx);
            }
        }
        None
    }

    fn begin_node_selection(&mut self, idx: usize) {
        if self.state.click_interaction_type != ClickInteractionType::None {
            return;
        }
        self.state.click_interaction_type = ClickInteractionType::Node;
        if !self.state.selected_node_indices.contains(&idx) {
            self.state.selected_node_indices.clear();
            self.state.selected_link_indices.clear();
            self.state.selected_node_indices.push(idx);

            if let Some(depth_idx) = self.state.node_depth_order.iter().position(|x| *x == idx) {
                let id = self.state.node_depth_order.remove(depth_idx);
                self.state.node_depth_order.push(id);
            }
            // self.state.node_depth_order.retain(|x| *x != idx);
            // self.state.node_depth_order.push(idx);
        }
    }
}

#[derive(Derivative)]
#[derivative(Default, Debug)]
struct ElementStateChange {
    link_started: bool,
    link_dropped: bool,
    link_created: bool,
}

impl ElementStateChange {
    pub fn reset(&mut self) {
        self.link_started = false;
        self.link_dropped = false;
        self.link_created = false;
    }
}

#[derive(PartialEq, Debug)]
enum ClickInteractionType {
    Node,
    Link,
    LinkCreation,
    Panning,
    BoxSelection,
    None,
}

#[derive(PartialEq, Debug)]
enum LinkCreationType {
    Standard,
    FromDetach,
}

#[derive(Derivative, Debug)]
#[derivative(Default)]
struct ClickInteractionStateLinkCreation {
    start_pin_idx: usize,
    end_pin_index: Option<usize>,
    #[derivative(Default(value = "LinkCreationType::Standard"))]
    link_creation_type: LinkCreationType,
}

#[derive(Derivative, Debug)]
#[derivative(Default)]
struct ClickInteractionState {
    link_creation: ClickInteractionStateLinkCreation,
    #[derivative(Default(value = "[[0.0; 2].into(); 2].into()"))]
    box_selection: egui::Rect,
}

/// This controls the modifers needed for certain mouse interactions
#[derive(Derivative, Debug)]
#[derivative(Default)]
pub struct IO {
    /// The Modfier that needs to pressed to pan the editor
    #[derivative(Default(value = "Modifier::None"))]
    pub emulate_three_button_mouse: Modifier,

    // The Modifier that needs to be pressed to detatch a link instead of creating a new one
    #[derivative(Default(value = "Modifier::None"))]
    pub link_detatch_with_modifier_click: Modifier,

    // The mouse button that pans the editor. Should probably not be set to Primary.
    #[derivative(Default(value = "Some(egui::PointerButton::Middle)"))]
    pub alt_mouse_button: Option<egui::PointerButton>,
}

/// Used to track which Egui Modifier needs to be pressed for certain IO actions
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Modifier {
    Alt,
    Crtl,
    Shift,
    Command,
    None,
}

impl Modifier {
    fn is_active(&self, mods: &egui::Modifiers) -> bool {
        match self {
            Modifier::Alt => mods.alt,
            Modifier::Crtl => mods.ctrl,
            Modifier::Shift => mods.shift,
            Modifier::Command => mods.command,
            Modifier::None => false,
        }
    }
}

pub trait Id {
    fn id(&self) -> usize;
    fn new(id: usize) -> Self;
}
