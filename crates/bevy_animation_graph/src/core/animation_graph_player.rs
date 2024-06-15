use super::{
    animation_graph::{AnimationGraph, InputOverlay, TimeState, TimeUpdate},
    context::{BoneDebugGizmos, DeferredGizmos, PassContext},
    edge_data::DataValue,
    errors::GraphError,
    pose::{BoneId, Pose},
    prelude::GraphContextArena,
};
use crate::prelude::SystemResources;
use bevy::{
    asset::prelude::*, ecs::prelude::*, reflect::prelude::*, render::color::Color, utils::HashMap,
};

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationGraphPlayer {
    pub(crate) paused: bool,
    pub(crate) animation: Option<Handle<AnimationGraph>>,
    pub(crate) context_arena: Option<GraphContextArena>,
    pub(crate) elapsed: TimeState,
    pub(crate) pending_update: Option<TimeUpdate>,
    pub(crate) deferred_gizmos: DeferredGizmos,
    pub(crate) debug_draw_bones: Vec<BoneId>,
    pub(crate) entity_map: HashMap<BoneId, Entity>,

    input_overlay: InputOverlay,
    /// Error that ocurred during graph evaluation in the last frame
    #[reflect(ignore)]
    error: Option<GraphError>,
}

impl AnimationGraphPlayer {
    /// Create a new animation graph player, with no graph playing
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn get_context_arena(&self) -> Option<&GraphContextArena> {
        self.context_arena.as_ref()
    }

    /// Set the animation graph to play
    pub fn with_graph(mut self, animation: Handle<AnimationGraph>) -> Self {
        self.context_arena = Some(GraphContextArena::new(animation.id()));
        self.animation = Some(animation);
        self
    }

    /// Clear all input parameters for the animation graph
    pub fn clear_input_parameters(&mut self) {
        self.input_overlay.clear();
    }

    /// Configure an input parameter for the animation graph
    pub fn set_input_parameter(&mut self, parameter_name: impl Into<String>, value: DataValue) {
        self.input_overlay
            .parameters
            .insert(parameter_name.into(), value);
    }

    /// Return an input parameter for the animation graph
    pub fn get_input_parameter(&self, parameter_name: &str) -> Option<DataValue> {
        self.input_overlay.parameters.get(parameter_name).cloned()
    }

    /// Start playing an animation, resetting state of the player.
    /// This will use a linear blending between the previous and the new animation to make a smooth transition.
    pub fn start(&mut self, handle: Handle<AnimationGraph>) -> &mut Self {
        self.context_arena = Some(GraphContextArena::new(handle.id()));
        self.animation = Some(handle);
        self.elapsed = TimeState::default();
        self.paused = false;
        self
    }

    /// Query the animation graph with the latest time update and inputs
    pub(crate) fn query(
        &mut self,
        system_resources: &SystemResources,
        root_entity: Entity,
    ) -> Option<Pose> {
        let graph_handle = self.animation.as_ref()?;
        let graph = system_resources.animation_graph_assets.get(graph_handle)?;

        let pose = match graph.query_with_overlay(
            self.elapsed.update,
            self.context_arena.as_mut().unwrap(),
            system_resources,
            &self.input_overlay,
            root_entity,
            &self.entity_map,
            &mut self.deferred_gizmos,
        ) {
            Ok(pose) => {
                self.error = None;
                pose
            }
            Err(error) => {
                self.error = Some(error);
                Pose::default()
            }
        };

        Some(pose)
    }

    pub fn get_pass_context<'a>(
        &'a mut self,
        system_resources: &'a SystemResources,
        root_entity: Entity,
    ) -> PassContext<'a> {
        let context_arena = self.context_arena.as_mut().unwrap();

        PassContext::new(
            context_arena.get_toplevel_id(),
            context_arena,
            system_resources,
            &self.input_overlay,
            root_entity,
            &self.entity_map,
            &mut self.deferred_gizmos,
        )
    }

    pub fn gizmo_for_bones(&mut self, bones: impl IntoIterator<Item = BoneId>) {
        self.debug_draw_bones.extend(bones);
    }

    pub(crate) fn debug_draw_bones(
        &mut self,
        system_resources: &SystemResources,
        root_entity: Entity,
    ) {
        if self.debug_draw_bones.is_empty() {
            return;
        }

        let mut bones = std::mem::take(&mut self.debug_draw_bones);
        let mut ctx = self
            .get_pass_context(system_resources, root_entity)
            .with_debugging(true);
        for bone_id in bones.drain(..) {
            ctx.bone_gizmo(bone_id, Color::YELLOW, None);
        }
    }

    pub fn pause(&mut self) -> &mut Self {
        self.paused = true;
        self
    }

    pub fn resume(&mut self) -> &mut Self {
        self.paused = false;
        self
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn reset(&mut self) -> &mut Self {
        self.pending_update = Some(TimeUpdate::Absolute(0.));
        self
    }

    pub fn get_animation_graph(&self) -> Option<Handle<AnimationGraph>> {
        self.animation.clone()
    }

    /// If graph evaluation produced an error in the last frame return the error, otherwise return
    /// `None`.
    pub fn get_error(&self) -> Option<GraphError> {
        self.error.clone()
    }
}
