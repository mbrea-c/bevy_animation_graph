//! Copy of https://github.com/bevyengine/bevy/blob/release-0.14.2/crates/bevy_reflect/src/serde/ser.rs
//! with https://github.com/bevyengine/bevy/pull/15548 pseudo-backported to it
//! (specialized for our own needs in this crate).

use bevy::reflect::serde::{Serializable, SerializationData};
use bevy::reflect::{
    Array, Enum, List, Map, Reflect, ReflectRef, ReflectSerialize, Struct, Tuple, TupleStruct,
    TypeInfo, TypeRegistry, VariantInfo, VariantType,
};
use serde::ser::{
    Error, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant,
};
use serde::Serializer;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};

fn get_serializable<'a, E: Error>(
    reflect_value: &'a dyn Reflect,
    type_registry: &TypeRegistry,
) -> Result<Serializable<'a>, E> {
    let info = reflect_value.get_represented_type_info().ok_or_else(|| {
        Error::custom(format_args!(
            "Type '{}' does not represent any type",
            reflect_value.reflect_type_path(),
        ))
    })?;

    let reflect_serialize = type_registry
        .get_type_data::<ReflectSerialize>(info.type_id())
        .ok_or_else(|| {
            Error::custom(format_args!(
                "Type '{}' did not register ReflectSerialize",
                info.type_path(),
            ))
        })?;
    Ok(reflect_serialize.get_serializable(reflect_value))
}

/// A serializer for reflected types whose type will be known during deserialization.
///
/// This is the serializer counterpart to [`TypedReflectDeserializer`].
///
/// See [`ReflectSerializer`] for a serializer that serializes an unknown type.
///
/// # Output
///
/// Since the type is expected to be known during deserialization,
/// this serializer will not output any additional type information,
/// such as the [type path].
///
/// Instead, it will output just the serialized data.
///
/// # Example
///
/// ```ignore
/// # use bevy_reflect::prelude::*;
/// # use bevy_reflect::{TypeRegistry, serde::TypedReflectSerializer};
/// #[derive(Reflect, PartialEq, Debug)]
/// #[type_path = "my_crate"]
/// struct MyStruct {
///   value: i32
/// }
///
/// let mut registry = TypeRegistry::default();
/// registry.register::<MyStruct>();
///
/// let input = MyStruct { value: 123 };
///
/// let reflect_serializer = TypedReflectSerializer::new(&input, &registry);
/// let output = ron::to_string(&reflect_serializer).unwrap();
///
/// assert_eq!(output, r#"(value:123)"#);
/// ```
///
/// [`TypedReflectDeserializer`]: crate::serde::TypedReflectDeserializer
/// [type path]: crate::TypePath::type_path
pub struct TypedReflectSerializer<'a, P> {
    value: &'a dyn Reflect,
    registry: &'a TypeRegistry,
    processor: Option<&'a P>,
}

pub trait ReflectSerializerProcessor {
    fn try_serialize<S>(
        &self,
        value: &dyn Reflect,
        registry: &TypeRegistry,
        serializer: S,
    ) -> Result<Result<S::Ok, S>, S::Error>
    where
        S: Serializer;
}

impl<'a, P: ReflectSerializerProcessor> TypedReflectSerializer<'a, P> {
    pub fn new(
        value: &'a dyn Reflect,
        registry: &'a TypeRegistry,
        processor: Option<&'a P>,
    ) -> Self {
        TypedReflectSerializer {
            value,
            registry,
            processor,
        }
    }
}

