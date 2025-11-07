use std::fmt;

use super::{AnimationGraph, Extra, pin};
use crate::prelude::{AnimationNode, DataSpec, DataValue, NodeLike, ReflectNodeLike};
use bevy::{
    asset::{AssetPath, LoadContext, ReflectHandle},
    platform::collections::HashMap,
    prelude::*,
    reflect::{
        ReflectFromReflect, TypeRegistration, TypeRegistry,
        serde::{
            ReflectDeserializer, ReflectDeserializerProcessor, ReflectSerializerProcessor,
            TypedReflectSerializer,
        },
    },
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, DeserializeSeed, IgnoredAny, Visitor},
    ser::SerializeStruct,
};

// What's up with the `AnimationNodeLoadDeserializer`?
//
// When initially deserializing the graph asset file, we don't know the type of
// node in `nodes` that we are deserializing. We figure that out after we have
// `ty`, access to the `TypeRegistry`, and the `LoadContext`. So how do we
// deserialize something we don't know the type of?
//
// The first approach is to deserialize the entire graph in one go, and store
// RON ASTs instead of the actual node value. So instead of seeing a
// `GraphNode`, we first see a `ron::Value`, and then once we know that we want
// to deserialize a `GraphNode`, we deserialize the value as that node.
//
// This would be the simplest and most understandable approach... if it wasn't
// for the fact that `ron::Value` doesn't actually store the AST properly. It
// can't handle enum variants, which means that `impl NodeLike`s would not be
// allowed to use enums, which is a massive drawback.
// (see https://github.com/ron-rs/ron/issues/496 - "round-tripping Value")
//
// The second approach is to write a custom deserializer which is almost
// identical to the `serde::Deserialize` derive macro, but we add custom logic
// specifically for `inner`. We use `DeserializeSeed` so that we can pass in the
// type registry and load context that we use. This approach *does* work
// properly, but introduces a ton of boilerplate code.
//
// To hopefully get the best of both worlds, the process for writing the
// node deserialize code below is:
// - uncomment the `AnimationNodeSerial` struct
// - expand the `Deserialize` macro output
// - use that to write the `DeserializeSeed` impl manually - try and follow it
//   as closely as possible, until we get to deserializing `inner`
// - recomment the `AnimationNodeSerial`
//
// For the graph deserialize, we do something similar, but copy the macro
// directly and just make changes (marked with `manual impl` comments), since
// there's less to change. We also leave the `AnimationGraphSerial` in, since
// it isn't exactly the same as `AnimationGraph` (could we change this?)
//
// Why, `ron`, why??

// #[derive(Deserialize)]
// struct AnimationNodeSerial {
//     name: String,
//     ty: String,
//     inner: ron::Value,
// }

pub struct AnimationNodeLoadDeserializer<'a, 'b> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'b>,
}

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

impl<'de> DeserializeSeed<'de> for AnimationNodeLoadDeserializer<'_, '_> {
    type Value = AnimationNode;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NodeInnerDeserializer<'a, 'b> {
            type_registry: &'a TypeRegistry,
            load_context: &'a mut LoadContext<'b>,
        }

        impl<'de> DeserializeSeed<'de> for NodeInnerDeserializer<'_, '_> {
            type Value = Box<dyn NodeLike>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
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

        const NAME: &str = "name";
        const INNER: &str = "inner";

        enum Field {
            Name,
            Inner,
            _Ignore,
        }

        struct FieldVisitor;

        impl Visitor<'_> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "field identifier")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    0 => Ok(Field::Name),
                    1 => Ok(Field::Inner),
                    _ => Ok(Field::_Ignore),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    NAME => Ok(Field::Name),
                    INNER => Ok(Field::Inner),
                    _ => Ok(Field::_Ignore),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ValueVisitor<'a, 'b> {
            type_registry: &'a TypeRegistry,
            load_context: &'a mut LoadContext<'b>,
        }

