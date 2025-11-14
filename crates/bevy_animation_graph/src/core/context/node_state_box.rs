use crate::prelude::node_states::GraphStateType;

#[derive(Debug)]
pub struct NodeStateBox {
    pub value: Box<dyn GraphStateType>,
}

impl Clone for NodeStateBox {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone_box(),
        }
    }
}

// manual impl of the `Reflect` macro
// How to reproduce:
// * Replace the `Box<stuff>` in the struct def with something else (e.g. `String`)
// * Add a `#[derive(Reflect)]`
// * Expand the macro, and copy-paste the generated code here
// * Change back the `String` to `Box<stuff>`
// * Fix the errors
//
// Why is this here? We cannot derive Reflect for `Box<dyn Reflect>` types.
// Tracking issue: https://github.com/bevyengine/bevy/issues/3392
// Once that is resolved we can remove these manual impls
const _: () = {
    impl ::bevy::reflect::GetTypeRegistration for NodeStateBox {
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
        #[inline(never)]
        fn register_type_dependencies(registry: &mut ::bevy::reflect::TypeRegistry) {
            <String as ::bevy::reflect::__macro_exports::RegisterForReflection>::__register(
                registry,
            );
        }
    }
    impl ::bevy::reflect::Typed for NodeStateBox {
        #[inline]
        fn type_info() -> &'static ::bevy::reflect::TypeInfo {
            static CELL: ::bevy::reflect::utility::NonGenericTypeInfoCell =
                ::bevy::reflect::utility::NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| {
                ::bevy::reflect::TypeInfo::Struct(::bevy::reflect::StructInfo::new::<Self>(&[
                    ::bevy::reflect::NamedField::new::<String>("value"),
                ]))
            })
        }
    }
    impl ::bevy::reflect::TypePath for NodeStateBox {
        fn type_path() -> &'static str {
            ::core::concat!(
                ::core::concat!(::core::module_path!(), "::"),
                "NodeStateBox"
            )
        }
        fn short_type_path() -> &'static str {
            "NodeStateBox"
        }
        fn type_ident() -> Option<&'static str> {
            ::core::option::Option::Some("NodeStateBox")
        }
        fn crate_name() -> Option<&'static str> {
            ::core::option::Option::Some(::core::module_path!().split(':').next().unwrap())
        }
        fn module_path() -> Option<&'static str> {
            ::core::option::Option::Some(::core::module_path!())
        }
    }
    impl ::bevy::reflect::Reflect for NodeStateBox {
        #[inline]
        fn into_any(
            self: ::bevy::reflect::__macro_exports::alloc_utils::Box<Self>,
        ) -> ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::core::any::Any> {
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
            self: ::bevy::reflect::__macro_exports::alloc_utils::Box<Self>,
        ) -> ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::bevy::reflect::Reflect>
        {
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
            value: ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::bevy::reflect::Reflect>,
        ) -> ::core::result::Result<
            (),
            ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::bevy::reflect::Reflect>,
        > {
            *self = <dyn ::bevy::reflect::Reflect>::take(value)?;
            ::core::result::Result::Ok(())
        }
    }
    ::bevy::reflect::__macro_exports::auto_register::inventory::submit! {
        ::bevy::reflect::__macro_exports::auto_register::AutomaticReflectRegistrations(<NodeStateBox as ::bevy::reflect::__macro_exports::auto_register::RegisterForReflection> ::__register)
    }
    impl ::bevy::reflect::Struct for NodeStateBox {
        fn field(
            &self,
            name: &str,
        ) -> ::core::option::Option<&dyn ::bevy::reflect::PartialReflect> {
            match name {
                "value" => ::core::option::Option::Some(self.value.as_reflect()),
                _ => ::core::option::Option::None,
            }
        }
        fn field_mut(
            &mut self,
            name: &str,
        ) -> ::core::option::Option<&mut dyn ::bevy::reflect::PartialReflect> {
            match name {
                "value" => ::core::option::Option::Some(self.value.as_reflect_mut()),
                _ => ::core::option::Option::None,
            }
        }
        fn field_at(
            &self,
            index: usize,
        ) -> ::core::option::Option<&dyn ::bevy::reflect::PartialReflect> {
            match index {
                0usize => ::core::option::Option::Some(self.value.as_reflect()),
                _ => ::core::option::Option::None,
            }
        }
        fn field_at_mut(
            &mut self,
            index: usize,
        ) -> ::core::option::Option<&mut dyn ::bevy::reflect::PartialReflect> {
            match index {
                0usize => ::core::option::Option::Some(self.value.as_reflect_mut()),
                _ => ::core::option::Option::None,
            }
        }
        fn name_at(&self, index: usize) -> ::core::option::Option<&str> {
            match index {
                0usize => ::core::option::Option::Some("value"),
                _ => ::core::option::Option::None,
            }
        }
        fn field_len(&self) -> usize {
            1usize
        }
        fn iter_fields(&'_ self) -> ::bevy::reflect::FieldIter<'_> {
            ::bevy::reflect::FieldIter::new(self)
        }
        fn to_dynamic_struct(&self) -> ::bevy::reflect::DynamicStruct {
            let mut dynamic: ::bevy::reflect::DynamicStruct = ::core::default::Default::default();
            dynamic.set_represented_type(
                ::bevy::reflect::PartialReflect::get_represented_type_info(self),
            );
            dynamic.insert_boxed(
                "value",
                ::bevy::reflect::PartialReflect::to_dynamic(self.value.as_reflect()),
            );
            dynamic
        }
    }
    impl ::bevy::reflect::PartialReflect for NodeStateBox {
        #[inline]
        fn get_represented_type_info(
            &self,
        ) -> ::core::option::Option<&'static ::bevy::reflect::TypeInfo> {
            ::core::option::Option::Some(<Self as ::bevy::reflect::Typed>::type_info())
        }
        #[inline]
        fn try_apply(
            &mut self,
            value: &dyn ::bevy::reflect::PartialReflect,
        ) -> ::core::result::Result<(), ::bevy::reflect::ApplyError> {
            if let ::bevy::reflect::ReflectRef::Struct(struct_value) =
                ::bevy::reflect::PartialReflect::reflect_ref(value)
            {
                for (i, value) in ::core::iter::Iterator::enumerate(
                    ::bevy::reflect::Struct::iter_fields(struct_value),
                ) {
                    let name = ::bevy::reflect::Struct::name_at(struct_value, i).unwrap();
                    if let ::core::option::Option::Some(v) =
                        ::bevy::reflect::Struct::field_mut(self, name)
                    {
                        ::bevy::reflect::PartialReflect::try_apply(v, value)?;
                    }
                }
            } else {
                return ::core::result::Result::Err(::bevy::reflect::ApplyError::MismatchedKinds {
                    from_kind: ::bevy::reflect::PartialReflect::reflect_kind(value),
                    to_kind: ::bevy::reflect::ReflectKind::Struct,
                });
            }
            ::core::result::Result::Ok(())
        }
        #[inline]
        fn reflect_kind(&self) -> ::bevy::reflect::ReflectKind {
            ::bevy::reflect::ReflectKind::Struct
        }
        #[inline]
        fn reflect_ref(&'_ self) -> ::bevy::reflect::ReflectRef<'_> {
            ::bevy::reflect::ReflectRef::Struct(self)
        }
        #[inline]
        fn reflect_mut(&'_ mut self) -> ::bevy::reflect::ReflectMut<'_> {
            ::bevy::reflect::ReflectMut::Struct(self)
        }
        #[inline]
        fn reflect_owned(
            self: ::bevy::reflect::__macro_exports::alloc_utils::Box<Self>,
        ) -> ::bevy::reflect::ReflectOwned {
            ::bevy::reflect::ReflectOwned::Struct(self)
        }
        #[inline]
        fn try_into_reflect(
            self: ::bevy::reflect::__macro_exports::alloc_utils::Box<Self>,
        ) -> ::core::result::Result<
            ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::bevy::reflect::Reflect>,
            ::bevy::reflect::__macro_exports::alloc_utils::Box<dyn ::bevy::reflect::PartialReflect>,
        > {
            ::core::result::Result::Ok(self)
        }
        #[inline]
        fn try_as_reflect(&self) -> ::core::option::Option<&dyn ::bevy::reflect::Reflect> {
            ::core::option::Option::Some(self)
        }
        #[inline]
        fn try_as_reflect_mut(
            &mut self,
        ) -> ::core::option::Option<&mut dyn ::bevy::reflect::Reflect> {
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
        fn reflect_partial_eq(
            &self,
            value: &dyn ::bevy::reflect::PartialReflect,
        ) -> ::core::option::Option<bool> {
            (::bevy::reflect::struct_partial_eq)(self, value)
        }
        #[inline]
        #[allow(
            unreachable_code,
            reason = "Ignored fields without a `clone` attribute will early-return with an error"
        )]
        fn reflect_clone(
            &self,
        ) -> Result<Box<dyn ::bevy::reflect::Reflect>, ::bevy::reflect::ReflectCloneError> {
            Ok(Box::new(Self {
                value: self.value.clone_box(),
            }))
        }
    }
    impl ::bevy::reflect::FromReflect for NodeStateBox {
        fn from_reflect(reflect: &dyn ::bevy::reflect::PartialReflect) -> Option<Self> {
            Some(reflect.try_downcast_ref::<Self>()?.clone())
        }
    }
};
