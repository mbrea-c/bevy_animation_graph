//! Carbon copy of [`bevy::reflect::serde`] but with an added [`ValueProcessor`]
//! used for potentially overriding the deserialization of certain fields.
//!
//! We use this to automatically load [`Handle`]s in reflect-built structs. Note
//! that this process is actually quite hacky - see [`crate::utils::asset`].
//!
//! This should also be upstreamed to Bevy eventually, but for some more general
//! use case than loading handles.

use bevy::reflect::serde::TypeRegistrationDeserializer;
use bevy::reflect::{
    erased_serde, serde::SerializationData, ArrayInfo, DynamicArray, DynamicEnum, DynamicList,
    DynamicMap, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, EnumInfo,
    ListInfo, Map, MapInfo, NamedField, Reflect, ReflectDeserialize, StructInfo, StructVariantInfo,
    TupleInfo, TupleStructInfo, TupleVariantInfo, TypeInfo, TypeRegistration, TypeRegistry,
    VariantInfo,
};
use serde::de::{
    DeserializeSeed, EnumAccess, Error, IgnoredAny, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde::Deserialize;
use std::any::TypeId;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::slice::Iter;

trait StructLikeInfo {
    fn get_field(&self, name: &str) -> Option<&NamedField>;
    fn field_at(&self, index: usize) -> Option<&NamedField>;
    fn get_field_len(&self) -> usize;
    fn iter_fields(&self) -> Iter<'_, NamedField>;
}

trait TupleLikeInfo {
    fn get_field_len(&self) -> usize;
}

trait Container {
    fn get_field_registration<'a, E: Error>(
        &self,
        index: usize,
        registry: &'a TypeRegistry,
    ) -> Result<&'a TypeRegistration, E>;
}

impl StructLikeInfo for StructInfo {
    fn get_field(&self, name: &str) -> Option<&NamedField> {
        self.field(name)
    }

    fn field_at(&self, index: usize) -> Option<&NamedField> {
        self.field_at(index)
    }

    fn get_field_len(&self) -> usize {
        self.field_len()
    }

    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

impl Container for StructInfo {
    fn get_field_registration<'a, E: Error>(
        &self,
        index: usize,
        registry: &'a TypeRegistry,
    ) -> Result<&'a TypeRegistration, E> {
        let field = self.field_at(index).ok_or_else(|| {
            Error::custom(format_args!(
                "no field at index {} on struct {}",
                index,
                self.type_path(),
            ))
        })?;
        get_registration(field.type_id(), field.type_path(), registry)
    }
}

impl StructLikeInfo for StructVariantInfo {
    fn get_field(&self, name: &str) -> Option<&NamedField> {
        self.field(name)
    }

    fn field_at(&self, index: usize) -> Option<&NamedField> {
        self.field_at(index)
    }

    fn get_field_len(&self) -> usize {
        self.field_len()
    }

    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

impl Container for StructVariantInfo {
    fn get_field_registration<'a, E: Error>(
        &self,
        index: usize,
        registry: &'a TypeRegistry,
    ) -> Result<&'a TypeRegistration, E> {
        let field = self.field_at(index).ok_or_else(|| {
            Error::custom(format_args!(
                "no field at index {} on variant {}",
                index,
                self.name(),
            ))
        })?;
        get_registration(field.type_id(), field.type_path(), registry)
    }
}

impl TupleLikeInfo for TupleInfo {
    fn get_field_len(&self) -> usize {
        self.field_len()
    }
}

impl Container for TupleInfo {
    fn get_field_registration<'a, E: Error>(
        &self,
        index: usize,
        registry: &'a TypeRegistry,
    ) -> Result<&'a TypeRegistration, E> {
        let field = self.field_at(index).ok_or_else(|| {
            Error::custom(format_args!(
                "no field at index {} on tuple {}",
                index,
                self.type_path(),
            ))
        })?;
        get_registration(field.type_id(), field.type_path(), registry)
    }
}

impl TupleLikeInfo for TupleStructInfo {
    fn get_field_len(&self) -> usize {
        self.field_len()
    }
}

