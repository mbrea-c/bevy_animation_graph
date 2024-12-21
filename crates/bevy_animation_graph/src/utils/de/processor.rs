use bevy::reflect::{PartialReflect, TypeRegistration, TypeRegistry};

pub trait ReflectDeserializerProcessor {
    fn try_deserialize<'de, D>(
        &mut self,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error>
    where
        D: serde::Deserializer<'de>;
}

impl ReflectDeserializerProcessor for () {
    fn try_deserialize<'de, D>(
        &mut self,
        _registration: &TypeRegistration,
        _registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Err(deserializer))
    }
}
