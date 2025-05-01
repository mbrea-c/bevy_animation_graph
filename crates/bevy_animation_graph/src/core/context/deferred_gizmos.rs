use super::SystemResources;
use crate::core::{
    pose::{BoneId, Pose},
    skeleton::Skeleton,
    space_conversion::SpaceConversionContext,
};
use bevy::{
    color::LinearRgba,
    gizmos::gizmos::Gizmos,
    math::{Isometry3d, Quat, Vec3},
    platform::collections::HashMap,
    prelude::Entity,
    reflect::Reflect,
};

#[derive(Clone)]
pub struct DeferredGizmoRef {
    ptr: *mut DeferredGizmos,
}

impl From<&mut DeferredGizmos> for DeferredGizmoRef {
    fn from(value: &mut DeferredGizmos) -> Self {
        Self { ptr: value }
    }
}
impl DeferredGizmoRef {
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut DeferredGizmos {
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

#[derive(Clone, Reflect, Default)]
pub struct DeferredGizmos {
    commands: Vec<DeferredGizmoCommand>,
}

/// Enables debug drawing of bones, poses and other useful gizmos.
///
/// Keep in mind that this is highly inefficient (as it often requires
/// eagerly computing transform propagation), so it should only be used
/// for debugging purposes.
pub struct DeferredGizmosContext<'a> {
    pub gizmos: &'a mut DeferredGizmos,
    pub resources: &'a SystemResources<'a, 'a>,
    pub entity_map: &'a HashMap<BoneId, Entity>,
    pub space_conversion: SpaceConversionContext<'a>,
}

impl DeferredGizmos {
    pub fn apply(&mut self, gizmos: &mut Gizmos) {
        for command in self.commands.drain(..) {
            command.apply(gizmos);
        }
    }

    pub fn queue(&mut self, command: DeferredGizmoCommand) {
        self.commands.push(command);
    }

    pub fn sphere(&mut self, position: Vec3, rotation: Quat, radius: f32, color: LinearRgba) {
        self.commands.push(DeferredGizmoCommand::Sphere(
            position, rotation, radius, color,
        ));
    }

    pub fn ray(&mut self, origin: Vec3, direction: Vec3, color: LinearRgba) {
        self.commands
            .push(DeferredGizmoCommand::Ray(origin, direction, color));
    }
}

#[derive(Clone, Reflect)]
pub enum DeferredGizmoCommand {
    Sphere(Vec3, Quat, f32, LinearRgba),
    Ray(Vec3, Vec3, LinearRgba),
    Bone(Vec3, Vec3, LinearRgba),
}

impl DeferredGizmoCommand {
    pub fn apply(self, gizmos: &mut Gizmos) {
        match self {
            DeferredGizmoCommand::Sphere(position, rotation, radius, color) => {
                gizmos.sphere(
                    Isometry3d {
                        rotation,
                        translation: position.into(),
                    },
                    radius,
                    color,
                );
            }
            DeferredGizmoCommand::Ray(origin, direction, color) => {
                gizmos.ray(origin, direction, color);
            }
            DeferredGizmoCommand::Bone(start, end, color) => {
                bone_gizmo(gizmos, start, end, color);
            }
        }
    }
}

fn bone_gizmo(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: LinearRgba) {
    if start == end {
        return;
    }

    const BONE_CENTER_RATIO: f32 = 0.3;
    const BONE_RADIUS: f32 = 0.05;

    let start_to_end = end - start;
    let third_way = start + start_to_end * BONE_CENTER_RATIO;
    let (oba, obb) = start_to_end.normalize().any_orthonormal_pair();
    let a = third_way + oba * BONE_RADIUS;
    let b = third_way + obb * BONE_RADIUS;
    let c = third_way - oba * BONE_RADIUS;
    let d = third_way - obb * BONE_RADIUS;
    gizmos.line(start, a, color);
    gizmos.line(start, b, color);
    gizmos.line(start, c, color);
    gizmos.line(start, d, color);
    gizmos.line(a, b, color);
    gizmos.line(b, c, color);
    gizmos.line(c, d, color);
    gizmos.line(d, a, color);
    gizmos.line(a, end, color);
    gizmos.line(b, end, color);
    gizmos.line(c, end, color);
    gizmos.line(d, end, color);
}

impl DeferredGizmosContext<'_> {
    pub fn pose_bone_gizmos(&mut self, color: LinearRgba, pose: &Pose) {
        let Some(skeleton) = self.resources.skeleton_assets.get(&pose.skeleton) else {
            return;
        };

        for bone_id in pose.paths.keys() {
            self.bone_gizmo(*bone_id, color, skeleton, Some(pose));
        }
    }

    pub fn bone_gizmo(
        &mut self,
        bone_id: BoneId,
        color: LinearRgba,
        skeleton: &Skeleton,
        pose: Option<&Pose>,
    ) {
        let default_pose = Pose::default();
        let pose = pose.unwrap_or(&default_pose);

        let Some(parent_id) = skeleton.parent(&bone_id) else {
            return;
        };
        let global_bone_transform = self
            .space_conversion
            .global_transform_of_bone(pose, skeleton, bone_id);
        let parent_bone_transform = self
            .space_conversion
            .global_transform_of_bone(pose, skeleton, parent_id);
        self.gizmos.queue(DeferredGizmoCommand::Bone(
            parent_bone_transform.translation,
            global_bone_transform.translation,
            color,
        ));
    }

    pub fn bone_sphere(&mut self, bone_id: BoneId, radius: f32, color: LinearRgba) {
        let entity = self.entity_map.get(&bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.gizmos.queue(DeferredGizmoCommand::Sphere(
            global_transform.translation,
            global_transform.rotation,
            radius,
            color,
        ));
    }

    pub fn bone_rays(&mut self, bone_id: BoneId) {
        let entity = self.entity_map.get(&bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.gizmos.queue(DeferredGizmoCommand::Ray(
            global_transform.translation,
            global_transform.rotation * Vec3::X * 0.3,
            LinearRgba::RED,
        ));
        self.gizmos.queue(DeferredGizmoCommand::Ray(
            global_transform.translation,
            global_transform.rotation * Vec3::Y * 0.3,
            LinearRgba::GREEN,
        ));
        self.gizmos.queue(DeferredGizmoCommand::Ray(
            global_transform.translation,
            global_transform.rotation * Vec3::Z * 0.3,
            LinearRgba::BLUE,
        ));
    }

    pub fn sphere_in_parent_bone_space(
        &mut self,
        bone_id: BoneId,
        position: Vec3,
        rotation: Quat,
        radius: f32,
        color: LinearRgba,
        skeleton: &Skeleton,
    ) {
        let parent_bone_id = skeleton.parent(&bone_id).unwrap();
        let entity = self.entity_map.get(&parent_bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.gizmos.queue(DeferredGizmoCommand::Sphere(
            global_transform * position,
            global_transform.rotation * rotation,
            radius,
            color,
        ));
    }

    pub fn ray_in_parent_bone_space(
        &mut self,
        bone_id: BoneId,
        origin: Vec3,
        direction: Vec3,
        color: LinearRgba,
        skeleton: &Skeleton,
    ) {
        let parent_bone_id = skeleton.parent(&bone_id).unwrap();
        let entity = self.entity_map.get(&parent_bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.gizmos.queue(DeferredGizmoCommand::Ray(
            global_transform * origin,
            global_transform.rotation * direction,
            color,
        ));
    }
}