impl Container for TupleStructInfo {
    fn get_field_registration<'a, E: Error>(
        &self,
        index: usize,
        registry: &'a TypeRegistry,
    ) -> Result<&'a TypeRegistration, E> {
        let field = self.field_at(index).ok_or_else(|| {
            Error::custom(format_args!(
                "no field at index {} on tuple struct {}",
                index,
                self.type_path(),
            ))
        })?;
        get_registration(field.type_id(), field.type_path(), registry)
    }
}

impl TupleLikeInfo for TupleVariantInfo {
    fn get_field_len(&self) -> usize {
        self.field_len()
    }
}

impl Container for TupleVariantInfo {
    fn get_field_registration<'a, E: Error>(
        &self,
        index: usize,
        registry: &'a TypeRegistry,
    ) -> Result<&'a TypeRegistration, E> {
        let field = self.field_at(index).ok_or_else(|| {
            Error::custom(format_args!(
                "no field at index {} on tuple variant {}",
                index,
                self.name(),
            ))
        })?;
        get_registration(field.type_id(), field.type_path(), registry)
    }
}

/// A debug struct used for error messages that displays a list of expected values.
///
/// # Example
///
/// ```ignore (Can't import private struct from doctest)
/// let expected = vec!["foo", "bar", "baz"];
/// assert_eq!("`foo`, `bar`, `baz`", format!("{}", ExpectedValues(expected)));
/// ```
struct ExpectedValues<T: Display>(Vec<T>);

impl<T: Display> Debug for ExpectedValues<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "`{item}`")?;
            if index < len - 1 {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

/// Represents a simple reflected identifier.
#[derive(Debug, Clone, Eq, PartialEq)]
struct Ident(String);

impl<'de> Deserialize<'de> for Ident {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdentVisitor;

        impl<'de> Visitor<'de> for IdentVisitor {
            type Value = Ident;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("identifier")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Ident(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Ident(value))
            }
        }

        deserializer.deserialize_identifier(IdentVisitor)
    }
}

pub struct ReflectDeserializer<'a, 'p> {
    registry: &'a TypeRegistry,
    pub processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'a, 'p> ReflectDeserializer<'a, 'p> {
    pub fn new(registry: &'a TypeRegistry, processor: Option<&'a mut ValueProcessor<'p>>) -> Self {
        Self {
            registry,
            processor,
        }
    }
}

impl<'de> DeserializeSeed<'de> for ReflectDeserializer<'_, '_> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UntypedReflectDeserializerVisitor<'a, 'p> {
            registry: &'a TypeRegistry,
            processor: Option<&'a mut ValueProcessor<'p>>,
        }

        impl<'de> Visitor<'de> for UntypedReflectDeserializerVisitor<'_, '_> {
            type Value = Box<dyn Reflect>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter
                    .write_str("map containing `type` and `value` entries for the reflected value")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let registration = map
                    .next_key_seed(TypeRegistrationDeserializer::new(self.registry))?
                    .ok_or_else(|| Error::invalid_length(0, &"a single entry"))?;

                let value = map.next_value_seed(TypedReflectDeserializer {
                    registration,
                    registry: self.registry,
                    processor: self.processor,
                })?;

                if map.next_key::<IgnoredAny>()?.is_some() {
                    return Err(Error::invalid_length(2, &"a single entry"));
                }

                Ok(value)
            }
        }

        deserializer.deserialize_map(UntypedReflectDeserializerVisitor {
            registry: self.registry,
            processor: self.processor,
        })
    }
}

