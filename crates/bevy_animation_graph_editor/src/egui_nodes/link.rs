use super::pin::PinType;
use bevy_egui::egui;
use bevy_inspector_egui::bevy_egui;
use derivative::Derivative;
use egui::epaint::PathShape;

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
    pub thickness: f32,
}

impl Default for LinkStyle {
    fn default() -> Self {
        Self {
            base: egui::Color32::BLUE,
            hovered: egui::Color32::LIGHT_BLUE,
            selected: egui::Color32::LIGHT_BLUE,
            thickness: 3.,
        }
    }
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct LinkSpec {
    pub id: usize,
    pub start_pin_index: usize,
    pub end_pin_index: usize,
    #[derivative(Debug = "ignore")]
    pub style: LinkStyleArgs,
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct LinkState {
    #[derivative(Debug = "ignore")]
    pub style: LinkStyle,
    #[derivative(Debug = "ignore")]
    pub shape: Option<egui::layers::ShapeIdx>,
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Link {
    pub spec: LinkSpec,
    pub state: LinkState,
}

impl PartialEq for Link {
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
pub struct BezierCurve(egui::Pos2, egui::Pos2, egui::Pos2, egui::Pos2);

impl BezierCurve {
    pub fn eval(&self, t: f32) -> egui::Pos2 {
        <[f32; 2]>::from(
            (1.0 - t).powi(3) * self.0.to_vec2()
                + 3.0 * (1.0 - t).powi(2) * t * self.1.to_vec2()
                + 3.0 * (1.0 - t) * t.powi(2) * self.2.to_vec2()
                + t.powi(3) * self.3.to_vec2(),
        )
        .into()
    }

    pub fn get_containing_rect_for_bezier_curve(&self, hover_distance: f32) -> egui::Rect {
        let min = self.0.min(self.3);
        let max = self.0.max(self.3);

        let mut rect = egui::Rect::from_min_max(min, max);
        rect.extend_with(self.1);
        rect.extend_with(self.2);
        rect.expand(hover_distance)
    }
}

#[derive(Debug)]
pub(crate) struct LinkBezierData {
    pub bezier: BezierCurve,
    pub num_segments: usize,
}

impl LinkBezierData {
    pub(crate) fn get_link_renderable(
        start: egui::Pos2,
        end: egui::Pos2,
        start_type: PinType,
        line_segments_per_length: f32,
    ) -> Self {
        let (mut start, mut end) = (start, end);
        if start_type == PinType::Input {
            std::mem::swap(&mut start, &mut end);
        }

        let link_length = end.distance(start);
        let offset = egui::vec2(0.25 * link_length, 0.0);
        Self {
            bezier: BezierCurve(start, start + offset, end - offset, end),
            num_segments: 1.max((link_length * line_segments_per_length) as usize),
        }
    }

    pub(crate) fn get_closest_point_on_cubic_bezier(&self, p: &egui::Pos2) -> egui::Pos2 {
        let mut p_last = self.bezier.0;
        let mut p_closest = self.bezier.0;
        let mut p_closest_dist = f32::MAX;
        let t_step = 1.0 / self.num_segments as f32;
        for i in 1..self.num_segments {
            let p_current = self.bezier.eval(t_step * i as f32);
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

    pub(crate) fn get_distance_to_cubic_bezier(&self, pos: &egui::Pos2) -> f32 {
        let point_on_curve = self.get_closest_point_on_cubic_bezier(pos);
        pos.distance(point_on_curve)
    }

    pub(crate) fn rectangle_overlaps_bezier(&self, rect: &egui::Rect) -> bool {
        let mut current = self.bezier.eval(0.0);
        let dt = 1.0 / self.num_segments as f32;
        for i in 0..self.num_segments {
            let next = self.bezier.eval((i + 1) as f32 * dt);
            if rectangle_overlaps_line_segment(rect, &current, &next) {
                return true;
            }
            current = next;
        }
        false
    }

    pub(crate) fn draw(&self, stroke: impl Into<egui::Stroke>) -> egui::Shape {
        let points = std::iter::once(self.bezier.0)
            .chain(
                (1..self.num_segments)
                    .map(|x| self.bezier.eval(x as f32 / self.num_segments as f32)),
            )
            .chain(std::iter::once(self.bezier.3))
            .collect();
        let path_shape = PathShape {
            points,
            closed: false,
            fill: egui::Color32::TRANSPARENT,
            stroke: stroke.into().into(),
        };
        egui::Shape::Path(path_shape)
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