        impl<'de> Visitor<'de> for ValueVisitor<'_, '_> {
            type Value = AnimationNode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "struct AnimationNode")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                const INVALID_LENGTH: &str = "struct AnimationNode with 3 elements";

                let name = seq
                    .next_element::<String>()?
                    .ok_or(de::Error::invalid_length(0, &INVALID_LENGTH))?;
                let inner = seq
                    .next_element_seed(NodeInnerDeserializer {
                        type_registry: self.type_registry,
                        load_context: self.load_context,
                    })?
                    .ok_or(de::Error::invalid_length(1, &INVALID_LENGTH))?;

                Ok(AnimationNode {
                    name,
                    inner,
                    should_debug: false,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut name = None::<String>;
                let mut inner = None::<Box<dyn NodeLike>>;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field(NAME));
                            }
                            name = Some(map.next_value::<String>()?);
                        }
                        Field::Inner => {
                            if inner.is_some() {
                                return Err(de::Error::duplicate_field(INNER));
                            }
                            inner = Some(map.next_value_seed(NodeInnerDeserializer {
                                type_registry: self.type_registry,
                                load_context: self.load_context,
                            })?);
                        }
                        _ => {
                            let _ = map.next_value::<IgnoredAny>();
                        }
                    }
                }

                Ok(AnimationNode {
                    name: name.ok_or(de::Error::missing_field(NAME))?,
                    inner: inner.ok_or(de::Error::missing_field(INNER))?,
                    should_debug: false,
                })
            }
        }

        let visitor = ValueVisitor {
            type_registry: self.type_registry,
            load_context: self.load_context,
        };
        deserializer.deserialize_struct("AnimationNode", &[NAME, INNER], visitor)
    }
}

pub type NodeIdSerial = String;
pub type PinIdSerial = String;
pub type TargetPinSerial = pin::TargetPin<NodeIdSerial, PinIdSerial>;
pub type SourcePinSerial = pin::SourcePin<NodeIdSerial, PinIdSerial>;

// #[derive(Deserialize)]
pub struct AnimationGraphSerial {
    pub nodes: Vec<AnimationNode>,
    pub edges_inverted: HashMap<TargetPinSerial, SourcePinSerial>,

    pub default_parameters: HashMap<PinIdSerial, DataValue>,
    pub input_times: HashMap<PinIdSerial, ()>,
    pub output_parameters: HashMap<PinIdSerial, DataSpec>,
    pub output_time: Option<()>,

    pub extra: Extra,
}

pub struct AnimationGraphLoadDeserializer<'a, 'b> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'b>,
}

