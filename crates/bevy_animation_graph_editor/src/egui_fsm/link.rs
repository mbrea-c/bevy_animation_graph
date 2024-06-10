use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;
use egui::epaint::PathShape;
use egui_dock::egui::layers::ShapeIdx;

#[derive(Debug, Default, Clone)]
pub struct LinkStyleArgs {
    pub base: Option<egui::Color32>,
    pub hovered: Option<egui::Color32>,
    pub selected: Option<egui::Color32>,
    pub thickness: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct LinkStyle {
    pub base: egui::Color32,
    pub hovered: egui::Color32,
    pub selected: egui::Color32,
    pub active_base: egui::Color32,
    pub active_hovered: egui::Color32,
    pub active_selected: egui::Color32,
    pub thickness: f32,
}

impl Default for LinkStyle {
    fn default() -> Self {
        Self {
            base: egui::Color32::from_rgba_unmultiplied(150, 150, 150, 127),
            hovered: egui::Color32::from_rgb(200, 200, 200),
            selected: egui::Color32::from_rgb(200, 200, 200),
            active_base: egui::Color32::from_rgb(100, 100, 200),
            active_hovered: egui::Color32::from_rgb(140, 140, 225),
            active_selected: egui::Color32::from_rgb(140, 140, 225),
            thickness: 3.,
        }
    }
}

#[derive(Derivative, Default, Clone)]
#[derivative(Debug)]
pub struct TransitionSpec {
    pub id: usize,
    pub start_pin_index: usize,
    pub end_pin_index: usize,
    #[derivative(Debug = "ignore")]
    pub style: LinkStyleArgs,
    pub active: bool,
}

#[derive(Derivative, Default, Clone)]
#[derivative(Debug)]
pub struct TransitionState {
    #[derivative(Debug = "ignore")]
    pub style: LinkStyle,
    #[derivative(Debug = "ignore")]
    pub line_shape: Option<egui::layers::ShapeIdx>,
    pub arrow_shape: Option<egui::layers::ShapeIdx>,
    pub links_for_node_pair: u32,
    pub index_in_node_pair: u32,
    pub offset_inverted: bool,
}

#[derive(Derivative, Default, Clone)]
#[derivative(Debug)]
pub struct Transition {
    pub spec: TransitionSpec,
    pub state: TransitionState,
}

impl Transition {
    pub fn perpendicular_offset(&self) -> f32 {
        if self.state.links_for_node_pair == 1 {
            return 0.;
        } else {
            let total_offset = 60.;
            (self.state.index_in_node_pair as f32 * total_offset
                / (self.state.links_for_node_pair as f32 - 1.)
                - 0.5 * total_offset)
                * (if self.state.offset_inverted { -1. } else { 1. })
        }
    }

    pub fn apply_offset(&self, start: egui::Pos2, end: egui::Pos2) -> (egui::Pos2, egui::Pos2) {
        let dir = (end - start).normalized();
        let perp = dir.rot90();

        (
            start + perp * self.perpendicular_offset(),
            end + perp * self.perpendicular_offset(),
        )
    }

    pub fn get_renderable(&self, start: egui::Pos2, end: egui::Pos2) -> LinkGraphicsData {
        let (start, end) = self.apply_offset(start, end);
        LinkGraphicsData::get_link_renderable(start, end, 3.)
    }
}

impl PartialEq for Transition {
    fn eq(&self, rhs: &Self) -> bool {
        let mut lhs_start = self.spec.start_pin_index;
        let mut lhs_end = self.spec.end_pin_index;
        let mut rhs_start = rhs.spec.start_pin_index;
        let mut rhs_end = rhs.spec.end_pin_index;

        if lhs_start > lhs_end {
            std::mem::swap(&mut lhs_start, &mut lhs_end);
        }

        if rhs_start > rhs_end {
            std::mem::swap(&mut rhs_start, &mut rhs_end);
        }

        lhs_start == rhs_start && lhs_end == rhs_end
    }
}

#[derive(Debug)]
pub struct Line(egui::Pos2, egui::Pos2);

impl Line {
    pub fn eval(&self, t: f32) -> egui::Pos2 {
        (self.0 * (1. - t) + (self.1 * t).to_vec2()).into()
    }

    pub fn get_containing_rect(&self, hover_distance: f32) -> egui::Rect {
        let min = self.0.min(self.1);
        let max = self.0.max(self.1);

        let rect = egui::Rect::from_min_max(min, max);
        rect.expand(hover_distance)
    }
}

#[derive(Debug)]
pub(crate) struct LinkGraphicsData {
    pub line: Line,
    pub num_segments: usize,
}

impl LinkGraphicsData {
    pub fn get_link_renderable(
        start: egui::Pos2,
        end: egui::Pos2,
        line_segments_per_length: f32,
    ) -> Self {
        let link_length = end.distance(start);
        Self {
            line: Line(start, end),
            num_segments: 1.max((link_length * line_segments_per_length) as usize),
        }
    }

