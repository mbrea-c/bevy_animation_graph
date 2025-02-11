use bevy::{math::Vec2, reflect::Reflect};

use super::geometry::{CachedTriangle, Triangle, Vertex, VertexId};

#[derive(Default, Reflect, Clone, Debug)]
pub struct Triangulation {
    triangles: Vec<CachedTriangle>,
}

impl Triangulation {
    pub fn from_points_delaunay(points: Vec<Vec2>) -> Self {
        let vertices = points
            .into_iter()
            .enumerate()
            .map(|(i, v)| Vertex::new(v, VertexId::Index(i)))
            .collect::<Vec<_>>();
        let super_triangle = Triangle::super_triangle(&vertices);
        let mut triangles = vec![CachedTriangle::from_triangle(super_triangle.clone())];
        for vertex in vertices {
            triangles = add_vertex(triangles, vertex);
        }

        Triangulation {
            triangles: triangles
                .into_iter()
                .filter(|triangle| triangle.inner().all_vertices_have_index())
                .collect(),
        }
    }

    pub fn find_linear_combination(&self, p: Vec2) -> Vec<(Vertex, f32)> {
        let (i, (closest_p, _)) = self
            .triangles
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.distance_to_point(p)))
            .min_by(|(_, (_, d0)), (_, (_, d1))| d0.partial_cmp(d1).unwrap())
            .unwrap();

        let triangle = self.triangles[i].inner();
        let bary = triangle.barycentric_coordinates(closest_p);

        vec![
            (triangle.p, bary.x),
            (triangle.q, bary.y),
            (triangle.r, bary.z),
        ]
    }
}

fn add_vertex(mut triangles: Vec<CachedTriangle>, vertex: Vertex) -> Vec<CachedTriangle> {
    let mut edges = vec![];

    triangles = triangles
        .into_iter()
        .filter(|triangle| {
            if triangle.in_circumcircle(vertex.val).unwrap_or(true) {
                edges.extend(triangle.inner().edges());
                false
            } else {
                true
            }
        })
        .collect();

    edges.dedup();

    edges.into_iter().for_each(|edge| {
        triangles.push(CachedTriangle::from_triangle(Triangle::new(
            edge.p, edge.q, vertex,
        )))
    });

    triangles
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_triangulation() {
        let triangulation = Triangulation::from_points_delaunay(vec![
            Vec2::new(0., 0.),
            Vec2::new(1., 0.),
            Vec2::new(0., 1.),
        ]);

        assert_eq!(triangulation.triangles.len(), 1);
    }

    #[test]
    fn test_linear_combination() {
        let triangulation = Triangulation::from_points_delaunay(vec![
            Vec2::new(0., 0.),
            Vec2::new(1., 0.),
            Vec2::new(0., 1.),
        ]);

        let linear_combination = triangulation.find_linear_combination(Vec2::new(0.1, 0.1));

        assert_eq!(
            1.,
            linear_combination[0].1 + linear_combination[1].1 + linear_combination[2].1
        );
    }
}