// auto-generated by macro, manually modified in "manual impl" comment blocks

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::manual_unwrap_or_default
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    // manual impl start
    impl<'de> _serde::de::DeserializeSeed<'de> for AnimationGraphLoadDeserializer<'_, '_> {
        type Value = AnimationGraphSerial;
        fn deserialize<__D>(
            self,
            __deserializer: __D,
        ) -> ::core::result::Result<Self::Value, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            struct NodesDeserializer<'a, 'b> {
                type_registry: &'a TypeRegistry,
                load_context: &'a mut LoadContext<'b>,
            }

            impl<'de> DeserializeSeed<'de> for NodesDeserializer<'_, '_> {
                type Value = Vec<AnimationNode>;

                fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct NodeVisitor<'a, 'b, 'c> {
                        de: &'c mut NodesDeserializer<'a, 'b>,
                    }

                    impl<'de> Visitor<'de> for NodeVisitor<'_, '_, '_> {
                        type Value = Vec<AnimationNode>;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            write!(formatter, "AnimationNode")
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: de::SeqAccess<'de>,
                        {
                            let mut nodes = Vec::new();
                            while let Some(node) =
                                seq.next_element_seed(AnimationNodeLoadDeserializer {
                                    type_registry: self.de.type_registry,
                                    load_context: self.de.load_context,
                                })?
                            {
                                nodes.push(node);
                            }
                            Ok(nodes)
                        }
                    }

                    deserializer.deserialize_seq(NodeVisitor { de: &mut self })
                }
            }
            // manual impl end
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __field4,
                __field5,
                __field6,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;

            impl _serde::de::Visitor<'_> for __FieldVisitor {
                type Value = __Field;
                fn expecting(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    std::fmt::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> ::core::result::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => ::core::result::Result::Ok(__Field::__field0),
                        1u64 => ::core::result::Result::Ok(__Field::__field1),
                        2u64 => ::core::result::Result::Ok(__Field::__field2),
                        3u64 => ::core::result::Result::Ok(__Field::__field3),
                        4u64 => ::core::result::Result::Ok(__Field::__field4),
                        5u64 => ::core::result::Result::Ok(__Field::__field5),
                        6u64 => ::core::result::Result::Ok(__Field::__field6),
                        _ => ::core::result::Result::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> ::core::result::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "nodes" => ::core::result::Result::Ok(__Field::__field0),
                        "edges_inverted" => ::core::result::Result::Ok(__Field::__field1),
                        "default_parameters" => ::core::result::Result::Ok(__Field::__field2),
                        "input_times" => ::core::result::Result::Ok(__Field::__field3),
                        "output_parameters" => ::core::result::Result::Ok(__Field::__field4),
                        "output_time" => ::core::result::Result::Ok(__Field::__field5),
                        "extra" => ::core::result::Result::Ok(__Field::__field6),
                        _ => ::core::result::Result::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> ::core::result::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"nodes" => ::core::result::Result::Ok(__Field::__field0),
                        b"edges_inverted" => ::core::result::Result::Ok(__Field::__field1),
                        b"default_parameters" => ::core::result::Result::Ok(__Field::__field2),
                        b"input_times" => ::core::result::Result::Ok(__Field::__field3),
                        b"output_parameters" => ::core::result::Result::Ok(__Field::__field4),
                        b"output_time" => ::core::result::Result::Ok(__Field::__field5),
                        b"extra" => ::core::result::Result::Ok(__Field::__field6),
                        _ => ::core::result::Result::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> ::core::result::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            // manual impl start - add seed fields
            #[doc(hidden)]
            struct __Visitor<'de, 'a, 'b> {
                marker: std::marker::PhantomData<AnimationGraphSerial>,
                lifetime: std::marker::PhantomData<&'de ()>,
                type_registry: &'a TypeRegistry,
                load_context: &'a mut LoadContext<'b>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de, '_, '_> {
                // manual impl end
                type Value = AnimationGraphSerial;
                fn expecting(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    std::fmt::Formatter::write_str(__formatter, "struct AnimationGraphSerial")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> core::result::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    // manual impl start - use seed fields
                    let __field0 = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        NodesDeserializer {
                            type_registry: self.type_registry,
                            load_context: self.load_context,
                        },
                    )? {
                        // manual impl end
                        Some(__value) => __value,
                        None => {
                            return core::result::Result::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        HashMap<TargetPinSerial, SourcePinSerial>,
                    >(&mut __seq)?
                    {
                        Some(__value) => __value,
                        None => {
                            return core::result::Result::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ));
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<
                        HashMap<PinIdSerial, DataValue>,
                    >(&mut __seq)?
                    {
                        Some(__value) => __value,
                        None => {
                            return core::result::Result::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ));
                        }
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<
                        HashMap<PinIdSerial, ()>,
                    >(&mut __seq)?
                    {
                        Some(__value) => __value,
                        None => {
                            return core::result::Result::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ));
                        }
                    };
                    let __field4 = match _serde::de::SeqAccess::next_element::<
                        HashMap<PinIdSerial, DataSpec>,
                    >(&mut __seq)?
                    {
                        Some(__value) => __value,
                        None => {
                            return core::result::Result::Err(_serde::de::Error::invalid_length(
                                4usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ));
                        }
                    };
                    let __field5 =
                        match _serde::de::SeqAccess::next_element::<Option<()>>(&mut __seq)? {
                            Some(__value) => __value,
                            None => {
                                return core::result::Result::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct AnimationGraphSerial with 7 elements",
                                    ),
                                );
                            }
                        };
                    let __field6 = match _serde::de::SeqAccess::next_element::<Extra>(&mut __seq)? {
                        Some(__value) => __value,
                        None => {
                            return core::result::Result::Err(_serde::de::Error::invalid_length(
                                6usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ));
                        }
                    };
                    ::core::result::Result::Ok(AnimationGraphSerial {
                        nodes: __field0,
                        edges_inverted: __field1,
                        default_parameters: __field2,
                        input_times: __field3,
                        output_parameters: __field4,
                        output_time: __field5,
                        extra: __field6,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> core::result::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: Option<Vec<AnimationNode>> = None;
                    let mut __field1: Option<HashMap<TargetPinSerial, SourcePinSerial>> = None;
                    let mut __field2: Option<HashMap<PinIdSerial, DataValue>> = None;
                    let mut __field3: Option<HashMap<PinIdSerial, ()>> = None;
                    let mut __field4: Option<HashMap<PinIdSerial, DataSpec>> = None;
                    let mut __field5: Option<Option<()>> = None;
                    let mut __field6: Option<Extra> = None;
                    while let Some(__key) = _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if Option::is_some(&__field0) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("nodes"),
                                    );
                                }
                                __field0 =
                                    // manual impl start - use seed
                                     Some(_serde::de::MapAccess::next_value_seed(
                                         &mut __map,
                                         NodesDeserializer {
                                             type_registry: self.type_registry,
                                             load_context: self.load_context,
                                         }
                                     )?);
                                // manual impl end
                            }
                            __Field::__field1 => {
                                if Option::is_some(&__field1) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "edges_inverted",
                                        ),
                                    );
                                }
                                __field1 = Some(_serde::de::MapAccess::next_value::<
                                    HashMap<TargetPinSerial, SourcePinSerial>,
                                >(&mut __map)?);
                            }
                            __Field::__field2 => {
                                if Option::is_some(&__field2) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "default_parameters",
                                        ),
                                    );
                                }
                                __field2 = Some(_serde::de::MapAccess::next_value::<
                                    HashMap<PinIdSerial, DataValue>,
                                >(&mut __map)?);
                            }
                            __Field::__field3 => {
                                if Option::is_some(&__field3) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "input_times",
                                        ),
                                    );
                                }
                                __field3 = Some(_serde::de::MapAccess::next_value::<
                                    HashMap<PinIdSerial, ()>,
                                >(&mut __map)?);
                            }
                            __Field::__field4 => {
                                if Option::is_some(&__field4) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "output_parameters",
                                        ),
                                    );
                                }
                                __field4 = Some(_serde::de::MapAccess::next_value::<
                                    HashMap<PinIdSerial, DataSpec>,
                                >(&mut __map)?);
                            }
                            __Field::__field5 => {
                                if Option::is_some(&__field5) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "output_time",
                                        ),
                                    );
                                }
                                __field5 = Some(_serde::de::MapAccess::next_value::<Option<()>>(
                                    &mut __map,
                                )?);
                            }
                            __Field::__field6 => {
                                if Option::is_some(&__field6) {
                                    return core::result::Result::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("extra"),
                                    );
                                }
                                __field6 =
                                    Some(_serde::de::MapAccess::next_value::<Extra>(&mut __map)?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        Some(__field0) => __field0,
                        // manual impl start
                        None => Default::default(),
                        // manual impl end
                    };
                    let __field1 = match __field1 {
                        Some(__field1) => __field1,
                        None => Default::default(),
                    };
                    let __field2 = match __field2 {
                        Some(__field2) => __field2,
                        None => Default::default(),
                    };
                    let __field3 = match __field3 {
                        Some(__field3) => __field3,
                        None => Default::default(),
                    };
                    let __field4 = match __field4 {
                        Some(__field4) => __field4,
                        None => Default::default(),
                    };
                    let __field5 = match __field5 {
                        Some(__field5) => __field5,
                        None => Default::default(),
                    };
                    let __field6 = match __field6 {
                        Some(__field6) => __field6,
                        None => Default::default(),
                    };
                    ::core::result::Result::Ok(AnimationGraphSerial {
                        nodes: __field0,
                        edges_inverted: __field1,
                        default_parameters: __field2,
                        input_times: __field3,
                        output_parameters: __field4,
                        output_time: __field5,
                        extra: __field6,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &[&str] = &[
                "nodes",
                "edges_inverted",
                "default_parameters",
                "input_times",
                "output_parameters",
                "output_time",
                "extra",
            ];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "AnimationGraphSerial",
                FIELDS,
                __Visitor {
                    marker: std::marker::PhantomData::<AnimationGraphSerial>,
                    lifetime: std::marker::PhantomData,
                    // manual impl start - add seed fields
                    load_context: self.load_context,
                    type_registry: self.type_registry,
                    // manual impl end
                },
            )
        }
    }
};

