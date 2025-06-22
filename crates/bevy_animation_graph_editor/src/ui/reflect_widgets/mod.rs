use std::{
    any::{Any, TypeId},
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use bevy::{
    app::App,
    asset::Handle,
    ecs::{reflect::AppTypeRegistry, resource::Resource},
    math::{Isometry3d, UVec3},
    platform::collections::HashMap,
    reflect::{PartialReflect, Reflect, Reflectable, TypeRegistry},
};
use bevy_animation_graph::{
    core::{
        animation_clip::EntityPath,
        colliders::core::{ColliderShape, SkeletonColliders},
        event_track::TrackItemValue,
        state_machine::high_level::StateMachine,
    },
    prelude::{AnimatedScene, AnimationGraph, GraphClip, config::PatternMapper},
};
use bevy_inspector_egui::{
    inspector_egui_impls::InspectorEguiImpl,
    reflect_inspector::{InspectorUi, ProjectorReflect},
    restricted_world_view::RestrictedWorldView,
};
use egui_dock::egui;

pub mod asset_picker;
pub mod checkbox;
pub mod entity_path;
pub mod pattern_mapper;
pub mod plugin;
pub mod submittable;
pub mod target_tracks;
pub mod vec2_plane;
pub mod wrap_ui;

pub trait EguiInspectorExtension: Sized {
    type Base: Reflectable + Clone + MakeBuffer<Self::Buffer> + Sized + Send + Sync + 'static;
    type Buffer: Default + Send + Sync + 'static;

    fn mutable(
        value: &mut Self::Base,
        buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool;

    fn readonly(
        value: &Self::Base,
        buffer: &Self::Buffer,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    );
}

pub trait EguiInspectorExtensionRegistration:
    EguiInspectorExtension + Sized + Send + Sync + 'static
where
    Self::Base: WidgetHash + std::fmt::Debug,
{
    fn register(self, app: &mut App) {
        app.register_type::<Self::Base>();
        // Working buffer
        app.insert_resource(EguiInspectorBuffers::<Self::Base, Self::Buffer>::default());
        // Top level buffer for non-dynamic
        app.insert_resource(
            EguiInspectorBuffers::<Self::Base, Self::Base, TopLevelBuffer>::default(),
        );
        let type_registry = app.world().resource::<AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        let type_registry = &mut type_registry;
        add_no_many::<Self::Base>(type_registry, Self::mutable_dyn, Self::readonly_dyn);
    }

    fn mutable_dyn(
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let value = value.downcast_mut::<Self::Base>().unwrap();
        let buffer = get_buffered::<Self::Base, Self::Buffer, ()>(
            env.context.world.as_mut().unwrap(),
            value,
            id,
        );

        Self::mutable(value, buffer, ui, options, id, env)
    }

    fn readonly_dyn(
        value: &dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let value = value.downcast_ref::<Self::Base>().unwrap();
        let buffer = get_buffered_readonly::<Self::Base, Self::Buffer, ()>(
            env.context.world.as_mut().unwrap(),
            value,
            id,
        );

        Self::readonly(value, buffer, ui, options, id, env)
    }
}

impl<T: EguiInspectorExtension + Sized + Send + Sync + 'static> EguiInspectorExtensionRegistration
    for T
where
    T::Base: WidgetHash + std::fmt::Debug,
{
}

pub trait MakeBuffer<B> {
    fn make_buffer(&self) -> B;
}

impl<T: Clone> MakeBuffer<T> for T {
    fn make_buffer(&self) -> T {
        self.clone()
    }
}

#[derive(Debug, Reflect)]
struct BufferField<B> {
    buffer: B,
    /// The hash of the original value this buffer is for.
    /// Used to detect when a value is changed "from under" the reflect editor UI,
    /// as the correct behaviour in that case is to reset the buffer
    start_hash: u64,
}

#[derive(Debug, Reflect)]
pub struct TopLevelBuffer;

#[derive(Resource, Debug, Reflect)]
struct EguiInspectorBuffers<S, B, M = ()> {
    bufs: HashMap<egui::Id, BufferField<B>>,
    /// marker for source type (which isn't stored but used to determine when to flush cache)
    _marker_s: std::marker::PhantomData<S>,
    /// The second marker is needed so we can have multiple resources for the same extension.
    /// In practice this will be used for a top-level buffer (used for `buffered_mut` calls) and a
    /// regular buffer accessible to UI code.
    _marker_m: std::marker::PhantomData<M>,
}

impl<S, B, M> Default for EguiInspectorBuffers<S, B, M> {
    fn default() -> Self {
        Self {
            bufs: HashMap::default(),
            _marker_s: std::marker::PhantomData,
            _marker_m: std::marker::PhantomData,
        }
    }
}

impl<S, B, M> EguiInspectorBuffers<S, B, M>
where
    S: WidgetHash + MakeBuffer<B>,
{
    /// If the original value was changed and we need to flush the buffer, flush it
    pub fn reset_if_needed(&mut self, value: &S, id: egui::Id) {
        if self.should_reset_field(value, id) {
            self.reset_field(value, id);
        }
    }

    fn should_reset_field(&self, value: &S, id: egui::Id) -> bool {
        if let Some(field_hash) = self.bufs.get(&id).map(|f| f.start_hash) {
            // First we need to compute the hash of value
            let hash = value.widget_hash();
            field_hash != hash
        } else {
            true
        }
    }

    fn reset_field(&mut self, value: &S, id: egui::Id) {
        self.bufs.insert(
            id,
            BufferField {
                buffer: value.make_buffer(),
                start_hash: value.widget_hash(),
            },
        );
    }
}

fn get_buffered<'w, S, B, M>(
    world: &mut RestrictedWorldView<'w>,
    value: &S,
    id: egui::Id,
) -> &'w mut B
where
    S: WidgetHash + MakeBuffer<B> + Send + Sync + 'static,
    B: Send + Sync + 'static,
    M: Send + Sync + 'static,
{
    let mut res = world
        .get_resource_mut::<EguiInspectorBuffers<S, B, M>>()
        .unwrap();
    res.reset_if_needed(value, id);
    // SAFETY: This is safe because the buffers are only accessed from the inspector
    //         which should only be accessed from one thread. Additionally, every
    //         item ID should only be accessed once, mutably. There are never multiple references
    //         to any buffer.
    unsafe { &mut *(&mut res.bufs.get_mut(&id).unwrap().buffer as *mut B) }
}

fn get_buffered_readonly<'w, S, B, M>(
    world: &mut RestrictedWorldView<'w>,
    value: &S,
    id: egui::Id,
) -> &'w B
where
    S: WidgetHash + MakeBuffer<B> + Send + Sync + 'static,
    B: Send + Sync + 'static,
    M: Send + Sync + 'static,
{
    let mut res = world
        .get_resource_mut::<EguiInspectorBuffers<S, B, M>>()
        .unwrap();
    res.reset_if_needed(value, id);
    // SAFETY: This is safe because the buffers are only accessed from the inspector
    //         which should only be accessed from one thread. Additionally, every
    //         item ID should only be accessed once, mutably. There are never multiple references
    //         to any buffer.
    // TODO: Avoid unsafe altogether by using an RC pointer or something like that
    unsafe { &*(&res.bufs.get(&id).unwrap().buffer as *const B) }
}

