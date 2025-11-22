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
    animation_clip::{GraphClip, Interpolation},
    animation_graph::{PinMap, TimeUpdate},
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{DataSpec, DataValue, events::EventQueue},
    errors::GraphError,
    event_track::sample_tracks,
    id::BoneId,
    pose::{BonePose, Pose},
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct ClipNode {
    pub(crate) clip: Handle<GraphClip>,
    pub(crate) override_duration: Option<f32>,
    pub(crate) override_interpolation: Option<Interpolation>,
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

        // Sample events and publish
        let events = sample_tracks(clip.event_tracks.values(), time);
        ctx.set_data_fwd(
            Self::OUT_EVENT_QUEUE,
            DataValue::EventQueue(EventQueue::with_events(events)),
        );

        let mut out_pose = Pose {
            timestamp: time,
            skeleton: clip.skeleton.clone(),
            ..Pose::default()
        };

        let time = time.clamp(0., clip_duration);

        for (bone_id, curves) in &clip.curves {
            let mut bone_pose = BonePose::default();
            for curve in curves {
                let value = sample_animation_curve(curve, time);
                match value {
                    CurveValue::Translation(t) => bone_pose.translation = Some(t),
                    CurveValue::Rotation(r) => bone_pose.rotation = Some(r),
                    CurveValue::Scale(s) => bone_pose.scale = Some(s),
                    CurveValue::BoneWeights(w) => bone_pose.weights = Some(w),
                }
            }
            out_pose.add_bone(bone_pose, BoneId::from(*bone_id));
        }

        ctx.set_data_fwd(Self::OUT_POSE, DataValue::Pose(out_pose));

        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::OUT_POSE.into(), DataSpec::Pose),
            (Self::OUT_EVENT_QUEUE.into(), DataSpec::EventQueue),
        ]
        .into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
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
