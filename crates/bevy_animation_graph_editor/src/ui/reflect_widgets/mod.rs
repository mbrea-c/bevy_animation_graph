use std::{
    any::{Any, TypeId},
    hash::{DefaultHasher, Hash, Hasher},
};

use bevy::{
    app::App,
    ecs::{reflect::AppTypeRegistry, system::Resource},
    reflect::{PartialReflect, TypeRegistry},
    utils::HashMap,
};
use bevy_animation_graph::{core::animation_clip::EntityPath, prelude::config::PatternMapper};
use bevy_inspector_egui::{
    inspector_egui_impls::InspectorEguiImpl,
    reflect_inspector::{InspectorUi, ProjectorReflect},
    restricted_world_view::RestrictedWorldView,
};
use egui_dock::egui;

pub mod checkbox;
pub mod core;
pub mod entity_path;
pub mod pattern_mapper;
pub mod plugin;

pub trait EguiInspectorExtension {
    type Base: Clone + Sized + Send + Sync + 'static;
    type Buffer: Default + Send + Sync + 'static;

    fn mutable(
        value: &mut Self::Base,
        buffer: Option<&mut Self::Buffer>,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool;

    fn readonly(
        value: &Self::Base,
        buffer: Option<&Self::Buffer>,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    );

    fn init_buffer(#[allow(unused_variables)] value: &Self::Base) -> Option<Self::Buffer> {
        None
    }

    fn needs_buffer() -> bool {
        false
    }
}

pub trait EguiInspectorExtensionRegistration:
    EguiInspectorExtension + Sized + Send + Sync + 'static
where
    Self::Base: HashExt,
{
    fn register(self, app: &mut App) {
        if Self::needs_buffer() {
            app.insert_resource(EguiInspectorBuffers::<Self>::default());
        }
        // Top level buffer for non-dynamic
        app.insert_resource(EguiInspectorBuffers::<Self, TopLevelBuffer>::default());
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
        let buffer = Self::needs_buffer()
            .then(|| get_buffered::<Self, ()>(env.context.world.as_mut().unwrap(), value, id));

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
        let buffer = Self::needs_buffer().then(|| {
            get_buffered_readonly::<Self, ()>(env.context.world.as_mut().unwrap(), value, id)
        });

        Self::readonly(value, buffer, ui, options, id, env)
    }
}

impl<T: EguiInspectorExtension + Sized + Send + Sync + 'static> EguiInspectorExtensionRegistration
    for T
where
    T::Base: HashExt,
{
}

pub trait HashExt {
    fn hash_ext(&self) -> u64;
}

struct BufferField<B> {
    buffer: B,
    /// The hash of the original value this buffer is for.
    /// Used to detect when a value is changed "from under" the reflect editor UI,
    /// as the correct behaviour in that case is to reset the buffer
    start_hash: u64,
}

pub struct TopLevelBuffer;

#[derive(Resource)]
struct EguiInspectorBuffers<E: EguiInspectorExtension, M = ()> {
    bufs: HashMap<egui::Id, BufferField<E::Buffer>>,
    _marker_e: std::marker::PhantomData<E>,
    /// The second marker is needed so we can have multiple resources for the same extension.
    /// In practice this will be used for a top-level buffer (used for `buffered_mut` calls) and a
    /// regular buffer accessible to UI code.
    _marker_m: std::marker::PhantomData<M>,
}

impl<E: EguiInspectorExtension, M> Default for EguiInspectorBuffers<E, M> {
    fn default() -> Self {
        Self {
            bufs: HashMap::default(),
            _marker_e: std::marker::PhantomData,
            _marker_m: std::marker::PhantomData,
        }
    }
}

impl<E: EguiInspectorExtension, M> EguiInspectorBuffers<E, M>
where
    E::Base: HashExt,
{
    /// If the original value was changed and we need to flush the buffer, flush it
    pub fn reset_if_needed(&mut self, value: &E::Base, id: egui::Id) {
        if self.should_reset_field(value, id) {
            self.reset_field(value, id);
        }
    }

    fn should_reset_field(&self, value: &E::Base, id: egui::Id) -> bool {
        if let Some(field_hash) = self.bufs.get(&id).map(|f| f.start_hash) {
            // First we need to compute the hash of value
            let hash = value.hash_ext();
            field_hash != hash
        } else {
            true
        }
    }

    fn reset_field(&mut self, value: &E::Base, id: egui::Id) {
        self.bufs.insert(
            id,
            BufferField {
                buffer: E::init_buffer(value).unwrap(),
                start_hash: value.hash_ext(),
            },
        );
    }
}

fn get_buffered<'w, E: EguiInspectorExtension, M>(
    world: &mut RestrictedWorldView<'w>,
    value: &E::Base,
    id: egui::Id,
) -> &'w mut E::Buffer
where
    E: Send + Sync + 'static,
    E::Base: HashExt + Send + Sync + Clone + 'static,
    E::Buffer: Send + Sync + 'static,
    M: Send + Sync + 'static,
{
    let mut res = world
        .get_resource_mut::<EguiInspectorBuffers<E, M>>()
        .unwrap();
    res.reset_if_needed(value, id);
    // SAFETY: This is safe because the buffers are only accessed from the inspector
    //         which should only be accessed from one thread. Additionally, every
    //         item ID should only be accessed once, mutably. There are never multiple references
    //         to any buffer.
    unsafe { &mut *(&mut res.bufs.get_mut(&id).unwrap().buffer as *mut E::Buffer) }
}

fn get_buffered_readonly<'w, E: EguiInspectorExtension, M>(
    world: &mut RestrictedWorldView<'w>,
    value: &E::Base,
    id: egui::Id,
) -> &'w E::Buffer
where
    E: Send + Sync + 'static,
    E::Base: HashExt + Send + Sync + Clone + 'static,
    E::Buffer: Send + Sync + 'static,
    M: Send + Sync + 'static,
{
    let mut res = world
        .get_resource_mut::<EguiInspectorBuffers<E, M>>()
        .unwrap();
    res.reset_if_needed(value, id);
    // SAFETY: This is safe because the buffers are only accessed from the inspector
    //         which should only be accessed from one thread. Additionally, every
    //         item ID should only be accessed once, mutably. There are never multiple references
    //         to any buffer.
    // TODO: Avoid unsafe altogether by using an RC pointer or something like that
    unsafe { &*(&res.bufs.get(&id).unwrap().buffer as *const E::Buffer) }
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

/// We need a macro since `impl <T: Hash> HashExt for T` does not allow
/// specialized impls
#[macro_export]
macro_rules! impl_hash_ext_from_hash {
    ( $($t:ty),* ) => {
    $( impl HashExt for $t
    {
        fn hash_ext(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
        }
    }) *
    }
}

impl_hash_ext_from_hash! { PatternMapper, EntityPath, bool }
