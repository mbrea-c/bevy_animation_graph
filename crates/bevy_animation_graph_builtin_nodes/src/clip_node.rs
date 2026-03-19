use std::any::TypeId;

use bevy::{
    asset::Handle,
    math::{Quat, Vec3},
    platform::hash::Hashed,
    prelude::{
        Animatable, AnimatableProperty, AnimationNodeIndex, EvaluatorId, Transform, VariableCurve,
    },
    reflect::prelude::*,
};
use bevy_animation_graph_core::{
    animation_clip::{EntityPath, GraphClip, Interpolation},
    animation_graph::TimeUpdate,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{
        DataSpec, DataValue,
        events::{AnimationEvent, EventQueue, SampledEvent},
    },
    errors::GraphError,
    event_track::sample_tracks,
    id::BoneId,
    pose::{BonePose, Pose, RootMotionDelta, RootMotionMode},
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct ClipNode {
    pub(crate) clip: Handle<GraphClip>,
    pub(crate) override_duration: Option<f32>,
    pub(crate) override_interpolation: Option<Interpolation>,
    /// Controls whether and how root motion is extracted from this clip.
    pub(crate) root_motion_mode: RootMotionMode,
    /// Which bone to use as the root motion source. If `None`, uses the skeleton's root bone.
    pub(crate) root_motion_bone: Option<EntityPath>,
}

impl ClipNode {
    pub const OUT_POSE: &'static str = "pose";
    pub const OUT_EVENT_QUEUE: &'static str = "events";

    pub fn new(
        clip: Handle<GraphClip>,
        override_duration: Option<f32>,
        override_interpolation: Option<Interpolation>,
    ) -> Self {
        Self {
            clip,
            override_duration,
            override_interpolation,
            root_motion_mode: RootMotionMode::Disabled,
            root_motion_bone: None,
        }
    }

    #[inline]
    pub fn clip_duration(&self, ctx: &NodeContext) -> Result<f32, GraphError> {
        if let Some(duration) = self.override_duration {
            Ok(duration)
        } else {
            ctx.graph_context
                .resources
                .graph_clip_assets
                .get(&self.clip)
                .ok_or(GraphError::ClipMissing)
                .map(|c| c.duration())
        }
    }

    pub fn update_time(&self, ctx: &NodeContext, input: &TimeUpdate) -> Result<f32, GraphError> {
        let prev_time = ctx.prev_time();

        ctx.graph_context
            .resources
            .graph_clip_assets
            .get(&self.clip)
            .ok_or(GraphError::ClipMissing)
            .and_then(|clip| {
                input
                    .partial_update_with_tracks(prev_time, &clip.event_tracks)
                    .ok_or(GraphError::TimeUpdateFailed)
            })
    }
}

impl NodeLike for ClipNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_duration_fwd(Some(self.clip_duration(&ctx)?));
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let clip_duration = self.clip_duration(&ctx)?;

        let Some(clip) = ctx
            .graph_context
            .resources
            .graph_clip_assets
            .get(&self.clip)
        else {
            // TODO: Should we propagate a GraphError instead?
            ctx.set_data_fwd(Self::OUT_POSE, DataValue::Pose(Pose::default()));
            return Ok(());
        };

        let time_update = ctx.time_update_fwd()?;
        let time = self.update_time(&ctx, &time_update)?;
        ctx.set_time(time);

        // Sample events
        let mut event_queue =
            EventQueue::with_events(sample_tracks(clip.event_tracks.values(), time));

        let mut out_pose = Pose {
            timestamp: time,
            skeleton: clip.skeleton.clone(),
            ..Pose::default()
        };

        if time > clip_duration {
            event_queue.add_event(SampledEvent::instant(AnimationEvent::AnimationClipFinished));
        }

        let clamped_time = time.clamp(0., clip_duration);

        for (bone_id, curves) in &clip.curves {
            let mut bone_pose = BonePose::default();
            for curve in curves {
                let value = sample_animation_curve(curve, clamped_time);
                match value {
                    CurveValue::Translation(t) => bone_pose.translation = Some(t),
                    CurveValue::Rotation(r) => bone_pose.rotation = Some(r),
                    CurveValue::Scale(s) => bone_pose.scale = Some(s),
                    CurveValue::BoneWeights(w) => bone_pose.weights = Some(w),
                }
            }
            out_pose.add_bone(bone_pose, BoneId::from(*bone_id));
        }

        // --- Root motion extraction ---
        if self.root_motion_mode != RootMotionMode::Disabled {
            let root_bone_id = self.root_motion_bone.as_ref().map(|p| p.id()).or_else(|| {
                ctx.graph_context
                    .resources
                    .skeleton_assets
                    .get(&clip.skeleton)
                    .map(|s| s.root())
            });

            if let Some(root_bone_id) = root_bone_id {
                let target_id = root_bone_id.animation_target_id();

                // Use BoneId derived from the AnimationTargetId for pose lookups,
                // since the pose was populated using BoneId::from(AnimationTargetId).
                let pose_bone_id = BoneId::from(target_id);

                // Sample root bone at current time (already done above)
                let current_bone = out_pose.get_bone(pose_bone_id);
                let current_translation = current_bone
                    .and_then(|b| b.translation)
                    .unwrap_or(Vec3::ZERO);
                let current_rotation = current_bone
                    .and_then(|b| b.rotation)
                    .unwrap_or(Quat::IDENTITY);

                // Helper: sample root bone translation/rotation at a given time
                let has_root_curves = clip.curves.contains_key(&target_id);
                let sample_root_at = |t: f32| -> (Vec3, Quat) {
                    let mut tr = Vec3::ZERO;
                    let mut rot = Quat::IDENTITY;
                    if let Some(curves) = clip.curves.get(&target_id) {
                        for curve in curves {
                            let value = sample_animation_curve(curve, t);
                            match value {
                                CurveValue::Translation(v) => tr = v,
                                CurveValue::Rotation(v) => rot = v,
                                _ => {}
                            }
                        }
                    }
                    (tr, rot)
                };

                if !has_root_curves {
                    bevy::log::warn_once!(
                        "Root motion: no animation curves found for root bone {:?} \
                         (target_id={:?}). The clip has {} bone entries.",
                        self.root_motion_bone,
                        target_id,
                        clip.curves.len()
                    );
                }

                let prev_time = ctx.prev_time();
                let clamped_prev_time = prev_time.clamp(0., clip_duration);

                // Compute delta, handling loop wraps correctly.
                // When the animation loops (current time < previous time), the root bone
                // position resets. We compute the delta in two parts:
                //   1. From prev_time to end of clip
                //   2. From start of clip to current_time
                let (mut delta_translation, mut delta_rotation) =
                    if clamped_time < clamped_prev_time - 0.001 {
                        // Loop wrap detected
                        let (end_tr, end_rot) = sample_root_at(clip_duration);
                        let (prev_tr, prev_rot) = sample_root_at(clamped_prev_time);
                        let (start_tr, start_rot) = sample_root_at(0.0);

                        // Delta from prev to end + delta from start to current
                        let dt1 = end_tr - prev_tr;
                        let dr1 = prev_rot.inverse() * end_rot;
                        let dt2 = current_translation - start_tr;
                        let dr2 = start_rot.inverse() * current_rotation;

                        (dt1 + dt2, dr1 * dr2)
                    } else {
                        // Normal (no wrap)
                        let (prev_tr, prev_rot) = sample_root_at(clamped_prev_time);
                        (
                            current_translation - prev_tr,
                            prev_rot.inverse() * current_rotation,
                        )
                    };

                // Get rest pose for zeroing
                let rest_local = ctx
                    .graph_context
                    .resources
                    .skeleton_assets
                    .get(&clip.skeleton)
                    .and_then(|s| s.default_transforms(root_bone_id))
                    .map(|dt| dt.local);
                let rest_translation = rest_local.map(|t| t.translation).unwrap_or(Vec3::ZERO);
                let rest_rotation = rest_local.map(|t| t.rotation).unwrap_or(Quat::IDENTITY);

                // Apply mode filtering and zero root bone in visual pose.
                match self.root_motion_mode {
                    RootMotionMode::Full => {
                        // Use full delta, zero root bone completely
                        if let Some(bone_idx) = out_pose.paths.get(&pose_bone_id).copied() {
                            out_pose.bones[bone_idx].translation = Some(rest_translation);
                            out_pose.bones[bone_idx].rotation = Some(rest_rotation);
                        } else {
                            bevy::log::warn!(
                                "Root motion: could not find root bone in pose for zeroing. \
                                 root_bone_id={:?}, pose has {} bones",
                                root_bone_id,
                                out_pose.paths.len()
                            );
                        }
                    }
                    RootMotionMode::GroundPlane => {
                        // Extract only XZ translation + Y rotation
                        // Keep Y translation and XZ rotation in visual pose
                        delta_translation.y = 0.0;

                        // Extract Y-axis rotation only
                        let (axis, angle) = delta_rotation.to_axis_angle();
                        let y_angle = angle * axis.y;
                        delta_rotation = Quat::from_rotation_y(y_angle);

                        // Zero only XZ translation in the visual pose.
                        // Keep Y (vertical bob) and full rotation (decomposing and
                        // removing only Y rotation cleanly is complex).
                        if let Some(bone_idx) = out_pose.paths.get(&pose_bone_id).copied()
                            && let Some(ref mut t) = out_pose.bones[bone_idx].translation
                        {
                            t.x = rest_translation.x;
                            t.z = rest_translation.z;
                        }
                    }
                    RootMotionMode::Disabled => unreachable!(),
                }

                out_pose.root_motion = Some(RootMotionDelta {
                    translation: delta_translation,
                    rotation: delta_rotation,
                });
            }
        }

        ctx.set_data_fwd(Self::OUT_EVENT_QUEUE, DataValue::EventQueue(event_queue));
        ctx.set_data_fwd(Self::OUT_POSE, DataValue::Pose(out_pose));

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_data(Self::OUT_EVENT_QUEUE, DataSpec::EventQueue)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "⏵ Animation Clip".into()
    }
}