/// A deserializer for reflected types whose [`TypeRegistration`] is known.
///
/// This is the deserializer counterpart to [`TypedReflectSerializer`].
///
/// See [`ReflectDeserializer`] for a deserializer that expects an unknown type.
///
/// # Input
///
/// Since the type is already known, the input is just the serialized data.
///
/// # Output
///
/// This deserializer will return a [`Box<dyn Reflect>`] containing the deserialized data.
///
/// For value types (i.e. [`ReflectKind::Value`]) or types that register [`ReflectDeserialize`] type data,
/// this `Box` will contain the expected type.
/// For example, deserializing an `i32` will return a `Box<i32>` (as a `Box<dyn Reflect>`).
///
/// Otherwise, this `Box` will contain the dynamic equivalent.
/// For example, a deserialized struct might return a [`Box<DynamicStruct>`]
/// and a deserialized `Vec` might return a [`Box<DynamicList>`].
///
/// This means that if the actual type is needed, these dynamic representations will need to
/// be converted to the concrete type using [`FromReflect`] or [`ReflectFromReflect`].
///
/// # Example
///
/// ```ignore
/// # use std::any::TypeId;
/// # use serde::de::DeserializeSeed;
/// # use bevy_reflect::prelude::*;
/// # use bevy_reflect::{DynamicStruct, TypeRegistry, serde::TypedReflectDeserializer};
/// #[derive(Reflect, PartialEq, Debug)]
/// struct MyStruct {
///   value: i32
/// }
///
/// let mut registry = TypeRegistry::default();
/// registry.register::<MyStruct>();
///
/// let input = r#"(
///   value: 123
/// )"#;
///
/// let registration = registry.get(TypeId::of::<MyStruct>()).unwrap();
///
/// let mut deserializer = ron::Deserializer::from_str(input).unwrap();
/// let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
///
/// let output: Box<dyn Reflect> = reflect_deserializer.deserialize(&mut deserializer).unwrap();
///
/// // Since `MyStruct` is not a value type and does not register `ReflectDeserialize`,
/// // we know that its deserialized representation will be a `DynamicStruct`.
/// assert!(output.is::<DynamicStruct>());
/// assert!(output.represents::<MyStruct>());
///
/// // We can convert back to `MyStruct` using `FromReflect`.
/// let value: MyStruct = <MyStruct as FromReflect>::from_reflect(&*output).unwrap();
/// assert_eq!(value, MyStruct { value: 123 });
///
/// // We can also do this dynamically with `ReflectFromReflect`.
/// let type_id = output.get_represented_type_info().unwrap().type_id();
/// let reflect_from_reflect = registry.get_type_data::<ReflectFromReflect>(type_id).unwrap();
/// let value: Box<dyn Reflect> = reflect_from_reflect.from_reflect(&*output).unwrap();
/// assert!(value.is::<MyStruct>());
/// assert_eq!(value.take::<MyStruct>().unwrap(), MyStruct { value: 123 });
/// ```
///
/// [`TypedReflectSerializer`]: crate::serde::TypedReflectSerializer
/// [`Box<dyn Reflect>`]: crate::Reflect
/// [`ReflectKind::Value`]: crate::ReflectKind::Value
/// [`ReflectDeserialize`]: crate::ReflectDeserialize
/// [`Box<DynamicStruct>`]: crate::DynamicStruct
/// [`Box<DynamicList>`]: crate::DynamicList
/// [`FromReflect`]: crate::FromReflect
/// [`ReflectFromReflect`]: crate::ReflectFromReflect
pub struct TypedReflectDeserializer<'a, 'p> {
    pub registration: &'a TypeRegistration,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut ValueProcessor<'p>>,
}

// Why are we using dynamic dispatch? Why not just
// `TypedReflectDeserializer<P: ValueProcessor>?`
//   I tried to get this approach working for a long time. But we need
//   `impl<T: ValueProcessor> ValueProcessor for &mut T`, which for some reason
//   causes rustc to overflow when type checking. Making it static probably is
//   the best approach, but I've already spent way too much time on this.
//
// Why `ValueProcessor<'p>` instead of `ValueProcessor`?
//   Otherwise the closures must be `'static`. This makes doing what we do in
//   `animation_graph::serial` impossible, since we would be moving our
//   `&mut LoadContext` into the closure.
//
// Can we pass some data out of `can_deserialize` into `deserialize`?
//   Technically this is possible, but would add an extra type parameter on
//   `ValueProcessor`, which I don't want to do, since it would infect
//   `TypedReflectDeserializer`, which is just annoying to deal with.
//   If we were using a non-dyn method (the original one described above), then
//   yes, but there is another problem. The seed produced by `seed_deserialize`
//   won't be able to take values with lifetimes from the `&TypeRegistration`.
//   We *need* HKTs for this. So this seed won't be particularly useful.
//
// Why `Option<&mut ValueProcessor>` instead of `Option<ValueProcessor>`?
//   `TypedReflectDeserializer` will make other deserializers, which in turn
//   will make more `TypedReflectDeserializers` values. We can't guarantee
//   `ValueProcessor: Clone` since the closures inside may not (will not) be
//   `Clone`.

