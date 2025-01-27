use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::prelude::{EditProxy, InterpolateLinear, PassContext, SpecContext};
use crate::utils::delaunay::Triangulation;
use crate::utils::unwrap::UnwrapVal;
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

/// Allows you to map input poses to points in a 2D plane. A Delaunay triangulation of these points
/// will be computed at load time, and at runtime the poses assigned to the vertices of closest triangle
/// to the "position" input will be interpolated based on the barycentric coordinates of the
/// supplied point. If the point is outside all triangles, it will be projected to the closest
/// triangle.
///
/// This node is useful, for example, to blend between directional movement and strafe animations
/// in a shooter game.
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct BlendSpaceNode {
    pub mode: BlendMode,
    pub sync_mode: BlendSyncMode,
    pub points: Vec<(String, Vec2)>,
    pub triangulation: Triangulation,
}

impl BlendSpaceNode {
    /// 2D position in the blend space to sample
    pub const POSITION: &'static str = "position";
    pub const IN_POSE_A: &'static str = "pose A";
    pub const IN_TIME_A: &'static str = "time A";
    pub const IN_POSE_B: &'static str = "pose B";
    pub const IN_TIME_B: &'static str = "time B";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(mode: BlendMode, sync_mode: BlendSyncMode, points: Vec<(String, Vec2)>) -> Self {
        Self {
            mode,
            sync_mode,
            points: points.clone(),
            triangulation: Triangulation::from_points_delaunay(
                points.into_iter().map(|x| x.1).collect(),
            ),
        }
    }
}

impl NodeLike for BlendSpaceNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration_1 = ctx.duration_back(Self::IN_TIME_A)?;
        let duration_2 = ctx.duration_back(Self::IN_TIME_B)?;

        let out_duration = match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        ctx.set_time_update_back(Self::IN_TIME_A, input);
        let in_frame_1: Pose = ctx.data_back(Self::IN_POSE_A)?.val();

        match self.sync_mode {
            BlendSyncMode::Absolute => {
                ctx.set_time_update_back(
                    Self::IN_TIME_B,
                    TimeUpdate::Absolute(in_frame_1.timestamp),
                );
            }
            BlendSyncMode::NoSync => {
                ctx.set_time_update_back(Self::IN_TIME_B, input);
            }
        };

        let in_frame_2: Pose = ctx.data_back(Self::IN_POSE_B)?.val();

        let out = match self.mode {
            BlendMode::LinearInterpolate => {
                let alpha = ctx.data_back(Self::POSITION)?.unwrap_f32();
                in_frame_1.interpolate_linear(&in_frame_2, alpha)
            }
            BlendMode::Additive => {
                let alpha = ctx.data_back(Self::POSITION)?.unwrap_f32();
                in_frame_1.additive_blend(&in_frame_2, alpha)
            }
            BlendMode::Difference => in_frame_1.difference(&in_frame_2),
        };

        ctx.set_time(out.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, out);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        let mut input_spec = PinMap::from([(Self::POSITION.into(), DataSpec::F32)]);

        input_spec.extend(
            self.points
                .iter()
                .map(|(key, _)| (format!("pose {}", key), DataSpec::Pose)),
        );

        input_spec
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        let mut input_spec = PinMap::new();

        input_spec.extend(
            self.points
                .iter()
                .map(|(key, _)| (format!("time {}", key), ())),
        );

        input_spec
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "∑ Blend".into()
    }
}

#[derive(Clone, Reflect)]
pub struct FlipLRProxy {
    pub mode: BlendMode,
    pub sync_mode: BlendSyncMode,
    pub points: Vec<(String, Vec2)>,
}

impl EditProxy for BlendSpaceNode {
    type Proxy = FlipLRProxy;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self {
        Self {
            mode: proxy.mode,
            sync_mode: proxy.sync_mode,
            points: proxy.points.clone(),
            triangulation: Triangulation::from_points_delaunay(
                proxy.points.clone().into_iter().map(|x| x.1).collect(),
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
