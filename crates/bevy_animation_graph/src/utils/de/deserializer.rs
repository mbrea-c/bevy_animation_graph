use crate::utils::de::TypeRegistrationDeserializer;
use bevy::reflect::{
    serde::SerializationData, PartialReflect, ReflectDeserialize, TypeInfo, TypeRegistration,
    TypeRegistry,
};
use core::{fmt, fmt::Formatter};
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};

use super::{
    arrays::ArrayVisitor, enums::EnumVisitor, error_utils::make_custom_error, lists::ListVisitor,
    maps::MapVisitor, options::OptionVisitor, sets::SetVisitor, structs::StructVisitor,
    tuple_structs::TupleStructVisitor, tuples::TupleVisitor, ReflectDeserializeWithRegistry,
    ReflectDeserializerProcessor,
};

pub struct ReflectDeserializer<'a, P: ReflectDeserializerProcessor = ()> {
    registry: &'a TypeRegistry,
    processor: Option<&'a mut P>,
}

impl<'de, P: ReflectDeserializerProcessor> DeserializeSeed<'de> for ReflectDeserializer<'_, P> {
    type Value = Box<dyn PartialReflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UntypedReflectDeserializerVisitor<'a, P> {
            registry: &'a TypeRegistry,
            processor: Option<&'a mut P>,
        }

        impl<'de, P: ReflectDeserializerProcessor> Visitor<'de>
            for UntypedReflectDeserializerVisitor<'_, P>
        {
            type Value = Box<dyn PartialReflect>;

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

                let value = map.next_value_seed(TypedReflectDeserializer::new_internal(
                    registration,
                    self.registry,
                    self.processor,
                ))?;

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

pub struct TypedReflectDeserializer<'a, P: ReflectDeserializerProcessor = ()> {
    registration: &'a TypeRegistration,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut P>,
}

impl<'a, P: ReflectDeserializerProcessor> TypedReflectDeserializer<'a, P> {
    /// Creates a typed deserializer with a processor.
    ///
    /// If you do not need any custom logic for handling certain types, use
    /// [`new`].
    ///
    /// [`new`]: Self::new
    pub fn with_processor(
        registration: &'a TypeRegistration,
        registry: &'a TypeRegistry,
        processor: &'a mut P,
    ) -> Self {
        Self {
            registration,
            registry,
            processor: Some(processor),
        }
    }

    /// An internal constructor for creating a deserializer without resetting the type info stack.
    pub(super) fn new_internal(
        registration: &'a TypeRegistration,
        registry: &'a TypeRegistry,
        processor: Option<&'a mut P>,
    ) -> Self {
        Self {
            registration,
            registry,
            processor,
        }
    }
}

impl<'de, P: ReflectDeserializerProcessor> DeserializeSeed<'de>
    for TypedReflectDeserializer<'_, P>
{
    type Value = Box<dyn PartialReflect>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialize_internal = || -> Result<Self::Value, D::Error> {
            // First, check if our processor wants to deserialize this type
            // This takes priority over any other deserialization operations
            let deserializer = if let Some(processor) = self.processor.as_deref_mut() {
                match processor.try_deserialize(self.registration, self.registry, deserializer) {
                    Ok(Ok(value)) => {
                        return Ok(value);
                    }
                    Err(err) => {
                        return Err(make_custom_error(err));
                    }
                    Ok(Err(deserializer)) => deserializer,
                }
            } else {
                deserializer
            };

            let type_path = self.registration.type_info().type_path();

            // Handle both Value case and types that have a custom `ReflectDeserialize`
            if let Some(deserialize_reflect) = self.registration.data::<ReflectDeserialize>() {
                let value = deserialize_reflect.deserialize(deserializer)?;
                return Ok(value.into_partial_reflect());
            }

            if let Some(deserialize_reflect) =
                self.registration.data::<ReflectDeserializeWithRegistry>()
            {
                let value = deserialize_reflect.deserialize(deserializer, self.registry)?;
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
                    let mut dynamic_tuple_struct = if tuple_struct_info.field_len() == 1
                        && self.registration.data::<SerializationData>().is_none()
                    {
                        deserializer.deserialize_newtype_struct(
                            tuple_struct_info.type_path_table().ident().unwrap(),
                            TupleStructVisitor {
                                tuple_struct_info,
                                registration: self.registration,
                                registry: self.registry,
                                processor: self.processor,
                            },
                        )?
                    } else {
                        deserializer.deserialize_tuple_struct(
                            tuple_struct_info.type_path_table().ident().unwrap(),
                            tuple_struct_info.field_len(),
                            TupleStructVisitor {
                                tuple_struct_info,
                                registration: self.registration,
                                registry: self.registry,
                                processor: self.processor,
                            },
                        )?
                    };
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
                TypeInfo::Set(set_info) => {
                    let mut dynamic_set = deserializer.deserialize_seq(SetVisitor {
                        set_info,
                        registry: self.registry,
                        processor: self.processor,
                    })?;
                    dynamic_set.set_represented_type(Some(self.registration.type_info()));
                    Ok(Box::new(dynamic_set))
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
                            processor: self.processor,
                        })?
                    } else {
                        deserializer.deserialize_enum(
                            enum_info.type_path_table().ident().unwrap(),
                            enum_info.variant_names(),
                            EnumVisitor {
                                enum_info,
                                registration: self.registration,
                                registry: self.registry,
                                processor: self.processor,
                            },
                        )?
                    };
                    dynamic_enum.set_represented_type(Some(self.registration.type_info()));
                    Ok(Box::new(dynamic_enum))
                }
                TypeInfo::Opaque(_) => {
                    // This case should already be handled
                    Err(make_custom_error(format_args!(
                        "type `{type_path}` did not register the `ReflectDeserialize` type data. For certain types, this may need to be registered manually using `register_type_data`",
                    )))
                }
            }
        };

        deserialize_internal()
    }
}
