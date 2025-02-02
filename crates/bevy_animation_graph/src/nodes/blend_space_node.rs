use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{EditProxy, PassContext, ReflectEditProxy, SpecContext};
use crate::utils::delaunay::Triangulation;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[reflect(Default, Serialize)]
pub enum BlendMode {
    #[default]
    LinearInterpolate,
    Additive,
    Difference,
}

#[derive(Reflect, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub enum BlendSyncMode {
    /// Sets the absolute timestamp of input 2 equal to the timestamp from input 1
    #[default]
    Absolute,
    /// Propagates the same time update that was received, does not try to sync the inputs.
    NoSync,
}

#[derive(Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
pub struct PointElement {
    pub id: String,
    pub point: Vec2,
}

/// Allows you to map input poses to points in a 2D plane. A Delaunay triangulation of these points
/// will be computed at load time, and at runtime the poses assigned to the vertices of closest triangle
/// to the "position" input will be interpolated based on the barycentric coordinates of the
/// supplied point. If the point is outside all triangles, it will be projected to the closest
/// triangle.
///
/// This node is useful, for example, to blend between directional movement and strafe animations
/// in a shooter game.
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike, EditProxy)]
pub struct BlendSpaceNode {
    pub mode: BlendMode,
    pub sync_mode: BlendSyncMode,
    pub points: Vec<PointElement>,
    pub triangulation: Triangulation,
}

impl BlendSpaceNode {
    /// 2D position in the blend space to sample
    pub const POSITION: &'static str = "position";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(mode: BlendMode, sync_mode: BlendSyncMode, points: Vec<PointElement>) -> Self {
        Self {
            mode,
            sync_mode,
            points: points.clone(),
            triangulation: Triangulation::from_points_delaunay(
                points.into_iter().map(|x| x.point).collect(),
            ),
        }
    }

    pub fn pose_pin_id(key: &str) -> String {
        format!("pose {}", key)
    }

    pub fn time_pin_id(key: &str) -> String {
        format!("time {}", key)
    }

    fn vertex_key(&self, id: usize) -> &str {
        &self.points[id].id
    }
}

impl NodeLike for BlendSpaceNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let position = ctx.data_back(Self::POSITION)?.as_vec2().unwrap();
        let [(v0, _), (v1, _), (v2, _)] = self.triangulation.find_linear_combination(position);

        let duration_0 = ctx.duration_back(Self::time_pin_id(self.vertex_key(v0.id.index())))?;
        let duration_1 = ctx.duration_back(Self::time_pin_id(self.vertex_key(v1.id.index())))?;
        let duration_2 = ctx.duration_back(Self::time_pin_id(self.vertex_key(v2.id.index())))?;

        let out_duration = if duration_0.is_none() && duration_1.is_none() && duration_2.is_none() {
            None
        } else {
            Some(
                duration_0
                    .unwrap_or(0.)
                    .max(duration_1.unwrap_or(0.))
                    .max(duration_2.unwrap_or(0.)),
            )
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        let position = ctx.data_back(Self::POSITION)?.as_vec2().unwrap();
        let [(v0, f0), (v1, f1), (v2, f2)] = self.triangulation.find_linear_combination(position);

        ctx.set_time_update_back(Self::time_pin_id(self.vertex_key(v0.id.index())), input);
        ctx.set_time_update_back(Self::time_pin_id(self.vertex_key(v1.id.index())), input);
        ctx.set_time_update_back(Self::time_pin_id(self.vertex_key(v2.id.index())), input);

        let pose_0 = ctx
            .data_back(Self::pose_pin_id(self.vertex_key(v0.id.index())))?
            .into_pose()
            .unwrap();
        let pose_1 = ctx
            .data_back(Self::pose_pin_id(self.vertex_key(v1.id.index())))?
            .into_pose()
            .unwrap();
        let pose_2 = ctx
            .data_back(Self::pose_pin_id(self.vertex_key(v2.id.index())))?
            .into_pose()
            .unwrap();

        // We do a linearized weighted average.
        // While this is not the mathematically correct way of averaging quaternions,
        // it's fast and in practice provides decent results.
        let out_pose = pose_0
            .scalar_mult(f0)
            .linear_add(&pose_1.scalar_mult(f1))
            .linear_add(&pose_2.scalar_mult(f2))
            .normalize_quat();

        // TODO: Find master animation track and sync according to sync_mode
        // match self.sync_mode {
        //     BlendSyncMode::Absolute => {
        //         ctx.set_time_update_back(
        //             Self::IN_TIME_B,
        //             TimeUpdate::Absolute(in_frame_1.timestamp),
        //         );
        //     }
        //     BlendSyncMode::NoSync => {
        //         ctx.set_time_update_back(Self::IN_TIME_B, input);
        //     }
        // };

        // TODO: Blend according to blend_mode
        // let out = match self.mode {
        //     BlendMode::LinearInterpolate => {
        //         let alpha = ctx.data_back(Self::POSITION)?.unwrap_f32();
        //         in_frame_1.interpolate_linear(&in_frame_2, alpha)
        //     }
        //     BlendMode::Additive => {
        //         let alpha = ctx.data_back(Self::POSITION)?.unwrap_f32();
        //         in_frame_1.additive_blend(&in_frame_2, alpha)
        //     }
        //     BlendMode::Difference => in_frame_1.difference(&in_frame_2),
        // };

        ctx.set_time(out_pose.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, out_pose);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        let mut input_spec = PinMap::from([(Self::POSITION.into(), DataSpec::Vec2)]);

        input_spec.extend(
            self.points
                .iter()
                .map(|p| (Self::pose_pin_id(&p.id), DataSpec::Pose)),
        );

        input_spec
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        let mut input_spec = PinMap::new();

        input_spec.extend(self.points.iter().map(|p| (Self::time_pin_id(&p.id), ())));

        input_spec
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "∑ Blend space 2D".into()
    }
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct FlipLRProxy {
    pub mode: BlendMode,
    pub sync_mode: BlendSyncMode,
    pub points: Vec<PointElement>,
}

impl EditProxy for BlendSpaceNode {
    type Proxy = FlipLRProxy;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self {
        Self {
            mode: proxy.mode,
            sync_mode: proxy.sync_mode,
            points: proxy.points.clone(),
            triangulation: Triangulation::from_points_delaunay(
                proxy.points.clone().into_iter().map(|x| x.point).collect(),
            ),
        }
    }

    fn make_proxy(&self) -> Self::Proxy {
        Self::Proxy {
            mode: self.mode,
            sync_mode: self.sync_mode,
            points: self.points.clone(),
        }
    }
}
