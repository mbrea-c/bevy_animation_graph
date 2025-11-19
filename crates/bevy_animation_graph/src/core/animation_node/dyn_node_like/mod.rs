pub mod serial;

use bevy::prelude::{Deref, DerefMut};

use crate::core::animation_node::NodeLike;

#[derive(Debug, Deref, DerefMut)]
pub struct DynNodeLike(#[deref] pub(crate) Box<dyn NodeLike>);

impl DynNodeLike {
    pub fn new(value: impl NodeLike) -> Self {
        Self(Box::new(value))
    }

    pub fn new_boxed(value: Box<dyn NodeLike>) -> Self {
        Self(value)
    }
}

impl Clone for DynNodeLike {
    fn clone(&self) -> Self {
        Self(self.0.clone_node_like())
    }
}

impl ::bevy::reflect::GetTypeRegistration for DynNodeLike {
    fn get_type_registration() -> ::bevy::reflect::TypeRegistration {
        let mut registration = ::bevy::reflect::TypeRegistration::of::<Self>();
        registration.insert::<::bevy::reflect::ReflectFromPtr>(
            ::bevy::reflect::FromType::<Self>::from_type(),
        );
        registration.insert::<::bevy::reflect::ReflectFromReflect>(
            ::bevy::reflect::FromType::<Self>::from_type(),
        );
        registration
    }
}
impl ::bevy::reflect::Typed for DynNodeLike {
    #[inline]
    fn type_info() -> &'static ::bevy::reflect::TypeInfo {
        static CELL: ::bevy::reflect::utility::NonGenericTypeInfoCell =
            ::bevy::reflect::utility::NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| {
            ::bevy::reflect::TypeInfo::TupleStruct(::bevy::reflect::TupleStructInfo::new::<Self>(
                &[
                        // ::bevy::reflect::UnnamedField::new::<Box<dyn NodeLike>>(0usize),
                    ],
            ))
        })
    }
}
impl ::bevy::reflect::TypePath for DynNodeLike {
    fn type_path() -> &'static str {
        ::core::concat!(::core::concat!(::core::module_path!(), "::"), "DynNodeLike")
    }
    fn short_type_path() -> &'static str {
        "DynNodeLike"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynNodeLike")
    }
    fn crate_name() -> Option<&'static str> {
        Some(::core::module_path!().split(':').next().unwrap())
    }
    fn module_path() -> Option<&'static str> {
        Some(::core::module_path!())
    }
}
impl ::bevy::reflect::Reflect for DynNodeLike {
    #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn ::core::any::Any> {
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
    fn into_reflect(self: Box<Self>) -> Box<dyn ::bevy::reflect::Reflect> {
        self
    }
    #[inline]
    fn as_reflect(&self) -> &dyn ::bevy::reflect::Reflect {
        self
    }
    #[inline]
    fn as_reflect_mut(&mut self) -> &mut dyn ::bevy::reflect::Reflect {
        self
    }
    #[inline]
    fn set(
        &mut self,
        value: Box<dyn ::bevy::reflect::Reflect>,
    ) -> Result<(), Box<dyn ::bevy::reflect::Reflect>> {
        *self = <dyn ::bevy::reflect::Reflect>::take(value)?;
        Ok(())
    }
}
::bevy::reflect::__macro_exports::auto_register::inventory::submit! {
    ::bevy::reflect::__macro_exports::auto_register::AutomaticReflectRegistrations(<DynNodeLike as ::bevy::reflect::__macro_exports::auto_register::RegisterForReflection> ::__register)
}
impl ::bevy::reflect::TupleStruct for DynNodeLike {
    fn field(&self, index: usize) -> Option<&dyn ::bevy::reflect::PartialReflect> {
        match index {
            0usize => Some(self.0.as_partial_reflect()),
            _ => None,
        }
    }
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn ::bevy::reflect::PartialReflect> {
        match index {
            0usize => Some(self.0.as_partial_reflect_mut()),
            _ => None,
        }
    }
    #[inline]
    fn field_len(&self) -> usize {
        1usize
    }
    #[inline]
    fn iter_fields(&'_ self) -> ::bevy::reflect::TupleStructFieldIter<'_> {
        ::bevy::reflect::TupleStructFieldIter::new(self)
    }
}
impl ::bevy::reflect::PartialReflect for DynNodeLike {
    #[inline]
    fn get_represented_type_info(&self) -> Option<&'static ::bevy::reflect::TypeInfo> {
        Some(<Self as ::bevy::reflect::Typed>::type_info())
    }
    #[inline]
    fn try_apply(
        &mut self,
        value: &dyn ::bevy::reflect::PartialReflect,
    ) -> Result<(), ::bevy::reflect::ApplyError> {
        if let ::bevy::reflect::ReflectRef::TupleStruct(struct_value) =
            ::bevy::reflect::PartialReflect::reflect_ref(value)
        {
            for (i, value) in ::core::iter::Iterator::enumerate(
                ::bevy::reflect::TupleStruct::iter_fields(struct_value),
            ) {
                if let ::core::option::Option::Some(v) =
                    ::bevy::reflect::TupleStruct::field_mut(self, i)
                {
                    ::bevy::reflect::PartialReflect::try_apply(v, value)?;
                }
            }
        } else {
            return Err(::bevy::reflect::ApplyError::MismatchedKinds {
                from_kind: ::bevy::reflect::PartialReflect::reflect_kind(value),
                to_kind: ::bevy::reflect::ReflectKind::TupleStruct,
            });
        }
        Result::Ok(())
    }
    #[inline]
    fn reflect_kind(&self) -> ::bevy::reflect::ReflectKind {
        ::bevy::reflect::ReflectKind::TupleStruct
    }
    #[inline]
    fn reflect_ref(&'_ self) -> ::bevy::reflect::ReflectRef<'_> {
        ::bevy::reflect::ReflectRef::TupleStruct(self)
    }
    #[inline]
    fn reflect_mut(&'_ mut self) -> ::bevy::reflect::ReflectMut<'_> {
        ::bevy::reflect::ReflectMut::TupleStruct(self)
    }
    #[inline]
    fn reflect_owned(self: Box<Self>) -> ::bevy::reflect::ReflectOwned {
        ::bevy::reflect::ReflectOwned::TupleStruct(self)
    }
    #[inline]
    fn try_into_reflect(
        self: Box<Self>,
    ) -> Result<Box<dyn ::bevy::reflect::Reflect>, Box<dyn ::bevy::reflect::PartialReflect>> {
        ::core::result::Result::Ok(self)
    }
    #[inline]
    fn try_as_reflect(&self) -> ::core::option::Option<&dyn ::bevy::reflect::Reflect> {
        ::core::option::Option::Some(self)
    }
    #[inline]
    fn try_as_reflect_mut(&mut self) -> ::core::option::Option<&mut dyn ::bevy::reflect::Reflect> {
        ::core::option::Option::Some(self)
    }
    #[inline]
    fn into_partial_reflect(
        self: ::bevy::reflect::__macro_exports::alloc_utils::Box<Self>,
    ) -> ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::bevy::reflect::PartialReflect>
    {
        self
    }
    #[inline]
    fn as_partial_reflect(&self) -> &dyn ::bevy::reflect::PartialReflect {
        self
    }
    #[inline]
    fn as_partial_reflect_mut(&mut self) -> &mut dyn ::bevy::reflect::PartialReflect {
        self
    }
    fn reflect_partial_eq(&self, value: &dyn ::bevy::reflect::PartialReflect) -> Option<bool> {
        (::bevy::reflect::tuple_struct_partial_eq)(self, value)
    }
    #[inline]
    fn reflect_clone(
        &self,
    ) -> Result<Box<dyn ::bevy::reflect::Reflect>, ::bevy::reflect::ReflectCloneError> {
        Ok(::bevy::reflect::__macro_exports::alloc_utils::Box::new(
            Self(self.0.clone_node_like()),
        ))
    }
}
impl ::bevy::reflect::FromReflect for DynNodeLike {
    fn from_reflect(reflect: &dyn ::bevy::reflect::PartialReflect) -> Option<Self> {
        Some(reflect.try_downcast_ref::<Self>()?.clone())
    }
}
