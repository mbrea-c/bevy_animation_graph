use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::new_context::NodeContext;
use crate::prelude::{EditProxy, ReflectEditProxy, SpecContext};
use crate::utils::delaunay::Triangulation;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::BlendSyncMode;

#[derive(Reflect, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[reflect(Default, Serialize)]
pub enum BlendMode {
    #[default]
    LinearizedInterpolate,
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
        format!("pose {key}")
    }

    pub fn events_pin_id(key: &str) -> String {
        format!("events {key}")
    }

    pub fn time_pin_id(key: &str) -> String {
        format!("time {key}")
    }

    fn vertex_key(&self, id: usize) -> &str {
        &self.points[id].id
    }
}

impl NodeLike for BlendSpaceNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let position = ctx.data_back(Self::POSITION)?.as_vec2()?;
        let (v0, _) = self
            .triangulation
            .find_linear_combination(position)
            .into_iter()
            .max_by(|l, r| l.1.partial_cmp(&r.1).unwrap())
            .unwrap();

        // We return the duration of the input with the highest weight
        let master_duration =
            ctx.duration_back(Self::time_pin_id(self.vertex_key(v0.id.index())))?;

        ctx.set_duration_fwd(master_duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        let position = ctx.data_back(Self::POSITION)?.as_vec2()?;
        let mut linear_combination = self.triangulation.find_linear_combination(position);
        // sorted by highest weight first, lowest weight last
        linear_combination.sort_by(|l, r| l.1.partial_cmp(&r.1).unwrap().reverse());

        let (v0, f0) = linear_combination[0];
        let (v1, f1) = linear_combination[1];
        let (v2, f2) = linear_combination[2];

        ctx.set_time_update_back(
            Self::time_pin_id(self.vertex_key(v0.id.index())),
            input.clone(),
        );
        let pose_0 = ctx
            .data_back(Self::pose_pin_id(self.vertex_key(v0.id.index())))?
            .into_pose()?;

        match &self.sync_mode {
            BlendSyncMode::Absolute => {
                ctx.set_time_update_back(
                    Self::time_pin_id(self.vertex_key(v1.id.index())),
                    TimeUpdate::Absolute(pose_0.timestamp),
                );
                ctx.set_time_update_back(
                    Self::time_pin_id(self.vertex_key(v2.id.index())),
                    TimeUpdate::Absolute(pose_0.timestamp),
                );
            }
            BlendSyncMode::NoSync => {
                ctx.set_time_update_back(
                    Self::time_pin_id(self.vertex_key(v1.id.index())),
                    input.clone(),
                );
                ctx.set_time_update_back(Self::time_pin_id(self.vertex_key(v2.id.index())), input);
            }
            BlendSyncMode::EventTrack(track_name) => {
                let event_queue_0 = ctx
                    .data_back(Self::events_pin_id(self.vertex_key(v0.id.index())))?
                    .into_event_queue()?;
                if let Some(event) = event_queue_0.events.iter().find(|ev| {
                    ev.track
                        .as_ref()
                        .map(|track| track == track_name)
                        .unwrap_or(false)
                }) {
                    let time_update = TimeUpdate::PercentOfEvent {
                        percent: event.percentage,
                        event: event.event.clone(),
                        track: track_name.clone(),
                    };

                    ctx.set_time_update_back(
                        Self::time_pin_id(self.vertex_key(v1.id.index())),
                        time_update.clone(),
                    );
                    ctx.set_time_update_back(
                        Self::time_pin_id(self.vertex_key(v2.id.index())),
                        time_update,
                    );
                } else {
                    ctx.set_time_update_back(
                        Self::time_pin_id(self.vertex_key(v1.id.index())),
                        input.clone(),
                    );
                    ctx.set_time_update_back(
                        Self::time_pin_id(self.vertex_key(v2.id.index())),
                        input,
                    );
                }
            }
        };

        let pose_1 = ctx
            .data_back(Self::pose_pin_id(self.vertex_key(v1.id.index())))?
            .into_pose()?;
        let pose_2 = ctx
            .data_back(Self::pose_pin_id(self.vertex_key(v2.id.index())))?
            .into_pose()?;

        // We do a linearized weighted average.
        // While this is not the mathematically correct way of averaging quaternions,
        // it's fast and in practice provides decent results.
        let out_pose = match self.mode {
            BlendMode::LinearizedInterpolate => [(&pose_0, f0), (&pose_1, f1), (&pose_2, f2)]
                .into_iter()
                .map(|(pose, f)| pose.scalar_mult(f))
                .reduce(|pose_acc, pose_elem| pose_acc.linear_add(&pose_elem))
                .unwrap()
                .normalize_quat(),
        };

        ctx.set_time(pose_0.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, out_pose);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        let mut input_spec = PinMap::from([(Self::POSITION.into(), DataSpec::Vec2)]);

        input_spec.extend(self.points.iter().flat_map(|p| {
            let mut out = vec![(Self::pose_pin_id(&p.id), DataSpec::Pose)];

            if matches!(self.sync_mode, BlendSyncMode::EventTrack(_)) {
                out.push((Self::events_pin_id(&p.id), DataSpec::EventQueue));
            }

            out
        }));

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
            sync_mode: proxy.sync_mode.clone(),
            points: proxy.points.clone(),
            triangulation: Triangulation::from_points_delaunay(
                proxy.points.clone().into_iter().map(|x| x.point).collect(),
            ),
        }
    }

    fn make_proxy(&self) -> Self::Proxy {
        Self::Proxy {
            mode: self.mode,
            sync_mode: self.sync_mode.clone(),
            points: self.points.clone(),
        }
    }
}