fn add_no_many<T: 'static>(
    type_registry: &mut TypeRegistry,
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::new(
            fn_mut,
            fn_readonly,
            many_unimplemented,
        ));
}

fn many_unimplemented(
    _ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn PartialReflect],
    _projector: &dyn ProjectorReflect,
) -> bool {
    false
}

type InspectorEguiImplFn =
    fn(&mut dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>) -> bool;
type InspectorEguiImplFnReadonly =
    fn(&dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>);

pub trait WidgetHash {
    fn widget_hash(&self) -> u64;
}

/// We need a macro since `impl <T: Hash> HashExt for T` does not allow
/// specialized impls
#[macro_export]
macro_rules! impl_widget_hash_from_hash {
    ( $($t:ty),* ) => {
    $( impl $crate::ui::reflect_widgets::WidgetHash for $t
    {
        fn widget_hash(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::hash::DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
        }
    }) *
    }
}

impl_widget_hash_from_hash! {
    PatternMapper,
    EntityPath,
    bool,
    TrackItemValue,
    Handle<AnimationGraph>,
    Handle<GraphClip>,
    Handle<StateMachine>,
    Handle<AnimatedScene>,
    Handle<SkeletonColliders>,
    String,
    PathBuf
}

impl WidgetHash for ColliderShape {
    fn widget_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        match self {
            ColliderShape::Sphere(sphere) => unsafe {
                std::mem::transmute::<_, u32>(sphere.radius).hash(&mut hasher);
            },
            ColliderShape::Capsule(capsule3d) => unsafe {
                std::mem::transmute::<_, u32>(capsule3d.half_length).hash(&mut hasher);
                std::mem::transmute::<_, u32>(capsule3d.radius).hash(&mut hasher);
            },
            ColliderShape::Cuboid(cuboid) => unsafe {
                std::mem::transmute::<_, UVec3>(cuboid.half_size).hash(&mut hasher);
            },
        }

        hasher.finish()
    }
}

impl WidgetHash for Isometry3d {
    fn widget_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        unsafe {
            std::mem::transmute::<_, u128>(self.translation).hash(&mut hasher);
            std::mem::transmute::<_, u128>(self.rotation).hash(&mut hasher);
        }

        hasher.finish()
    }
}