#[expect(
    clippy::type_complexity,
    reason = "can't use a type alias as a trait in `Box` here"
)]
pub struct ValueProcessor<'p> {
    pub can_deserialize: Box<dyn FnMut(&TypeRegistration) -> bool + 'p>,
    pub deserialize: Box<
        dyn FnMut(
                &TypeRegistration,
                &mut dyn erased_serde::Deserializer,
            ) -> Result<Box<dyn Reflect>, erased_serde::Error>
            + 'p,
    >,
}

impl<'de> DeserializeSeed<'de> for TypedReflectDeserializer<'_, '_> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Ask the value processor to process this
        if let Some(processor) = self.processor.as_deref_mut() {
            if (processor.can_deserialize)(self.registration) {
                let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
                return (processor.deserialize)(self.registration, &mut deserializer)
                    // TODO: is there a better way of returning this error?
                    .map_err(|err| serde::de::Error::custom(err.to_string()));
            }
        }

        let type_path = self.registration.type_info().type_path();

        // Handle both Value case and types that have a custom `ReflectDeserialize`
        if let Some(deserialize_reflect) = self.registration.data::<ReflectDeserialize>() {
            let value = deserialize_reflect.deserialize(deserializer)?;
            return Ok(value);
        }

        match self.registration.type_info() {
            TypeInfo::Struct(struct_info) => {
                let mut dynamic_struct = deserializer.deserialize_struct(
                    struct_info.type_path_table().ident().unwrap(),
                    struct_info.field_names(),
                    StructVisitor {
                        struct_info,
                        registration: self.registration,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?;
                dynamic_struct.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_struct))
            }
            TypeInfo::TupleStruct(tuple_struct_info) => {
                let mut dynamic_tuple_struct = deserializer.deserialize_tuple_struct(
                    tuple_struct_info.type_path_table().ident().unwrap(),
                    tuple_struct_info.field_len(),
                    TupleStructVisitor {
                        tuple_struct_info,
                        registry: self.registry,
                        registration: self.registration,
                        processor: self.processor,
                    },
                )?;
                dynamic_tuple_struct.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_tuple_struct))
            }
            TypeInfo::List(list_info) => {
                let mut dynamic_list = deserializer.deserialize_seq(ListVisitor {
                    list_info,
                    registry: self.registry,
                    processor: self.processor,
                })?;
                dynamic_list.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_list))
            }
            TypeInfo::Array(array_info) => {
                let mut dynamic_array = deserializer.deserialize_tuple(
                    array_info.capacity(),
                    ArrayVisitor {
                        array_info,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?;
                dynamic_array.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_array))
            }
            TypeInfo::Map(map_info) => {
                let mut dynamic_map = deserializer.deserialize_map(MapVisitor {
                    map_info,
                    registry: self.registry,
                    processor: self.processor,
                })?;
                dynamic_map.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_map))
            }
            TypeInfo::Tuple(tuple_info) => {
                let mut dynamic_tuple = deserializer.deserialize_tuple(
                    tuple_info.field_len(),
                    TupleVisitor {
                        tuple_info,
                        registration: self.registration,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?;
                dynamic_tuple.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_tuple))
            }
            TypeInfo::Enum(enum_info) => {
                let mut dynamic_enum = if enum_info.type_path_table().module_path()
                    == Some("core::option")
                    && enum_info.type_path_table().ident() == Some("Option")
                {
                    deserializer.deserialize_option(OptionVisitor {
                        enum_info,
                        registry: self.registry,
                        processor: self.processor.as_deref_mut(),
                    })?
                } else {
                    deserializer.deserialize_enum(
                        enum_info.type_path_table().ident().unwrap(),
                        enum_info.variant_names(),
                        EnumVisitor {
                            enum_info,
                            registration: self.registration,
                            registry: self.registry,
                            processor: self.processor.as_deref_mut(),
                        },
                    )?
                };
                dynamic_enum.set_represented_type(Some(self.registration.type_info()));
                Ok(Box::new(dynamic_enum))
            }
            TypeInfo::Value(_) => {
                // This case should already be handled
                Err(Error::custom(format_args!(
                    "the TypeRegistration for {type_path} doesn't have ReflectDeserialize",
                )))
            }
        }
    }
}

