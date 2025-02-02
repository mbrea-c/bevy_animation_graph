use super::{
    animation_graph::{PinId, PinMap},
    edge_data::DataSpec,
    errors::GraphError,
};
use crate::{
    nodes::DummyNode,
    prelude::{PassContext, SpecContext},
};
use bevy::{
    prelude::{Deref, DerefMut},
    reflect::{prelude::*, FromType},
    utils::HashMap,
};
use std::{any::TypeId, fmt::Debug};

#[reflect_trait]
pub trait NodeLike: NodeLikeClone + Send + Sync + Debug + Reflect {
    fn duration(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn update(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        PinMap::new()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        PinMap::new()
    }

    fn time_input_spec(&self, _ctx: SpecContext) -> PinMap<()> {
        PinMap::new()
    }

    /// Specify whether or not a node outputs a pose, and which space the pose is in
    fn time_output_spec(&self, _ctx: SpecContext) -> Option<()> {
        None
    }

    /// The name of this node.
    fn display_name(&self) -> String;

    /// The order of the input pins. This way, you can mix time and data pins in the UI.
    fn input_pin_ordering(&self, _ctx: SpecContext) -> PinOrdering {
        PinOrdering::default()
    }

    /// The order of the output pins. This way, you can mix time and data pins in the UI.
    fn output_pin_ordering(&self, _ctx: SpecContext) -> PinOrdering {
        PinOrdering::default()
    }
}

pub trait NodeLikeClone {
    fn clone_node_like(&self) -> Box<dyn NodeLike>;
}

impl<T> NodeLikeClone for T
where
    T: 'static + NodeLike + Clone,
{
    fn clone_node_like(&self) -> Box<dyn NodeLike> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NodeLike> {
    fn clone(&self) -> Self {
        self.clone_node_like()
    }
}

#[derive(Clone)]
pub struct ReflectEditProxy {
    pub proxy_type_id: TypeId,
    pub from_proxy: fn(&dyn Reflect) -> Box<dyn NodeLike>,
    pub to_proxy: fn(&dyn NodeLike) -> Box<dyn Reflect>,
}

impl<T> FromType<T> for ReflectEditProxy
where
    T: EditProxy + NodeLike,
{
    fn from_type() -> Self {
        Self {
            proxy_type_id: TypeId::of::<<T as EditProxy>::Proxy>(),
            from_proxy: from_proxy::<T>,
            to_proxy: to_proxy::<T>,
        }
    }
}

fn from_proxy<T: EditProxy + NodeLike>(proxy: &dyn Reflect) -> Box<dyn NodeLike> {
    if proxy.type_id() == TypeId::of::<T::Proxy>() {
        let proxy = proxy.downcast_ref::<T::Proxy>().unwrap();
        Box::new(T::update_from_proxy(proxy))
    } else {
        panic!("Type mismatch")
    }
}

fn to_proxy<T: EditProxy + NodeLike>(node: &dyn NodeLike) -> Box<dyn Reflect> {
    if node.type_id() == TypeId::of::<T>() {
        let node = node.as_any().downcast_ref::<T>().unwrap();
        Box::new(T::make_proxy(node))
    } else {
        panic!("Type mismatch")
    }
}

pub trait EditProxy {
    type Proxy: Reflect;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self;
    fn make_proxy(&self) -> Self::Proxy;
}

#[derive(Clone, Reflect, Debug, Default)]
pub struct PinOrdering {
    keys: HashMap<PinId, i32>,
}

impl PinOrdering {
    pub fn new(keys: impl Into<HashMap<PinId, i32>>) -> Self {
        Self { keys: keys.into() }
    }

    pub fn pin_key(&self, pin_id: &PinId) -> i32 {
        self.keys.get(pin_id).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
// #[derive(Reflect)] // manual reflect impl (see below)
pub struct AnimationNode {
    pub name: String,
    #[deref]
    pub inner: Box<dyn NodeLike>,
    // #[reflect(ignore)] // manual reflect impl (see below)
    pub should_debug: bool,
}

impl AnimationNode {
    #[must_use]
    pub fn new(name: impl Into<String>, inner: Box<dyn NodeLike>) -> Self {
        Self {
            name: name.into(),
            inner,
            should_debug: false,
        }
    }
}

impl Default for AnimationNode {
    fn default() -> Self {
        Self::new("", Box::new(DummyNode))
    }
}

const _: () = {
    #[allow(unused_mut)]
    impl bevy::reflect::GetTypeRegistration for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
        String: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        /* manual reflect impl
        Box<dyn NodeLike>: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        */
    {
        fn get_type_registration() -> bevy::reflect::TypeRegistration {
            let mut registration = bevy::reflect::TypeRegistration::of::<Self>();
            registration.insert::<bevy::reflect::ReflectFromPtr>(
                bevy::reflect::FromType::<Self>::from_type(),
            );
            registration.insert::<bevy::reflect::ReflectFromReflect>(
                bevy::reflect::FromType::<Self>::from_type(),
            );
            registration
        }
        #[inline(never)]
        fn register_type_dependencies(registry: &mut bevy::reflect::TypeRegistry) {
            <String as bevy::reflect::__macro_exports::RegisterForReflection>::__register(registry);
            // <Box<dyn NodeLike>as bevy::reflect::__macro_exports::RegisterForReflection> ::__register(registry); // manual reflect impl
        }
    }
    impl bevy::reflect::Typed for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
        String: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        /* manual reflect impl
        Box<dyn NodeLike>: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        */
    {
        #[inline]
        fn type_info() -> &'static bevy::reflect::TypeInfo {
            static CELL: bevy::reflect::utility::NonGenericTypeInfoCell =
                bevy::reflect::utility::NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| {
                bevy::reflect::TypeInfo::Struct(
                    bevy::reflect::StructInfo::new::<Self>(&[
                        bevy::reflect::NamedField::new::<String>("name").with_custom_attributes(
                            bevy::reflect::attributes::CustomAttributes::default(),
                        ),
                        /* manual reflect impl
                        bevy::reflect::NamedField::new::<Box<dyn NodeLike>>("inner")
                            .with_custom_attributes(
                                bevy::reflect::attributes::CustomAttributes::default(),
                            ),
                        */
                    ])
                    .with_custom_attributes(bevy::reflect::attributes::CustomAttributes::default()),
                )
            })
        }
    }
    impl bevy::reflect::TypePath for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
    {
        fn type_path() -> &'static str {
            ::core::concat!(
                ::core::concat!(::core::module_path!(), "::"),
                "AnimationNode"
            )
        }
        fn short_type_path() -> &'static str {
            "AnimationNode"
        }
        fn type_ident() -> Option<&'static str> {
            ::core::option::Option::Some("AnimationNode")
        }
        fn crate_name() -> Option<&'static str> {
            ::core::option::Option::Some(::core::module_path!().split(':').next().unwrap())
        }
        fn module_path() -> Option<&'static str> {
            ::core::option::Option::Some(::core::module_path!())
        }
    }
    impl bevy::reflect::Reflect for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
        String: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        /* manual reflect impl
        Box<dyn NodeLike>: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        */
    {
        #[inline]
        fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn ::core::any::Any> {
            self
        }
        #[inline]
        fn as_any(&self) -> &dyn ::core::any::Any {
            self
        }
        #[inline]
        fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {
            self
        }
        #[inline]
        fn into_reflect(
            self: ::std::boxed::Box<Self>,
        ) -> ::std::boxed::Box<dyn bevy::reflect::Reflect> {
            self
        }
        #[inline]
        fn as_reflect(&self) -> &dyn bevy::reflect::Reflect {
            self
        }
        #[inline]
        fn as_reflect_mut(&mut self) -> &mut dyn bevy::reflect::Reflect {
            self
        }
        #[inline]
        fn set(
            &mut self,
            value: ::std::boxed::Box<dyn bevy::reflect::Reflect>,
        ) -> ::core::result::Result<(), ::std::boxed::Box<dyn bevy::reflect::Reflect>> {
            *self = <dyn bevy::reflect::Reflect>::take(value)?;
            ::core::result::Result::Ok(())
        }
    }
    impl bevy::reflect::Struct for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
        String: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        /* manual reflect impl
        Box<dyn NodeLike>: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        */
    {
        fn field(&self, name: &str) -> ::core::option::Option<&dyn bevy::reflect::PartialReflect> {
            match name {
                "name" => ::core::option::Option::Some(&self.name),
                // manual reflect impl
                "inner" => ::core::option::Option::Some(self.inner.as_partial_reflect()),
                _ => ::core::option::Option::None,
            }
        }
        fn field_mut(
            &mut self,
            name: &str,
        ) -> ::core::option::Option<&mut dyn bevy::reflect::PartialReflect> {
            match name {
                "name" => ::core::option::Option::Some(&mut self.name),
                // manual reflect impl
                "inner" => ::core::option::Option::Some(self.inner.as_partial_reflect_mut()),
                _ => ::core::option::Option::None,
            }
        }
        fn field_at(
            &self,
            index: usize,
        ) -> ::core::option::Option<&dyn bevy::reflect::PartialReflect> {
            match index {
                0usize => ::core::option::Option::Some(&self.name),
                // manual reflect impl
                1usize => ::core::option::Option::Some(self.inner.as_partial_reflect()),
                _ => ::core::option::Option::None,
            }
        }
        fn field_at_mut(
            &mut self,
            index: usize,
        ) -> ::core::option::Option<&mut dyn bevy::reflect::PartialReflect> {
            match index {
                0usize => ::core::option::Option::Some(&mut self.name),
                // manual reflect impl
                1usize => ::core::option::Option::Some(self.inner.as_partial_reflect_mut()),
                _ => ::core::option::Option::None,
            }
        }
        fn name_at(&self, index: usize) -> ::core::option::Option<&str> {
            match index {
                0usize => ::core::option::Option::Some("name"),
                1usize => ::core::option::Option::Some("inner"),
                _ => ::core::option::Option::None,
            }
        }
        fn field_len(&self) -> usize {
            2usize
        }
        fn iter_fields(&self) -> bevy::reflect::FieldIter {
            bevy::reflect::FieldIter::new(self)
        }
        fn clone_dynamic(&self) -> bevy::reflect::DynamicStruct {
            let mut dynamic: bevy::reflect::DynamicStruct = ::core::default::Default::default();
            dynamic.set_represented_type(bevy::reflect::PartialReflect::get_represented_type_info(
                self,
            ));
            dynamic.insert_boxed(
                "name",
                bevy::reflect::PartialReflect::clone_value(&self.name),
            );
            dynamic.insert_boxed(
                "inner",
                // manual reflect impl
                bevy::reflect::PartialReflect::clone_value(&*self.inner),
            );
            dynamic
        }
    }
    impl bevy::reflect::PartialReflect for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
        String: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        /* manual reflect impl
        Box<dyn NodeLike>: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        */
    {
        #[inline]
        fn get_represented_type_info(
            &self,
        ) -> ::core::option::Option<&'static bevy::reflect::TypeInfo> {
            ::core::option::Option::Some(<Self as bevy::reflect::Typed>::type_info())
        }
        #[inline]
        fn clone_value(&self) -> ::std::boxed::Box<dyn bevy::reflect::PartialReflect> {
            ::std::boxed::Box::new(bevy::reflect::Struct::clone_dynamic(self))
        }
        #[inline]
        fn try_apply(
            &mut self,
            value: &dyn bevy::reflect::PartialReflect,
        ) -> ::core::result::Result<(), bevy::reflect::ApplyError> {
            if let bevy::reflect::ReflectRef::Struct(struct_value) =
                bevy::reflect::PartialReflect::reflect_ref(value)
            {
                for (i, value) in ::core::iter::Iterator::enumerate(
                    bevy::reflect::Struct::iter_fields(struct_value),
                ) {
                    let name = bevy::reflect::Struct::name_at(struct_value, i).unwrap();
                    if let ::core::option::Option::Some(v) =
                        bevy::reflect::Struct::field_mut(self, name)
                    {
                        bevy::reflect::PartialReflect::try_apply(v, value)?;
                    }
                }
            } else {
                return ::core::result::Result::Err(bevy::reflect::ApplyError::MismatchedKinds {
                    from_kind: bevy::reflect::PartialReflect::reflect_kind(value),
                    to_kind: bevy::reflect::ReflectKind::Struct,
                });
            }
            ::core::result::Result::Ok(())
        }
        #[inline]
        fn reflect_kind(&self) -> bevy::reflect::ReflectKind {
            bevy::reflect::ReflectKind::Struct
        }
        #[inline]
        fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
            bevy::reflect::ReflectRef::Struct(self)
        }
        #[inline]
        fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
            bevy::reflect::ReflectMut::Struct(self)
        }
        #[inline]
        fn reflect_owned(self: ::std::boxed::Box<Self>) -> bevy::reflect::ReflectOwned {
            bevy::reflect::ReflectOwned::Struct(self)
        }
        #[inline]
        fn try_into_reflect(
            self: ::std::boxed::Box<Self>,
        ) -> ::core::result::Result<
            ::std::boxed::Box<dyn bevy::reflect::Reflect>,
            ::std::boxed::Box<dyn bevy::reflect::PartialReflect>,
        > {
            ::core::result::Result::Ok(self)
        }
        #[inline]
        fn try_as_reflect(&self) -> ::core::option::Option<&dyn bevy::reflect::Reflect> {
            ::core::option::Option::Some(self)
        }
        #[inline]
        fn try_as_reflect_mut(
            &mut self,
        ) -> ::core::option::Option<&mut dyn bevy::reflect::Reflect> {
            ::core::option::Option::Some(self)
        }
        #[inline]
        fn into_partial_reflect(
            self: ::std::boxed::Box<Self>,
        ) -> ::std::boxed::Box<dyn bevy::reflect::PartialReflect> {
            self
        }
        #[inline]
        fn as_partial_reflect(&self) -> &dyn bevy::reflect::PartialReflect {
            self
        }
        #[inline]
        fn as_partial_reflect_mut(&mut self) -> &mut dyn bevy::reflect::PartialReflect {
            self
        }
        fn reflect_partial_eq(
            &self,
            value: &dyn bevy::reflect::PartialReflect,
        ) -> ::core::option::Option<bool> {
            (bevy::reflect::struct_partial_eq)(self, value)
        }
    }
    impl bevy::reflect::FromReflect for AnimationNode
    where
        AnimationNode: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
        String: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        /* manual reflect impl
        Box<dyn NodeLike>: bevy::reflect::FromReflect
            + bevy::reflect::TypePath
            + bevy::reflect::MaybeTyped
            + bevy::reflect::__macro_exports::RegisterForReflection,
        */
    {
        fn from_reflect(
            reflect: &dyn bevy::reflect::PartialReflect,
        ) -> ::core::option::Option<Self> {
            // manual reflect impl start
            Some(reflect.try_downcast_ref::<Self>()?.clone())
            /*
            if let bevy::reflect::ReflectRef::Struct(__ref_struct) =
                bevy::reflect::PartialReflect::reflect_ref(reflect)
            {
                let __this = Self {
                    name: (|| {
                        <String as bevy::reflect::FromReflect>::from_reflect(
                            bevy::reflect::Struct::field(__ref_struct, "name")?,
                        )
                    })()?,
                    inner: (|| {
                        <Box<dyn NodeLike> as bevy::reflect::FromReflect>::from_reflect(
                            bevy::reflect::Struct::field(__ref_struct, "inner")?,
                        )
                    })()?,
                    should_debug: ::core::default::Default::default(),
                };
                ::core::option::Option::Some(__this)
            } else {
                ::core::option::Option::None
            } */
            // manual reflect impl end
        }
    }
};
