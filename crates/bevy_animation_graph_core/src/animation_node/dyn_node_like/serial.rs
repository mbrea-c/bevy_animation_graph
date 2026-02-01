use core::fmt;

use bevy::{
    asset::{AssetPath, LoadContext, ReflectHandle},
    platform::collections::HashMap,
    reflect::{
        PartialReflect, ReflectFromReflect, TypeRegistration, TypeRegistry,
        serde::{
            ReflectDeserializer, ReflectDeserializerProcessor, ReflectSerializerProcessor,
            TypedReflectSerializer,
        },
    },
};
use serde::{
    Serialize,
    de::{self, DeserializeSeed, Visitor},
};

use crate::animation_node::{NodeLike, ReflectNodeLike, dyn_node_like::DynNodeLike};

struct HandleDeserializeProcessor<'a, 'b> {
    load_context: &'a mut LoadContext<'b>,
}

impl ReflectDeserializerProcessor for HandleDeserializeProcessor<'_, '_> {
    fn try_deserialize<'de, D>(
        &mut self,
        registration: &TypeRegistration,
        _registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AssetPathVisitor;

        impl<'de> Visitor<'de> for AssetPathVisitor {
            type Value = AssetPath<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("asset path")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AssetPath::try_parse(v)
                    .map_err(|err| de::Error::custom(format!("not a valid asset path: {err:#}")))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AssetPath::try_parse(&v)
                    .map(AssetPath::into_owned)
                    .map_err(|err| de::Error::custom(format!("not a valid asset path: {err:#}")))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AssetPath::try_parse(v)
                    .map(AssetPath::into_owned)
                    .map_err(|err| de::Error::custom(format!("not a valid asset path: {err:#}")))
            }
        }

        let Some(handle_info) = registration.data::<ReflectHandle>() else {
            return Ok(Err(deserializer));
        };
        let asset_type_id = handle_info.asset_type_id();
        let asset_path = deserializer.deserialize_str(AssetPathVisitor)?;
        let untyped_handle = self
            .load_context
            .loader()
            .with_dynamic_type(asset_type_id)
            .load(asset_path);
        let typed_handle = handle_info.typed(untyped_handle);
        Ok(Ok(typed_handle.into_partial_reflect()))
    }
}

pub struct DynNodeLikeDeserializer<'a, 'b> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'b>,
}

impl<'de> DeserializeSeed<'de> for DynNodeLikeDeserializer<'_, '_> {
    type Value = DynNodeLike;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct NodeInnerDeserializer<'a, 'b> {
            type_registry: &'a TypeRegistry,
            load_context: &'a mut LoadContext<'b>,
        }

        impl<'de> DeserializeSeed<'de> for NodeInnerDeserializer<'_, '_> {
            type Value = Box<dyn NodeLike>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                let Self {
                    type_registry,
                    load_context,
                } = self;

                let mut processor = HandleDeserializeProcessor { load_context };
                let reflect_deserializer =
                    ReflectDeserializer::with_processor(type_registry, &mut processor);
                let inner = reflect_deserializer.deserialize(deserializer)?;

                let type_info = inner
                    .get_represented_type_info()
                    .ok_or_else(|| de::Error::custom("value is not a concrete type"))?;
                let ty = type_info.type_path();
                let type_registration = type_registry
                    .get(type_info.type_id())
                    .ok_or_else(|| de::Error::custom(format!("`{ty}` is not registered")))?;
                let node_like = type_registration
                    .data::<ReflectNodeLike>()
                    .ok_or(de::Error::custom(format!("`{ty}` is not a `NodeLike`")))?;
                let from_reflect =
                    type_registration
                        .data::<ReflectFromReflect>()
                        .ok_or(de::Error::custom(format!(
                            "`{ty}` cannot be created from reflection"
                        )))?;
                let inner = from_reflect.from_reflect(inner.as_partial_reflect()).unwrap_or_else(|| {
                    panic!(
                        "from reflect mismatch - reflecting from a `{}` into a `{ty}` - value: {inner:?}",
                        inner.reflect_type_path()
                    )
                });
                let inner = node_like.get_boxed(inner).unwrap_or_else(|value| {
                    panic!("value of type `{ty}` should be a `NodeLike` - value: {value:?}")
                });