struct StructVisitor<'a, 'p> {
    struct_info: &'static StructInfo,
    registration: &'a TypeRegistration,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for StructVisitor<'_, '_> {
    type Value = DynamicStruct;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected struct value")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        visit_struct_seq(
            &mut seq,
            self.struct_info,
            self.registration,
            self.registry,
            self.processor,
        )
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        visit_struct(
            &mut map,
            self.struct_info,
            self.registration,
            self.registry,
            self.processor,
        )
    }
}

struct TupleStructVisitor<'a, 'p> {
    tuple_struct_info: &'static TupleStructInfo,
    registry: &'a TypeRegistry,
    registration: &'a TypeRegistration,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for TupleStructVisitor<'_, '_> {
    type Value = DynamicTupleStruct;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected tuple struct value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        visit_tuple(
            &mut seq,
            self.tuple_struct_info,
            self.registration,
            self.registry,
            self.processor,
        )
        .map(DynamicTupleStruct::from)
    }
}

struct TupleVisitor<'a, 'p> {
    tuple_info: &'static TupleInfo,
    registration: &'a TypeRegistration,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for TupleVisitor<'_, '_> {
    type Value = DynamicTuple;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected tuple value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        visit_tuple(
            &mut seq,
            self.tuple_info,
            self.registration,
            self.registry,
            self.processor,
        )
    }
}

struct ArrayVisitor<'a, 'p> {
    array_info: &'static ArrayInfo,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for ArrayVisitor<'_, '_> {
    type Value = DynamicArray;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected array value")
    }

    fn visit_seq<V>(mut self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        let registration = get_registration(
            self.array_info.item_type_id(),
            self.array_info.item_type_path_table().path(),
            self.registry,
        )?;
        while let Some(value) = seq.next_element_seed(TypedReflectDeserializer {
            registration,
            registry: self.registry,
            processor: self.processor.as_deref_mut(),
        })? {
            vec.push(value);
        }

        if vec.len() != self.array_info.capacity() {
            return Err(Error::invalid_length(
                vec.len(),
                &self.array_info.capacity().to_string().as_str(),
            ));
        }

        Ok(DynamicArray::new(vec.into_boxed_slice()))
    }
}

struct ListVisitor<'a, 'p> {
    list_info: &'static ListInfo,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for ListVisitor<'_, '_> {
    type Value = DynamicList;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected list value")
    }

    fn visit_seq<V>(mut self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut list = DynamicList::default();
        let registration = get_registration(
            self.list_info.item_type_id(),
            self.list_info.item_type_path_table().path(),
            self.registry,
        )?;
        while let Some(value) = seq.next_element_seed(TypedReflectDeserializer {
            registration,
            registry: self.registry,
            processor: self.processor.as_deref_mut(),
        })? {
            list.push_box(value);
        }
        Ok(list)
    }
}

struct MapVisitor<'a, 'p> {
    map_info: &'static MapInfo,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for MapVisitor<'_, '_> {
    type Value = DynamicMap;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected map value")
    }

    fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut dynamic_map = DynamicMap::default();
        let key_registration = get_registration(
            self.map_info.key_type_id(),
            self.map_info.key_type_path_table().path(),
            self.registry,
        )?;
        let value_registration = get_registration(
            self.map_info.value_type_id(),
            self.map_info.value_type_path_table().path(),
            self.registry,
        )?;
        while let Some(key) = map.next_key_seed(TypedReflectDeserializer {
            registration: key_registration,
            registry: self.registry,
            processor: self.processor.as_deref_mut(),
        })? {
            let value = map.next_value_seed(TypedReflectDeserializer {
                registration: value_registration,
                registry: self.registry,
                processor: self.processor.as_deref_mut(),
            })?;
            dynamic_map.insert_boxed(key, value);
        }

        Ok(dynamic_map)
    }
}

struct EnumVisitor<'a, 'p> {
    enum_info: &'static EnumInfo,
    registration: &'a TypeRegistration,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for EnumVisitor<'_, '_> {
    type Value = DynamicEnum;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected enum value")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let mut dynamic_enum = DynamicEnum::default();
        let (variant_info, variant) = data.variant_seed(VariantDeserializer {
            enum_info: self.enum_info,
        })?;