impl<'a, P: ReflectSerializerProcessor> Serialize for TypedReflectSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // First, check if our processor wants to serialize this type
        // This takes priority over any other serialization operations
        let serializer = if let Some(processor) = self.processor {
            match processor.try_serialize(self.value, self.registry, serializer) {
                Ok(Ok(value)) => {
                    return Ok(value);
                }
                Err(err) => {
                    return Err(err);
                }
                Ok(Err(serializer)) => serializer,
            }
        } else {
            serializer
        };

        // Handle both Value case and types that have a custom `Serialize`
        let serializable = get_serializable::<S::Error>(self.value, self.registry);
        if let Ok(serializable) = serializable {
            return serializable.borrow().serialize(serializer);
        }

        match self.value.reflect_ref() {
            ReflectRef::Struct(value) => StructSerializer {
                struct_value: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::TupleStruct(value) => TupleStructSerializer {
                tuple_struct: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Tuple(value) => TupleSerializer {
                tuple: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::List(value) => ListSerializer {
                list: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Array(value) => ArraySerializer {
                array: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Map(value) => MapSerializer {
                map: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Enum(value) => EnumSerializer {
                enum_value: value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Value(_) => Err(serializable.err().unwrap()),
        }
    }
}

pub struct ReflectValueSerializer<'a> {
    pub registry: &'a TypeRegistry,
    pub value: &'a dyn Reflect,
}

impl<'a> Serialize for ReflectValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        get_serializable::<S::Error>(self.value, self.registry)?
            .borrow()
            .serialize(serializer)
    }
}

pub struct StructSerializer<'a, P> {
    pub struct_value: &'a dyn Struct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for StructSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let type_info = self
            .struct_value
            .get_represented_type_info()
            .ok_or_else(|| {
                Error::custom(format_args!(
                    "cannot get type info for {}",
                    self.struct_value.reflect_type_path()
                ))
            })?;

        let struct_info = match type_info {
            TypeInfo::Struct(struct_info) => struct_info,
            info => {
                return Err(Error::custom(format_args!(
                    "expected struct type but received {info:?}"
                )));
            }
        };

        let serialization_data = self
            .registry
            .get(type_info.type_id())
            .and_then(|registration| registration.data::<SerializationData>());
        let ignored_len = serialization_data.map(|data| data.len()).unwrap_or(0);
        let mut state = serializer.serialize_struct(
            struct_info.type_path_table().ident().unwrap(),
            self.struct_value.field_len() - ignored_len,
        )?;

        for (index, value) in self.struct_value.iter_fields().enumerate() {
            if serialization_data
                .map(|data| data.is_field_skipped(index))
                .unwrap_or(false)
            {
                continue;
            }
            let key = struct_info.field_at(index).unwrap().name();
            state.serialize_field(
                key,
                &TypedReflectSerializer::new(value, self.registry, self.processor),
            )?;
        }
        state.end()
    }
}

