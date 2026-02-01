use bevy::{
    color::LinearRgba,
    math::{Quat, Vec3},
    reflect::{Reflect, std_traits::ReflectDefault},
    transform::components::Transform,
};
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct TwoBoneIKNode;

impl TwoBoneIKNode {
    pub const IN_TIME: &'static str = "time";
    pub const IN_POSE: &'static str = "pose";
    pub const OUT_POSE: &'static str = "pose";
    pub const TARGETBONE: &'static str = "target_path";
    pub const TARGETPOS: &'static str = "target_position";

    pub fn new() -> Self {
        Self {}
    }
}

impl NodeLike for TwoBoneIKNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME, input);
        let target = ctx.data_back(Self::TARGETBONE)?.into_entity_path()?;
        let target = target.id();
        let target_pos_char = ctx.data_back(Self::TARGETPOS)?.into_vec3()?;
        //let targetrotation: Quat = ctx.parameter_back(Self::TARGETROT).unwrap();
        let mut pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;
        let Some(skeleton) = ctx
            .graph_context
            .resources
            .skeleton_assets
            .get(&pose.skeleton)
        else {
            return Err(GraphError::SkeletonMissing(ctx.node_id));
        };

        if let (Some(bone_id), Some(parent_path), Some(grandparent_path)) = (
            pose.paths.get(&target),
            skeleton.parent(&target),
            skeleton.parent(&target).and_then(|p| skeleton.parent(&p)),
        ) {
            // Debug render (if enabled)
            ctx.graph_context.use_debug_gizmos(|mut gizmos| {
                gizmos.bone_gizmo(target, LinearRgba::RED, false, skeleton, Some(&pose))
            });
            ctx.graph_context.use_debug_gizmos(|mut gizmos| {
                gizmos.bone_gizmo(parent_path, LinearRgba::RED, false, skeleton, Some(&pose))
            });

            let bone = pose.bones[*bone_id].clone();
            let target_gp = ctx.graph_context.space_conversion().root_to_bone_space(
                Transform::from_translation(target_pos_char),
                &pose,
                skeleton,
                skeleton.parent(&grandparent_path).unwrap(),
            );

            let target_pos_gp = target_gp.translation;

            let parent_id = pose.paths.get(&parent_path).unwrap();
            let parent_transform = {
                let parent_bone = pose.bones.get_mut(*parent_id).unwrap();
                parent_bone.to_transform()
            };

            let grandparent_id = pose.paths.get(&grandparent_path).unwrap();
            let grandparent_bone = pose.bones.get_mut(*grandparent_id).unwrap();
            let grandparent_transform = grandparent_bone.to_transform();

            let bone_transform = bone.to_transform();

            let parent_gp_transform = grandparent_transform * parent_transform;
            let bone_gp_transform = parent_gp_transform * bone_transform;

            let (bone_gp_transform, parent_gp_transform, grandparent_transform) = two_bone_ik(
                bone_gp_transform,
                parent_gp_transform,
                grandparent_transform,
                target_pos_gp,
            );

            let parent_transform =
                Transform::from_matrix(grandparent_transform.to_matrix().inverse())
                    * parent_gp_transform;
            let bone_transform = Transform::from_matrix(parent_gp_transform.to_matrix().inverse())
                * bone_gp_transform;

            pose.bones[*grandparent_id].rotation = Some(grandparent_transform.rotation);
            pose.bones[*parent_id].rotation = Some(parent_transform.rotation);
            pose.bones[*bone_id].rotation = Some(bone_transform.rotation);

            // Debug render (if enabled)
            ctx.graph_context.use_debug_gizmos(|mut gizmos| {
                gizmos.bone_gizmo(target, LinearRgba::BLUE, false, skeleton, Some(&pose))
            });
            ctx.graph_context.use_debug_gizmos(|mut gizmos| {
                gizmos.bone_gizmo(parent_path, LinearRgba::BLUE, false, skeleton, Some(&pose))
            });
        }
        ctx.set_time(pose.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, pose);
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::TARGETBONE, DataSpec::EntityPath)
            .add_input_data(Self::TARGETPOS, DataSpec::Vec3)
            .add_input_data(Self::IN_POSE, DataSpec::Pose)
            .add_input_time(Self::IN_TIME);
        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "Two Bone IK".into()
    }
}

// Adapted from https://blog.littlepolygon.com/posts/twobone/
fn two_bone_ik(
    bone: Transform,
    parent: Transform,
    grandparent: Transform,
    target_pos: Vec3,
) -> (Transform, Transform, Transform) {
    const MAX_LEN_OFFSET: f32 = 0.01;

    // compute joint positions
    let in_end_loc = bone.translation;
    let in_mid_loc = parent.translation;
    let in_root_loc = grandparent.translation;

    // compute bone lengths
    let upper_len = in_root_loc.distance(in_mid_loc);
    let lower_len = in_mid_loc.distance(in_end_loc);
    let max_len = upper_len + lower_len - MAX_LEN_OFFSET;

    // compute input planar basis vectors
    let to_end = (in_end_loc - in_root_loc).normalize();
    let in_pole_vec = (in_mid_loc - in_root_loc).reject_from(to_end).normalize();

    // compute final planar basis vectors
    let to_target_offset = (target_pos - in_root_loc).clamp_length_max(max_len);
    let to_target_dist = to_target_offset.length();
    let to_target = to_target_offset / to_target_dist;

    let to_target_swing = Quat::from_rotation_arc(to_end, to_target);
    let out_pole_vec = to_target_swing * in_pole_vec;

    // apply law of cosines to get middle joint angle
    let denom = 2. * upper_len * to_target_dist;
    let mut cos_angle = 0.;
    if denom > f32::EPSILON {
        cos_angle = (to_target_dist * to_target_dist + upper_len * upper_len
            - lower_len * lower_len)
            / denom;
    }
    let angle = cos_angle.acos();

    // compute final joint positions
    let pole_dist = upper_len * angle.sin();
    let eff_dist = upper_len * cos_angle;
    let out_end_loc = in_root_loc + to_target_offset;
    let out_mid_loc = in_root_loc + eff_dist * to_target + pole_dist * out_pole_vec;

    // compute final rotations
    let in_to_mid = in_mid_loc - in_root_loc;
    let out_to_mid = out_mid_loc - in_root_loc;
    let root_swing = Quat::from_rotation_arc(in_to_mid.normalize(), out_to_mid.normalize());
    let in_end_loc_with_root_swing = in_root_loc + root_swing * (in_end_loc - in_root_loc);
    let to_in_end = in_end_loc_with_root_swing - out_mid_loc;
    let to_out_end = out_end_loc - out_mid_loc;
    let mid_swing =
        Quat::from_rotation_arc(to_in_end.normalize(), to_out_end.normalize()) * root_swing;

    // set up output transforms
    let out_grandparent = Transform {
        rotation: root_swing * grandparent.rotation,
        ..grandparent
    };

    let out_parent = Transform {
        translation: out_mid_loc,
        rotation: mid_swing * parent.rotation,
        ..parent
    };
    let out_bone = Transform {
        translation: out_end_loc,
        rotation: mid_swing * bone.rotation,
        ..bone
    };

    (out_bone, out_parent, out_grandparent)
}