// For serialization, we make an `AnimationNode` wrapper that contains
// a reference to the type registry. Then, we wrap that new type in an
// `AnimationGraphSerializer` type that mirrors the structure of `AnimationGraph`.
// We only need to write custom serialization logic for the `AnimationNode` serializer type,
// we get the `AnimationGraph` serialization "for free".

pub struct AnimationNodeSerializer<'a> {
    pub type_registry: &'a TypeRegistry,
    pub name: String,
    pub inner: Box<dyn NodeLike>,
}

impl AnimationNodeSerializer<'_> {
    pub fn new<'a>(
        node: &AnimationNode,
        type_registry: &'a TypeRegistry,
    ) -> AnimationNodeSerializer<'a> {
        AnimationNodeSerializer {
            type_registry,
            name: node.name.clone(),
            inner: node.inner.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct AnimationGraphSerializer<'a> {
    pub nodes: Vec<AnimationNodeSerializer<'a>>,
    pub edges_inverted: HashMap<TargetPinSerial, SourcePinSerial>,

    pub default_parameters: HashMap<PinIdSerial, DataValue>,
    pub input_times: HashMap<PinIdSerial, ()>,
    pub output_parameters: HashMap<PinIdSerial, DataSpec>,
    pub output_time: Option<()>,

    pub extra: Extra,
}

