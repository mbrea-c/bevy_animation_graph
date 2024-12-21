use bevy::reflect::{PartialReflect, ReflectRef, TypeRegistry};
use serde::{ser::SerializeMap, Serialize, Serializer};

use super::{
    arrays::ArraySerializer, custom_serialization::try_custom_serialize, enums::EnumSerializer,
    error_utils::make_custom_error, lists::ListSerializer, maps::MapSerializer,
    sets::SetSerializer, structs::StructSerializer, tuple_structs::TupleStructSerializer,
    tuples::TupleSerializer,
};

pub trait ReflectSerializerProcessor {
    fn try_serialize<S>(
        &self,
        value: &dyn PartialReflect,
        registry: &TypeRegistry,
        serializer: S,
    ) -> Result<Result<S::Ok, S>, S::Error>
    where
        S: Serializer;
}

impl ReflectSerializerProcessor for () {
    fn try_serialize<S>(
        &self,
        _value: &dyn PartialReflect,
        _registry: &TypeRegistry,
        serializer: S,
    ) -> Result<Result<S::Ok, S>, S::Error>
    where
        S: Serializer,
    {
        Ok(Err(serializer))
    }
}

pub struct ReflectSerializer<'a, P = ()> {
    value: &'a dyn PartialReflect,
    registry: &'a TypeRegistry,
    processor: Option<&'a P>,
}

impl<P: ReflectSerializerProcessor> Serialize for ReflectSerializer<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(1))?;
        state.serialize_entry(
            self.value
                .get_represented_type_info()
                .ok_or_else(|| {
                    if self.value.is_dynamic() {
                        make_custom_error(format_args!(
                            "cannot serialize dynamic value without represented type: `{}`",
                            self.value.reflect_type_path()
                        ))
                    } else {
                        make_custom_error(format_args!(
                            "cannot get type info for `{}`",
                            self.value.reflect_type_path()
                        ))
                    }
                })?
                .type_path(),
            &TypedReflectSerializer::new_internal(self.value, self.registry, self.processor),
        )?;
        state.end()
    }
}

pub struct TypedReflectSerializer<'a, P = ()> {
    value: &'a dyn PartialReflect,
    registry: &'a TypeRegistry,
    processor: Option<&'a P>,
}

impl<'a, P> TypedReflectSerializer<'a, P> {
    /// Creates a serializer with a processor.
    ///
    /// If you do not need any custom logic for handling certain values, use
    /// [`new`].
    ///
    /// [`new`]: Self::new
    pub fn with_processor(
        value: &'a dyn PartialReflect,
        registry: &'a TypeRegistry,
        processor: &'a P,
    ) -> Self {
        Self {
            value,
            registry,
            processor: Some(processor),
        }
    }

    /// An internal constructor for creating a serializer without resetting the type info stack.
    pub(super) fn new_internal(
        value: &'a dyn PartialReflect,
        registry: &'a TypeRegistry,
        processor: Option<&'a P>,
    ) -> Self {
        Self {
            value,
            registry,
            processor,
        }
    }
}

impl<P: ReflectSerializerProcessor> Serialize for TypedReflectSerializer<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // First, check if our processor wants to serialize this type
        // This takes priority over any other serialization operations
        let serializer = if let Some(processor) = self.processor {
            match processor.try_serialize(self.value, self.registry, serializer) {
                Ok(Ok(value)) => {
                    return Ok(value);
                }
                Err(err) => {
                    return Err(make_custom_error(err));
                }
                Ok(Err(serializer)) => serializer,
            }
        } else {
            serializer
        };

        // Handle both Value case and types that have a custom `Serialize`
        let (serializer, error) = match try_custom_serialize(self.value, self.registry, serializer)
        {
            Ok(result) => return result,
            Err(value) => value,
        };

        let output = match self.value.reflect_ref() {
            ReflectRef::Struct(struct_value) => StructSerializer {
                struct_value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::TupleStruct(tuple_struct) => TupleStructSerializer {
                tuple_struct,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Tuple(tuple) => TupleSerializer {
                tuple,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::List(list) => ListSerializer {
                list,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Array(array) => ArraySerializer {
                array,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Map(map) => MapSerializer {
                map,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Set(set) => SetSerializer {
                set,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Enum(enum_value) => EnumSerializer {
                enum_value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Opaque(_) => Err(error),
        };

        output
    }
}
