use super::PassContext;
use crate::core::pose::BoneId;
use bevy::{
    gizmos::gizmos::Gizmos,
    math::{Quat, Vec3},
    reflect::Reflect,
    render::color::Color,
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
    pub commands: Vec<DeferredGizmoCommand>,
}

impl DeferredGizmos {
    pub fn apply(&mut self, gizmos: &mut Gizmos) {
        for command in self.commands.drain(..) {
            command.apply(gizmos);
        }
    }

    pub fn sphere(&mut self, position: Vec3, rotation: Quat, radius: f32, color: Color) {
        self.commands.push(DeferredGizmoCommand::Sphere(
            position, rotation, radius, color,
        ));
    }

    pub fn ray(&mut self, origin: Vec3, direction: Vec3, color: Color) {
        self.commands
            .push(DeferredGizmoCommand::Ray(origin, direction, color));
    }
}

#[derive(Clone, Reflect)]
pub enum DeferredGizmoCommand {
    Sphere(Vec3, Quat, f32, Color),
    Ray(Vec3, Vec3, Color),
}

impl DeferredGizmoCommand {
    pub fn apply(self, gizmos: &mut Gizmos) {
        match self {
            DeferredGizmoCommand::Sphere(position, rotation, radius, color) => {
                gizmos.sphere(position, rotation, radius, color);
            }
            DeferredGizmoCommand::Ray(origin, direction, color) => {
                gizmos.ray(origin, direction, color);
            }
        }
    }
}

pub trait BoneDebugGizmos {
    fn bone_sphere(&mut self, bone_id: BoneId, radius: f32, color: Color);
    fn bone_rays(&mut self, bone_id: BoneId);
    fn sphere_in_parent_bone_space(
        &mut self,
        bone_id: BoneId,
        position: Vec3,
        rotation: Quat,
        radius: f32,
        color: Color,
    );
    fn ray_in_parent_bone_space(
        &mut self,
        bone_id: BoneId,
        origin: Vec3,
        direction: Vec3,
        color: Color,
    );
}

impl BoneDebugGizmos for PassContext<'_> {
    fn bone_sphere(&mut self, bone_id: BoneId, radius: f32, color: Color) {
        let entity = self.entity_map.get(&bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.deferred_gizmos.as_mut().sphere(
            global_transform.translation,
            global_transform.rotation,
            radius,
            color,
        );
    }

    fn bone_rays(&mut self, bone_id: BoneId) {
        let entity = self.entity_map.get(&bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.deferred_gizmos.as_mut().ray(
            global_transform.translation,
            global_transform.rotation * Vec3::X * 0.3,
            Color::RED,
        );
        self.deferred_gizmos.as_mut().ray(
            global_transform.translation,
            global_transform.rotation * Vec3::Y * 0.3,
            Color::GREEN,
        );
        self.deferred_gizmos.as_mut().ray(
            global_transform.translation,
            global_transform.rotation * Vec3::Z * 0.3,
            Color::BLUE,
        );
    }
    fn sphere_in_parent_bone_space(
        &mut self,
        bone_id: BoneId,
        position: Vec3,
        rotation: Quat,
        radius: f32,
        color: Color,
    ) {
        let parent_bone_id = bone_id.parent().unwrap();
        let entity = self.entity_map.get(&parent_bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.deferred_gizmos.as_mut().sphere(
            global_transform * position,
            global_transform.rotation * rotation,
            radius,
            color,
        );
    }

    fn ray_in_parent_bone_space(
        &mut self,
        bone_id: BoneId,
        origin: Vec3,
        direction: Vec3,
        color: Color,
    ) {
        let parent_bone_id = bone_id.parent().unwrap();
        let entity = self.entity_map.get(&parent_bone_id).unwrap();
        let global_transform = self
            .resources
            .transform_query
            .get(*entity)
            .unwrap()
            .1
            .compute_transform();
        self.deferred_gizmos.as_mut().ray(
            global_transform * origin,
            global_transform.rotation * direction,
            color,
        );
    }
}