                Ok(inner)
            }
        }

        let inner = NodeInnerDeserializer {
            type_registry: self.type_registry,
            load_context: self.load_context,
        }
        .deserialize(deserializer)?;
        Ok(DynNodeLike(inner))

        // #[doc(hidden)]
        // struct Visitor<'de, 'a, 'b> {
        //     lifetime: PhantomData<&'de ()>,
        //     type_registry: &'a TypeRegistry,
        //     load_context: &'a mut LoadContext<'b>,
        // }

        // impl<'de> de::Visitor<'de> for Visitor<'de, '_, '_> {
        //     type Value = DynNodeLike;

        //     fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        //         core::fmt::Formatter::write_str(formatter, "tuple struct DynNodeLike")
        //     }

        //     #[inline]
        //     fn visit_newtype_struct<E>(self, e: E) -> Result<Self::Value, E::Error>
        //     where
        //         E: de::Deserializer<'de>,
        //     {
        //         let inner = NodeInnerDeserializer {
        //             type_registry: self.type_registry,
        //             load_context: self.load_context,
        //         }
        //         .deserialize(e)?;
        //         Ok(DynNodeLike(inner))
        //     }

        //     #[inline]
        //     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        //     where
        //         A: de::SeqAccess<'de>,
        //     {
        //         let inner = seq
        //             .next_element_seed(NodeInnerDeserializer {
        //                 type_registry: self.type_registry,
        //                 load_context: self.load_context,
        //             })?
        //             .ok_or(de::Error::invalid_length(
        //                 0usize,
        //                 &"tuple struct DynNodeLike with 1 element",
        //             ))?;

        //         Ok(DynNodeLike(inner))
        //     }
        // }

        // serde::Deserializer::deserialize_newtype_struct(
        //     deserializer,
        //     "DynNodeLike",
        //     Visitor {
        //         lifetime: PhantomData,
        //         type_registry: self.type_registry,
        //         load_context: self.load_context,
        //     },
        // )
    }
}

pub struct DynNodeLikeSerializer<'a> {
    pub type_registry: &'a TypeRegistry,
    pub value: DynNodeLike,
}

impl DynNodeLikeSerializer<'_> {
    pub fn new<'a>(
        node: &DynNodeLike,
        type_registry: &'a TypeRegistry,
    ) -> DynNodeLikeSerializer<'a> {
        DynNodeLikeSerializer {
            type_registry,
            value: node.clone(),
        }
    }
}

impl Serialize for DynNodeLikeSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        struct HandleProcessor;

        impl ReflectSerializerProcessor for HandleProcessor {
            fn try_serialize<S>(
                &self,
                value: &dyn PartialReflect,
                registry: &TypeRegistry,
                serializer: S,
            ) -> Result<Result<S::Ok, S>, S::Error>
            where
                S: serde::Serializer,
            {
                let Some(value) = value.try_as_reflect() else {
                    return Ok(Err(serializer));
                };

                let type_id = value.reflect_type_info().type_id();
                let Some(untyped_handle) = registry
                    .get_type_data::<ReflectHandle>(type_id)
                    .and_then(|reflect_handle| {
                        reflect_handle.downcast_handle_untyped(value.as_any())
                    })
                else {
                    return Ok(Err(serializer));
                };

                let Some(path) = untyped_handle.path() else {
                    return Err(serde::ser::Error::custom(
                        "asset handle does not have a path",
                    ));
                };
                let Some(path) = path.path().to_str() else {
                    return Err(serde::ser::Error::custom(
                        "asset handle has a non-UTF-8 path",
                    ));
                };

                serializer.serialize_str(path).map(Ok)
            }
        }

        let type_path = self
            .type_registry
            .get_type_info(self.value.0.type_id())
            .map(|t| t.type_path())
            .ok_or(serde::ser::Error::custom(format!(
                "no type registration for `{}`",
                self.value.0.reflect_type_path()
            )))?;

        let processor = HandleProcessor;
        let reflect_serialzer = TypedReflectSerializer::with_processor(
            self.value.0.as_partial_reflect(),
            self.type_registry,
            &processor,
        );
        let mut inner = HashMap::new();
        inner.insert(type_path, reflect_serialzer);

        inner.serialize(serializer)
    }
}