pub struct TupleStructSerializer<'a, P> {
    pub tuple_struct: &'a dyn TupleStruct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for TupleStructSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let type_info = self
            .tuple_struct
            .get_represented_type_info()
            .ok_or_else(|| {
                Error::custom(format_args!(
                    "cannot get type info for {}",
                    self.tuple_struct.reflect_type_path()
                ))
            })?;

        let tuple_struct_info = match type_info {
            TypeInfo::TupleStruct(tuple_struct_info) => tuple_struct_info,
            info => {
                return Err(Error::custom(format_args!(
                    "expected tuple struct type but received {info:?}"
                )));
            }
        };

        let serialization_data = self
            .registry
            .get(type_info.type_id())
            .and_then(|registration| registration.data::<SerializationData>());
        let ignored_len = serialization_data.map(|data| data.len()).unwrap_or(0);
        let mut state = serializer.serialize_tuple_struct(
            tuple_struct_info.type_path_table().ident().unwrap(),
            self.tuple_struct.field_len() - ignored_len,
        )?;

        for (index, value) in self.tuple_struct.iter_fields().enumerate() {
            if serialization_data
                .map(|data| data.is_field_skipped(index))
                .unwrap_or(false)
            {
                continue;
            }
            state.serialize_field(&TypedReflectSerializer::new(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}

pub struct EnumSerializer<'a, P> {
    pub enum_value: &'a dyn Enum,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for EnumSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let type_info = self.enum_value.get_represented_type_info().ok_or_else(|| {
            Error::custom(format_args!(
                "cannot get type info for {}",
                self.enum_value.reflect_type_path()
            ))
        })?;

        let enum_info = match type_info {
            TypeInfo::Enum(enum_info) => enum_info,
            info => {
                return Err(Error::custom(format_args!(
                    "expected enum type but received {info:?}"
                )));
            }
        };

        let enum_name = enum_info.type_path_table().ident().unwrap();
        let variant_index = self.enum_value.variant_index() as u32;
        let variant_info = enum_info
            .variant_at(variant_index as usize)
            .ok_or_else(|| {
                Error::custom(format_args!(
                    "variant at index `{variant_index}` does not exist",
                ))
            })?;
        let variant_name = variant_info.name();
        let variant_type = self.enum_value.variant_type();
        let field_len = self.enum_value.field_len();

        match variant_type {
            VariantType::Unit => {
                if type_info.type_path_table().module_path() == Some("core::option")
                    && type_info.type_path_table().ident() == Some("Option")
                {
                    serializer.serialize_none()
                } else {
                    serializer.serialize_unit_variant(enum_name, variant_index, variant_name)
                }
            }
            VariantType::Struct => {
                let struct_info = match variant_info {
                    VariantInfo::Struct(struct_info) => struct_info,
                    info => {
                        return Err(Error::custom(format_args!(
                            "expected struct variant type but received {info:?}",
                        )));
                    }
                };

                let mut state = serializer.serialize_struct_variant(
                    enum_name,
                    variant_index,
                    variant_name,
                    field_len,
                )?;
                for (index, field) in self.enum_value.iter_fields().enumerate() {
                    let field_info = struct_info.field_at(index).unwrap();
                    state.serialize_field(
                        field_info.name(),
                        &TypedReflectSerializer::new(field.value(), self.registry, self.processor),
                    )?;
                }
                state.end()
            }
            VariantType::Tuple if field_len == 1 => {
                let field = self.enum_value.field_at(0).unwrap();

                if type_info.type_path_table().module_path() == Some("core::option")
                    && type_info.type_path_table().ident() == Some("Option")
                {
                    serializer.serialize_some(&TypedReflectSerializer::new(
                        field,
                        self.registry,
                        self.processor,
                    ))
                } else {
                    serializer.serialize_newtype_variant(
                        enum_name,
                        variant_index,
                        variant_name,
                        &TypedReflectSerializer::new(field, self.registry, self.processor),
                    )
                }
            }
            VariantType::Tuple => {
                let mut state = serializer.serialize_tuple_variant(
                    enum_name,
                    variant_index,
                    variant_name,
                    field_len,
                )?;
                for field in self.enum_value.iter_fields() {
                    state.serialize_field(&TypedReflectSerializer::new(
                        field.value(),
                        self.registry,
                        self.processor,
                    ))?;
                }
                state.end()
            }
        }
    }
}

pub struct TupleSerializer<'a, P> {
    pub tuple: &'a dyn Tuple,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for TupleSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_tuple(self.tuple.field_len())?;

        for value in self.tuple.iter_fields() {
            state.serialize_element(&TypedReflectSerializer::new(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}

pub struct MapSerializer<'a, P> {
    pub map: &'a dyn Map,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for MapSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.map.len()))?;
        for (key, value) in self.map.iter() {
            state.serialize_entry(
                &TypedReflectSerializer::new(key, self.registry, self.processor),
                &TypedReflectSerializer::new(value, self.registry, self.processor),
            )?;
        }
        state.end()
    }
}

pub struct ListSerializer<'a, P> {
    pub list: &'a dyn List,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for ListSerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.list.len()))?;
        for value in self.list.iter() {
            state.serialize_element(&TypedReflectSerializer::new(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}

pub struct ArraySerializer<'a, P> {
    pub array: &'a dyn Array,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<'a, P: ReflectSerializerProcessor> Serialize for ArraySerializer<'a, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_tuple(self.array.len())?;
        for value in self.array.iter() {
            state.serialize_element(&TypedReflectSerializer::new(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