    pub(crate) fn get_closest_point_on_line(&self, p: &egui::Pos2) -> egui::Pos2 {
        let mut p_last = self.line.0;
        let mut p_closest = self.line.0;
        let mut p_closest_dist = f32::MAX;
        let t_step = 1.0 / self.num_segments as f32;
        for i in 1..self.num_segments {
            let p_current = self.line.eval(t_step * i as f32);
            let p_line = line_closest_point(&p_last, &p_current, p);
            let dist = p.distance_sq(p_line);
            if dist < p_closest_dist {
                p_closest = p_line;
                p_closest_dist = dist;
            }
            p_last = p_current;
        }
        p_closest
    }

    pub(crate) fn get_distance_to_line(&self, pos: &egui::Pos2) -> f32 {
        let point_on_curve = self.get_closest_point_on_line(pos);
        pos.distance(point_on_curve)
    }

    pub(crate) fn rectangle_overlaps_line(&self, rect: &egui::Rect) -> bool {
        let mut current = self.line.eval(0.0);
        let dt = 1.0 / self.num_segments as f32;
        for i in 0..self.num_segments {
            let next = self.line.eval((i + 1) as f32 * dt);
            if rectangle_overlaps_line_segment(rect, &current, &next) {
                return true;
            }
            current = next;
        }
        false
    }

    pub(crate) fn draw(
        &self,
        shape: ShapeIdx,
        arrow_shape: ShapeIdx,
        stroke: impl Into<egui::Stroke>,
        ui: &mut egui::Ui,
    ) {
        let stroke = stroke.into();
        let points = std::iter::once(self.line.0)
            .chain(
                (1..self.num_segments).map(|x| self.line.eval(x as f32 / self.num_segments as f32)),
            )
            .chain(std::iter::once(self.line.1))
            .collect();
        let path_shape = PathShape {
            points,
            closed: false,
            fill: egui::Color32::TRANSPARENT,
            stroke: stroke.into(),
        };
        ui.painter().set(shape, egui::Shape::Path(path_shape));
        self.draw_arrow(arrow_shape, stroke, ui);
    }

    fn draw_arrow(&self, shape: ShapeIdx, stroke: egui::Stroke, ui: &mut egui::Ui) {
        let arrow_size = 10.;
        let dir = (self.line.1 - self.line.0).normalized();
        let perp = dir.rot90();
        let mid = (self.line.0 + self.line.1.to_vec2()) / 2.;
        let arrow_start = mid - dir * arrow_size - perp * arrow_size;
        let arrow_end = mid - dir * arrow_size + perp * arrow_size;
        let points = vec![arrow_start, mid, arrow_end];
        let path_shape = PathShape {
            points,
            closed: true,
            fill: stroke.color,
            stroke,
        };
        ui.painter().set(shape, egui::Shape::Path(path_shape));
    }
}

pub fn line_closest_point(a: &egui::Pos2, b: &egui::Pos2, p: &egui::Pos2) -> egui::Pos2 {
    let ap = *p - *a;
    let ab_dir = *b - *a;
    let dot = ap.x * ab_dir.x + ap.y * ab_dir.y;
    if dot < 0.0 {
        return *a;
    }
    let ab_len_sqr = ab_dir.x * ab_dir.x + ab_dir.y * ab_dir.y;
    if dot > ab_len_sqr {
        return *b;
    }
    *a + ab_dir * dot / ab_len_sqr
}

fn eval_inplicit_line_eq(p1: &egui::Pos2, p2: &egui::Pos2, p: &egui::Pos2) -> f32 {
    (p2.y * p1.y) * p.x + (p1.x * p2.x) * p.y * (p2.x * p1.y - p1.x * p2.y)
}

fn rectangle_overlaps_line_segment(rect: &egui::Rect, p1: &egui::Pos2, p2: &egui::Pos2) -> bool {
    if rect.contains(*p1) || rect.contains(*p2) {
        return true;
    }

    let mut flip_rect = *rect;
    if flip_rect.min.x > flip_rect.max.x {
        std::mem::swap(&mut flip_rect.min.x, &mut flip_rect.max.x);
    }

    if flip_rect.min.y > flip_rect.max.y {
        std::mem::swap(&mut flip_rect.min.y, &mut flip_rect.max.y);
    }

    if (p1.x < flip_rect.min.x && p2.x < flip_rect.min.x)
        || (p1.x > flip_rect.max.x && p2.x > flip_rect.max.x)
        || (p1.y < flip_rect.min.y && p2.y < flip_rect.min.y)
        || (p1.y > flip_rect.max.y && p2.y > flip_rect.max.y)
    {
        return false;
    }

    let corner_signs = [
        eval_inplicit_line_eq(p1, p2, &flip_rect.left_bottom()).signum(),
        eval_inplicit_line_eq(p1, p2, &flip_rect.left_top()).signum(),
        eval_inplicit_line_eq(p1, p2, &flip_rect.right_bottom()).signum(),
        eval_inplicit_line_eq(p1, p2, &flip_rect.right_top()).signum(),
    ];

    let mut sum = 0.0;
    let mut sum_abs = 0.0;
    for sign in corner_signs.iter() {
        sum += sign;
        sum_abs += sign.abs();
    }

    (sum.abs() - sum_abs).abs() < f32::EPSILON
}
