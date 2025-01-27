use bevy::prelude::*;

#[derive(Default, Clone, Copy, Reflect, Debug, PartialEq, Eq, Hash)]
pub enum VertexId {
    /// E.g. part of the super-triangle in the delaunay triangulation algorithm
    #[default]
    NotProvided,
    /// An index into the original vertex list
    Index(usize),
}

#[derive(Default, Clone, Copy, Reflect, Debug, PartialEq)]
pub struct Vertex {
    pub val: Vec2,
    pub id: VertexId,
}

impl Vertex {
    pub fn new(p: Vec2, id: VertexId) -> Self {
        Self { val: p, id }
    }
}

#[derive(Default, Clone, Reflect, Debug)]
pub struct Edge {
    pub p: Vertex,
    pub q: Vertex,
}

impl Edge {
    pub fn new(p: Vertex, q: Vertex) -> Self {
        Self { p, q }
    }

    pub fn compare_value(&self, other: &Self) -> bool {
        (self.p.val == other.p.val && self.q.val == other.q.val)
            || (self.p.val == other.q.val && self.q.val == other.p.val)
    }

    pub fn closest_point_to(&self, v: Vec2) -> Vec2 {
        let pq = self.q.val - self.p.val;
        let pv = v - self.p.val;

        let t = pv.dot(pq) / pq.length_squared();

        let closest_point = if t < 0. {
            self.p.val
        } else if t > 1. {
            self.q.val
        } else {
            self.p.val + pq * t
        };

        closest_point
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.p == other.p && self.q == other.q) || (self.p == other.q && self.q == other.p)
    }
}

#[derive(Default, Clone, Reflect, Debug)]
pub struct Triangle {
    pub p: Vertex,
    pub q: Vertex,
    pub r: Vertex,
}

impl Triangle {
    pub fn new(p: Vertex, q: Vertex, r: Vertex) -> Self {
        Self { p, q, r }
    }

    pub fn super_triangle(vertices: &[Vertex]) -> Self {
        let min = vertices
            .iter()
            .copied()
            .map(|v| v.val)
            .fold(Vec2::INFINITY, Vec2::min);
        let max = vertices
            .iter()
            .copied()
            .map(|v| v.val)
            .fold(-Vec2::INFINITY, Vec2::max);
        let margin = (max - min) * 10.;
        Self {
            p: Vertex::new(
                Vec2::new(min.x - margin.x, min.y - margin.y * 3.),
                VertexId::NotProvided,
            ),
            q: Vertex::new(
                Vec2::new(min.x - margin.x, max.y + margin.y),
                VertexId::NotProvided,
            ),
            r: Vertex::new(
                Vec2::new(max.x + margin.x * 3., max.y + margin.y),
                VertexId::NotProvided,
            ),
        }
    }

    pub fn circumcenter(&self) -> Option<Vec2> {
        let pq = Line::from_points(self.p.val, self.q.val);
        let qr = Line::from_points(self.q.val, self.r.val);

        let pq_bisector = pq.bisector_between(self.p.val, self.q.val);
        let qr_bisector = qr.bisector_between(self.q.val, self.r.val);

        pq_bisector.intersection(&qr_bisector)
    }

    pub fn circumradius(&self) -> Option<f32> {
        let circumcenter = self.circumcenter()?;
        Some(self.p.val.distance(circumcenter))
    }

    pub fn circumradius_given_center(&self, circumcenter: Vec2) -> f32 {
        self.p.val.distance(circumcenter)
    }

    pub fn edges(&self) -> [Edge; 3] {
        [
            Edge::new(self.p, self.q),
            Edge::new(self.q, self.r),
            Edge::new(self.r, self.p),
        ]
    }

    pub fn has_vertices_in_common(&self, other: &Triangle) -> bool {
        self.p == other.p
            || self.p == other.q
            || self.p == other.r
            || self.q == other.p
            || self.q == other.q
            || self.q == other.r
            || self.r == other.p
            || self.r == other.q
            || self.r == other.r
    }

    pub fn barycentric_coordinates(&self, v: Vec2) -> Vec3 {
        let v0 = self.q.val - self.p.val;
        let v1 = self.r.val - self.p.val;
        let v2 = v - self.p.val;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);

        let denom = d00 * d11 - d01 * d01;
        let y = (d11 * d20 - d01 * d21) / denom;
        let z = (d00 * d21 - d01 * d20) / denom;
        let x = 1. - y - z;

        Vec3::new(x, y, z)
    }

    pub fn contains(&self, v: Vec2) -> bool {
        let bary = self.barycentric_coordinates(v);

        bary.x >= 0. && bary.y >= 0. && bary.z >= 0.
    }
}

#[derive(Default, Reflect, Clone, Debug)]
pub struct CachedTriangle {
    triangle: Triangle,
    circumcenter: Option<Vec2>,
    circumradius: Option<f32>,
}

impl CachedTriangle {
    pub fn from_triangle(triangle: Triangle) -> Self {
        let circumcenter = triangle.circumcenter();
        let circumradius = circumcenter.map(|c| triangle.circumradius_given_center(c));

        Self {
            triangle,
            circumcenter,
            circumradius,
        }
    }

    pub fn inner(&self) -> &Triangle {
        &self.triangle
    }

    pub fn into_inner(self) -> Triangle {
        self.triangle
    }

    pub fn in_circumcircle(&self, v: Vec2) -> Option<bool> {
        Some(v.distance(self.circumcenter?) <= self.circumradius?)
    }

    /// Finds the closest point inside the triangle to the given point, and the distance to it.
    ///
    /// Note that the distance is 0 if the point is inside the triangle.
    pub fn distance_to_point(&self, p: Vec2) -> (Vec2, f32) {
        if self.triangle.contains(p) {
            (p, 0.)
        } else {
            let (closest_point, closest_distance_squared) = self
                .triangle
                .edges()
                .into_iter()
                .map(|edge| edge.closest_point_to(p))
                .map(|closest_point| (closest_point, closest_point.distance_squared(p)))
                .min_by(|(_, d0), (_, d1)| d0.partial_cmp(d1).unwrap())
                .unwrap();

            (closest_point, closest_distance_squared.sqrt())
        }
    }
}

/// In form ax + by = c
struct Line {
    a: f32,
    b: f32,
    c: f32,
}

impl Line {
    fn from_points(p: Vec2, q: Vec2) -> Self {
        let a = q.y - p.y;
        let b = p.x - q.x;
        let c = a * p.x + b * p.y;

        Self { a, b, c }
    }

    fn intersection(&self, other: &Line) -> Option<Vec2> {
        let determinant = self.a * other.b - other.a * self.b;
        if determinant == 0. {
            None
        } else {
            let x = other.b * self.c - self.b * other.c;
            let y = self.a * other.c - other.a * self.c;

            Some(Vec2::new(x, y))
        }
    }

    fn bisector_between(&self, p: Vec2, q: Vec2) -> Self {
        let midpoint = (p + q) / 2.;
        let c = -self.b * midpoint.x + self.a * midpoint.y;
        let a = -self.b;
        let b = self.a;

        Self { a, b, c }
    }
}
