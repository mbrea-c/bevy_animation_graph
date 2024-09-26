//! Provides the [`ReflectHandle`] type data, letting you get the [`TypeId`] of
//! the `T` in a [`Handle<T>`].
//!
//! This is used by [`reflect_de`], and the type data is only added for
//! [`Handle<T>`] types that we actually make use of in this crate. Ideally,
//! this [`ReflectHandle`] would be upstreamed to Bevy.
//!
//! [`reflect_de`]: crate::utils::reflect_de

use std::any::TypeId;

use bevy::{prelude::*, reflect::FromType};

use crate::{
    core::{skeleton::Skeleton, state_machine::high_level::StateMachine},
    prelude::{AnimatedScene, GraphClip},
};

#[derive(Debug)]
pub struct HandleReflectPlugin;

impl Plugin for HandleReflectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type_data::<Handle<AnimationGraph>, ReflectHandle>()
            .register_type_data::<Handle<Scene>, ReflectHandle>()
            .register_type_data::<Handle<Skeleton>, ReflectHandle>()
            .register_type_data::<Handle<AnimatedScene>, ReflectHandle>()
            .register_type_data::<Handle<GraphClip>, ReflectHandle>()
            .register_type_data::<Handle<StateMachine>, ReflectHandle>()
            .register_type_data::<Handle<Image>, ReflectHandle>();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReflectHandle {
    asset_type_id: TypeId,
}

impl ReflectHandle {
    #[must_use]
    pub const fn asset_type_id(&self) -> TypeId {
        self.asset_type_id
    }
}

impl<T: Asset> FromType<Handle<T>> for ReflectHandle {
    fn from_type() -> Self {
        Self {
            asset_type_id: TypeId::of::<T>(),
        }
    }
}