#[allow(dead_code)]
enum CurveValue {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
    BoneWeights(Vec<f32>),
}

/// Sample the animation at a particular time
// HACK: We really need some API for sampling animation curves in Bevy outside of the builtin
// animation flow.
fn sample_animation_curve(curve: &VariableCurve, time: f32) -> CurveValue {
    let evaluator_id = curve.0.evaluator_id();
    let mut evaluator = curve.0.create_evaluator();

    let node_index = AnimationNodeIndex::default();

    let translation_evaluator_id = Hashed::new((TypeId::of::<Transform>(), 0));
    let rotation_evaluator_id = Hashed::new((TypeId::of::<Transform>(), 1));
    let scale_evaluator_id = Hashed::new((TypeId::of::<Transform>(), 2));

    curve
        .0
        .apply(evaluator.as_mut(), time, 1., node_index)
        .unwrap();

    match evaluator_id {
        EvaluatorId::ComponentField(id) => {
            // SAFETY: We only have a pointer to the evaluator, but we want to access private
            // fields. We're essentially telling the compiler: "Hey, operate on this block of
            // memory from somewhere else as if it had this type".
            // I would rather not do this, but we don't have sampling APIs yet.
            if id == &translation_evaluator_id {
                let animatable_evaluator: &AnimatableCurveEvaluator<Vec3> = unsafe {
                    std::mem::transmute(
                    evaluator
                        .downcast_ref::<bevy::animation::prelude::AnimatableCurveEvaluator<Vec3>>()
                        .unwrap(),
                 )
                };
                let value = animatable_evaluator.evaluator.stack[0].value;
                CurveValue::Translation(value)
            } else if id == &rotation_evaluator_id {
                let animatable_evaluator: &AnimatableCurveEvaluator<Quat> = unsafe {
                    std::mem::transmute(
                    evaluator
                        .downcast_ref::<bevy::animation::prelude::AnimatableCurveEvaluator<Quat>>()
                        .unwrap(),
                 )
                };
                let value = animatable_evaluator.evaluator.stack[0].value;
                CurveValue::Rotation(value)
            } else if id == &scale_evaluator_id {
                let animatable_evaluator: &AnimatableCurveEvaluator<Vec3> = unsafe {
                    std::mem::transmute(
                    evaluator
                        .downcast_ref::<bevy::animation::prelude::AnimatableCurveEvaluator<Vec3>>()
                        .unwrap(),
                 )
                };
                let value = animatable_evaluator.evaluator.stack[0].value;
                CurveValue::Scale(value)
            } else {
                todo!()
            }
        }
        EvaluatorId::Type(_id) => todo!(),
    }
}

// Why is this here?
//
// We need to access private fields in the evaluators in order to "extract" the
// sampled values. The evaluator trait no longer implements reflect, so our only option
// is to do a bit of unsafe memory shenanigans.
// We will transmute a reference of type `&bevy::animation::prelude::AnimatableCurveEvaluator` to
// `&AnimatableCurveEvaluator`, essentially telling the compiler "operate on the memory pointed to
// by this reference as if it had this custom type".
//
// We should aim to have a better animation curve sampling API in 0.16 in order to avoid having to
// do this.

pub struct AnimatableCurveEvaluator<A: Animatable> {
    evaluator: BasicAnimationCurveEvaluator<A>,
    _property: Box<dyn AnimatableProperty<Property = A>>,
}

struct BasicAnimationCurveEvaluator<A>
where
    A: Animatable,
{
    stack: Vec<BasicAnimationCurveEvaluatorStackElement<A>>,
    _blend_register: Option<(A, f32)>,
}

struct BasicAnimationCurveEvaluatorStackElement<A>
where
    A: Animatable,
{
    value: A,
    _weight: f32,
    _graph_node: AnimationNodeIndex,
}