impl AnimationGraphSerializer<'_> {
    pub fn new<'a>(
        graph: &AnimationGraph,
        type_registry: &'a TypeRegistry,
    ) -> AnimationGraphSerializer<'a> {
        let mut serial = AnimationGraphSerializer {
            nodes: Vec::new(),
            edges_inverted: graph.edges.clone(),
            default_parameters: graph.default_parameters.clone(),
            input_times: graph.input_times.clone(),
            output_parameters: graph.output_parameters.clone(),
            output_time: graph.output_time,
            extra: graph.extra.clone(),
        };

        for node in graph.nodes.values() {
            serial
                .nodes
                .push(AnimationNodeSerializer::new(node, type_registry));
        }

        serial
    }
}

impl Serialize for AnimationNodeSerializer<'_> {
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

        let mut state = serializer.serialize_struct("AnimationNodeSerializer", 3)?;

        state.serialize_field("name", &self.name)?;

        let type_path = self
            .type_registry
            .get_type_info(self.inner.type_id())
            .map(|t| t.type_path())
            .ok_or(serde::ser::Error::custom(format!(
                "no type registration for `{}`",
                self.inner.reflect_type_path()
            )))?;

        let processor = HandleProcessor;
        let reflect_serialzer = TypedReflectSerializer::with_processor(
            self.inner.as_partial_reflect(),
            self.type_registry,
            &processor,
        );
        let mut inner = HashMap::new();
        inner.insert(type_path, reflect_serialzer);
        state.serialize_field("inner", &inner)?;

        state.end()
    }
}
