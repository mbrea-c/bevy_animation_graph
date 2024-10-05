use std::ops::{Add, Mul};

use crate::core::animation_clip::{GraphClip, Interpolation, Keyframes, VariableCurve};
use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::id::BoneId;
use crate::core::pose::{BonePose, Pose};
use crate::core::prelude::{DataSpec, DataValue};
use crate::core::systems::get_keyframe;
use crate::interpolation::prelude::InterpolateStep;
use crate::prelude::{InterpolateLinear, PassContext, SpecContext};
use crate::utils::asset::GetTypedExt;
use bevy::asset::Handle;
use bevy::reflect::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::pose"]
pub struct Clip {
    pub(crate) clip: Handle<GraphClip>,
    pub(crate) override_duration: Option<f32>,
    pub(crate) override_interpolation: Option<Interpolation>,
}

impl Clip {
    pub const OUT: &str = "out";

    #[inline]
    pub fn clip_duration(&self, ctx: &PassContext) -> f32 {
        if let Some(duration) = self.override_duration {
            duration
        } else {
            ctx.resources
                .graph_clip_assets
                .get_typed(&self.clip, &ctx.resources.loaded_untyped_assets)
                .unwrap()
                .duration()
        }
    }
}

impl NodeLike for Clip {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_duration_fwd(Some(self.clip_duration(&ctx)));
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let clip_duration = self.clip_duration(&ctx);

        let Some(clip) = ctx
            .resources
            .graph_clip_assets
            .get_typed(&self.clip, &ctx.resources.loaded_untyped_assets)
        else {
            // TODO: Should we propagate a GraphError instead?
            ctx.set_data_fwd(Self::OUT, DataValue::Pose(Pose::default()));
            return Ok(());
        };

        let prev_time = ctx.prev_time();
        let time_update = ctx.time_update_fwd()?;
        let time = time_update.apply(prev_time);

        ctx.set_time(time);

        let mut out_pose = Pose {
            timestamp: time,
            skeleton: clip.skeleton.clone(),
            ..Pose::default()
        };

        let time = time.clamp(0., clip_duration);

        for (bone_id, curves) in &clip.curves {
            let mut bone_pose = BonePose::default();
            for curve in curves {
                // Some curves have only one keyframe used to set a transform
                let keyframe_count = curve.keyframe_timestamps.len();

                // Find the current keyframe
                // PERF: finding the current keyframe can be optimised
                let (step_start, step_end, prev_is_wrapped, next_is_wrapped) = match curve
                    .keyframe_timestamps
                    .binary_search_by(|probe| probe.partial_cmp(&time).unwrap())
                {
                    // this curve is finished
                    Ok(n) if n >= curve.keyframe_timestamps.len() - 1 => (n, 0, false, true),
                    Ok(i) => (i, i + 1, false, false),
                    // this curve isn't started yet
                    Err(0) => (curve.keyframe_timestamps.len() - 1, 0, true, false),
                    // this curve is finished
                    Err(n) if n > curve.keyframe_timestamps.len() - 1 => (n - 1, 0, false, true),
                    Err(i) => (i - 1, i, false, false),
                };

                if prev_is_wrapped {
                    sample_one_keyframe(step_end, keyframe_count, &curve.keyframes, &mut bone_pose);
                    continue;
                }
                if next_is_wrapped {
                    sample_one_keyframe(
                        step_start,
                        keyframe_count,
                        &curve.keyframes,
                        &mut bone_pose,
                    );
                    continue;
                }
                if step_start == step_end {
                    sample_one_keyframe(
                        step_start,
                        keyframe_count,
                        &curve.keyframes,
                        &mut bone_pose,
                    );
                    continue;
                }

                let mut prev_timestamp = curve.keyframe_timestamps[step_start];
                let mut next_timestamp = curve.keyframe_timestamps[step_end];

                if prev_is_wrapped {
                    prev_timestamp -= clip_duration;
                } else if next_is_wrapped {
                    next_timestamp += clip_duration;
                }

                sample_two_keyframes(
                    step_start,
                    step_end,
                    prev_timestamp,
                    next_timestamp,
                    time,
                    keyframe_count,
                    self.override_interpolation.unwrap_or(curve.interpolation),
                    curve,
                    &mut bone_pose,
                );
            }
            out_pose.add_bone(bone_pose, BoneId::from(*bone_id));
        }