        let value: DynamicVariant = match variant_info {
            VariantInfo::Unit(..) => variant.unit_variant()?.into(),
            VariantInfo::Struct(struct_info) => variant
                .struct_variant(
                    struct_info.field_names(),
                    StructVariantVisitor {
                        struct_info,
                        registration: self.registration,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?
                .into(),
            VariantInfo::Tuple(tuple_info) if tuple_info.field_len() == 1 => {
                let registration = tuple_info.get_field_registration(0, self.registry)?;
                let value = variant.newtype_variant_seed(TypedReflectDeserializer {
                    registration,
                    registry: self.registry,
                    processor: self.processor,
                })?;
                let mut dynamic_tuple = DynamicTuple::default();
                dynamic_tuple.insert_boxed(value);
                dynamic_tuple.into()
            }
            VariantInfo::Tuple(tuple_info) => variant
                .tuple_variant(
                    tuple_info.field_len(),
                    TupleVariantVisitor {
                        tuple_info,
                        registration: self.registration,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?
                .into(),
        };
        let variant_name = variant_info.name();
        let variant_index = self
            .enum_info
            .index_of(variant_name)
            .expect("variant should exist");
        dynamic_enum.set_variant_with_index(variant_index, variant_name, value);
        Ok(dynamic_enum)
    }
}

struct VariantDeserializer {
    enum_info: &'static EnumInfo,
}

impl<'de> DeserializeSeed<'de> for VariantDeserializer {
    type Value = &'static VariantInfo;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VariantVisitor(&'static EnumInfo);

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = &'static VariantInfo;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("expected either a variant index or variant name")
            }

            fn visit_u32<E>(self, variant_index: u32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.0.variant_at(variant_index as usize).ok_or_else(|| {
                    Error::custom(format_args!(
                        "no variant found at index `{}` on enum `{}`",
                        variant_index,
                        self.0.type_path()
                    ))
                })
            }

            fn visit_str<E>(self, variant_name: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.0.variant(variant_name).ok_or_else(|| {
                    let names = self.0.iter().map(|variant| variant.name());
                    Error::custom(format_args!(
                        "unknown variant `{}`, expected one of {:?}",
                        variant_name,
                        ExpectedValues(names.collect())
                    ))
                })
            }
        }

        deserializer.deserialize_identifier(VariantVisitor(self.enum_info))
    }
}

struct StructVariantVisitor<'a, 'p> {
    struct_info: &'static StructVariantInfo,
    registration: &'a TypeRegistration,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for StructVariantVisitor<'_, '_> {
    type Value = DynamicStruct;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected struct variant value")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        visit_struct_seq(
            &mut seq,
            self.struct_info,
            self.registration,
            self.registry,
            self.processor,
        )
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        visit_struct(
            &mut map,
            self.struct_info,
            self.registration,
            self.registry,
            self.processor,
        )
    }
}

struct TupleVariantVisitor<'a, 'p> {
    tuple_info: &'static TupleVariantInfo,
    registration: &'a TypeRegistration,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for TupleVariantVisitor<'_, '_> {
    type Value = DynamicTuple;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected tuple variant value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        visit_tuple(
            &mut seq,
            self.tuple_info,
            self.registration,
            self.registry,
            self.processor,
        )
    }
}

struct OptionVisitor<'a, 'p> {
    enum_info: &'static EnumInfo,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut ValueProcessor<'p>>,
}

impl<'de> Visitor<'de> for OptionVisitor<'_, '_> {
    type Value = DynamicEnum;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected option value of type ")?;
        formatter.write_str(self.enum_info.type_path())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mut option = DynamicEnum::default();
        option.set_variant("None", ());
        Ok(option)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let variant_info = self.enum_info.variant("Some").unwrap();
        match variant_info {
            VariantInfo::Tuple(tuple_info) if tuple_info.field_len() == 1 => {
                let field = tuple_info.field_at(0).unwrap();
                let registration =
                    get_registration(field.type_id(), field.type_path(), self.registry)?;
                let de = TypedReflectDeserializer {
                    registration,
                    registry: self.registry,
                    processor: self.processor,
                };
                let mut value = DynamicTuple::default();
                value.insert_boxed(de.deserialize(deserializer)?);
                let mut option = DynamicEnum::default();
                option.set_variant("Some", value);
                Ok(option)
            }
            info => Err(Error::custom(format_args!(
                "invalid variant, expected `Some` but got `{}`",
                info.name()
            ))),
        }
    }
}