        ctx.set_data_fwd(Self::OUT, DataValue::Pose(out_pose));

        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Pose)].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "⏵ Animation Clip".into()
    }
}

#[allow(clippy::too_many_arguments)]
fn sample_two_keyframes(
    step_start: usize,
    step_end: usize,
    prev_timestamp: f32,
    next_timestamp: f32,
    time: f32,
    keyframe_count: usize,
    override_interpolation: Interpolation,
    curve: &VariableCurve,
    bone_pose: &mut BonePose,
) {
    let lerp = if next_timestamp == prev_timestamp {
        1.
    } else {
        (time - prev_timestamp) / (next_timestamp - prev_timestamp)
    };
    let duration = next_timestamp - prev_timestamp;

    // Apply the keyframe
    match &curve.keyframes {
        Keyframes::Rotation(keyframes) => {
            let (prev, mut next, tangent_out_start, tangent_in_end) = match &curve.interpolation {
                Interpolation::Linear | Interpolation::Step => {
                    let prev = keyframes[step_start];
                    let next = keyframes[step_end];

                    (prev, next, Default::default(), Default::default())
                }
                Interpolation::CubicSpline => {
                    let prev = keyframes[step_start * 3 + 1];
                    let next = keyframes[step_end * 3 + 1];
                    let tangent_out_start = keyframes[step_start * 3 + 2];
                    let tangent_in_end = keyframes[step_end * 3];
                    (prev, next, tangent_out_start, tangent_in_end)
                }
            };

            // Choose the smallest angle for the rotation
            if next.dot(prev) < 0.0 {
                next = -next;
            }

            bone_pose.rotation = Some(match override_interpolation {
                Interpolation::Linear => prev.interpolate_linear(&next, lerp),
                Interpolation::Step => prev.interpolate_step(&next, lerp),
                Interpolation::CubicSpline => cubic_spline_interpolation(
                    prev,
                    tangent_out_start,
                    tangent_in_end,
                    next,
                    lerp,
                    duration,
                ),
            });
        }

        Keyframes::Translation(keyframes) => {
            let (prev, next, tangent_out_start, tangent_in_end) = match &curve.interpolation {
                Interpolation::Linear | Interpolation::Step => {
                    let prev = keyframes[step_start];
                    let next = keyframes[step_end];

                    (prev, next, Default::default(), Default::default())
                }
                Interpolation::CubicSpline => {
                    let prev = keyframes[step_start * 3 + 1];
                    let next = keyframes[step_end * 3 + 1];
                    let tangent_out_start = keyframes[step_start * 3 + 2];
                    let tangent_in_end = keyframes[step_end * 3];
                    (prev, next, tangent_out_start, tangent_in_end)
                }
            };

            bone_pose.translation = Some(match override_interpolation {
                Interpolation::Linear => prev.interpolate_linear(&next, lerp),
                Interpolation::Step => prev.interpolate_step(&next, lerp),
                Interpolation::CubicSpline => cubic_spline_interpolation(
                    prev,
                    tangent_out_start,
                    tangent_in_end,
                    next,
                    lerp,
                    duration,
                ),
            });
        }

        Keyframes::Scale(keyframes) => {
            let (prev, next, tangent_out_start, tangent_in_end) = match &curve.interpolation {
                Interpolation::Linear | Interpolation::Step => {
                    let prev = keyframes[step_start];
                    let next = keyframes[step_end];

                    (prev, next, Default::default(), Default::default())
                }
                Interpolation::CubicSpline => {
                    let prev = keyframes[step_start * 3 + 1];
                    let next = keyframes[step_end * 3 + 1];
                    let tangent_out_start = keyframes[step_start * 3 + 2];
                    let tangent_in_end = keyframes[step_end * 3];
                    (prev, next, tangent_out_start, tangent_in_end)
                }
            };

            bone_pose.scale = Some(match override_interpolation {
                Interpolation::Linear => prev.interpolate_linear(&next, lerp),
                Interpolation::Step => prev.interpolate_step(&next, lerp),
                Interpolation::CubicSpline => cubic_spline_interpolation(
                    prev,
                    tangent_out_start,
                    tangent_in_end,
                    next,
                    lerp,
                    duration,
                ),
            });
        }

        Keyframes::Weights(keyframes) => {
            let target_count = keyframes.len() / keyframe_count;
            let (prev, next, tangent_out_start, tangent_in_end) = match &curve.interpolation {
                Interpolation::Linear | Interpolation::Step => {
                    let prev: Vec<f32> = get_keyframe(target_count, keyframes, step_start).into();
                    let next: Vec<f32> = get_keyframe(target_count, keyframes, step_end).into();

                    let len = prev.len();

                    (prev, next, vec![0.; len], vec![0.; len])
                }
                Interpolation::CubicSpline => {
                    let prev: Vec<f32> =
                        get_keyframe(target_count, keyframes, step_start * 3 + 1).into();
                    let next: Vec<f32> =
                        get_keyframe(target_count, keyframes, step_end * 3 + 1).into();
                    let tangent_out_start: Vec<f32> =
                        get_keyframe(target_count, keyframes, step_start * 3 + 2).into();
                    let tangent_in_end: Vec<f32> =
                        get_keyframe(target_count, keyframes, step_end * 3).into();
                    (prev, next, tangent_out_start, tangent_in_end)
                }
            };

            bone_pose.weights = Some(match override_interpolation {
                Interpolation::Linear => prev.interpolate_linear(&next, lerp),
                Interpolation::Step => prev.interpolate_step(&next, lerp),
                Interpolation::CubicSpline => prev
                    .iter()
                    .zip(tangent_out_start)
                    .zip(tangent_in_end)
                    .zip(next)
                    .map(
                        |(((value_start, tangent_out_start), tangent_in_end), value_end)| {
                            cubic_spline_interpolation(
                                *value_start,
                                tangent_out_start,
                                tangent_in_end,
                                value_end,
                                lerp,
                                duration,
                            )
                        },
                    )
                    .collect(),
            });
        }
    }
}

fn sample_one_keyframe(
    step: usize,
    keyframe_count: usize,
    keyframes: &Keyframes,
    bone_pose: &mut BonePose,
) {
    match keyframes {
        Keyframes::Rotation(keyframes) => {
            let frame = keyframes[step];

            bone_pose.rotation = Some(frame);
        }
        Keyframes::Translation(keyframes) => {
            let frame = keyframes[step];
            bone_pose.translation = Some(frame);
        }

        Keyframes::Scale(keyframes) => {
            let frame = keyframes[step];
            bone_pose.scale = Some(frame);
        }

        Keyframes::Weights(keyframes) => {
            let target_count = keyframes.len() / keyframe_count;
            let morph_start: Vec<f32> = get_keyframe(target_count, keyframes, step).into();
            bone_pose.weights = Some(morph_start);
        }
    }
}

/// Helper function for cubic spline interpolation.
fn cubic_spline_interpolation<T>(
    value_start: T,
    tangent_out_start: T,
    tangent_in_end: T,
    value_end: T,
    lerp: f32,
    step_duration: f32,
) -> T
where
    T: Mul<f32, Output = T> + Add<Output = T>,
{
    value_start * (2.0 * lerp.powi(3) - 3.0 * lerp.powi(2) + 1.0)
        + tangent_out_start * (step_duration) * (lerp.powi(3) - 2.0 * lerp.powi(2) + lerp)
        + value_end * (-2.0 * lerp.powi(3) + 3.0 * lerp.powi(2))
        + tangent_in_end * step_duration * (lerp.powi(3) - lerp.powi(2))
}