fn visit_struct<'de, T, V>(
    map: &mut V,
    info: &'static T,
    registration: &TypeRegistration,
    registry: &TypeRegistry,
    mut processor: Option<&mut ValueProcessor>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: MapAccess<'de>,
{
    let mut dynamic_struct = DynamicStruct::default();
    while let Some(Ident(key)) = map.next_key::<Ident>()? {
        let field = info.get_field(&key).ok_or_else(|| {
            let fields = info.iter_fields().map(|field| field.name());
            Error::custom(format_args!(
                "unknown field `{}`, expected one of {:?}",
                key,
                ExpectedValues(fields.collect())
            ))
        })?;
        let registration = get_registration(field.type_id(), field.type_path(), registry)?;
        let value = map.next_value_seed(TypedReflectDeserializer {
            registration,
            registry,
            #[expect(clippy::needless_option_as_deref, reason = "we are reborrowing")]
            processor: processor.as_deref_mut(),
        })?;
        dynamic_struct.insert_boxed(&key, value);
    }

    if let Some(serialization_data) = registration.data::<SerializationData>() {
        for (skipped_index, skipped_field) in serialization_data.iter_skipped() {
            let Some(field) = info.field_at(*skipped_index) else {
                continue;
            };
            dynamic_struct.insert_boxed(field.name(), skipped_field.generate_default());
        }
    }

    Ok(dynamic_struct)
}

fn visit_tuple<'de, T, V>(
    seq: &mut V,
    info: &T,
    registration: &TypeRegistration,
    registry: &TypeRegistry,
    mut processor: Option<&mut ValueProcessor>,
) -> Result<DynamicTuple, V::Error>
where
    T: TupleLikeInfo + Container,
    V: SeqAccess<'de>,
{
    let mut tuple = DynamicTuple::default();

    let len = info.get_field_len();

    if len == 0 {
        // Handle empty tuple/tuple struct
        return Ok(tuple);
    }

    let serialization_data = registration.data::<SerializationData>();

    for index in 0..len {
        if let Some(value) = serialization_data.and_then(|data| data.generate_default(index)) {
            tuple.insert_boxed(value);
            continue;
        }

        let value = seq
            .next_element_seed(TypedReflectDeserializer {
                registration: info.get_field_registration(index, registry)?,
                registry,
                #[expect(clippy::needless_option_as_deref, reason = "we are reborrowing")]
                processor: processor.as_deref_mut(),
            })?
            .ok_or_else(|| Error::invalid_length(index, &len.to_string().as_str()))?;
        tuple.insert_boxed(value);
    }

    Ok(tuple)
}

fn visit_struct_seq<'de, T, V>(
    seq: &mut V,
    info: &T,
    registration: &TypeRegistration,
    registry: &TypeRegistry,
    mut processor: Option<&mut ValueProcessor>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo + Container,
    V: SeqAccess<'de>,
{
    let mut dynamic_struct = DynamicStruct::default();

    let len = info.get_field_len();

    if len == 0 {
        // Handle unit structs
        return Ok(dynamic_struct);
    }

    let serialization_data = registration.data::<SerializationData>();

    for index in 0..len {
        let name = info.field_at(index).unwrap().name();

        if serialization_data
            .map(|data| data.is_field_skipped(index))
            .unwrap_or_default()
        {
            if let Some(value) = serialization_data.unwrap().generate_default(index) {
                dynamic_struct.insert_boxed(name, value);
            }
            continue;
        }

        let value = seq
            .next_element_seed(TypedReflectDeserializer {
                registration: info.get_field_registration(index, registry)?,
                registry,
                #[expect(clippy::needless_option_as_deref, reason = "we are reborrowing")]
                processor: processor.as_deref_mut(),
            })?
            .ok_or_else(|| Error::invalid_length(index, &len.to_string().as_str()))?;
        dynamic_struct.insert_boxed(name, value);
    }

    Ok(dynamic_struct)
}

fn get_registration<'a, E: Error>(
    type_id: TypeId,
    type_path: &str,
    registry: &'a TypeRegistry,
) -> Result<&'a TypeRegistration, E> {
    let registration = registry.get(type_id).ok_or_else(|| {
        Error::custom(format_args!("no registration found for type `{type_path}`"))
    })?;
    Ok(registration)
}
