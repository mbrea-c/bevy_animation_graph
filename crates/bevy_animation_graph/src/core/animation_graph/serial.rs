use std::fmt;

use super::{pin, Extra};
use crate::{
    prelude::{AnimationNode, DataSpec, DataValue, NodeLike, OrderedMap, ReflectNodeLike},
    utils::reflect_de::{TypedReflectDeserializer, ValueProcessor},
};
use bevy::{
    asset::{AssetPath, LoadContext, ReflectHandle},
    reflect::{Reflect, ReflectFromReflect, TypeRegistration, TypeRegistry},
    utils::HashMap,
};
use serde::{
    de::{self, DeserializeSeed, IgnoredAny, Visitor},
    Deserialize, Deserializer,
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

fn deserialize_handle(
    registration: &TypeRegistration,
    deserializer: &mut dyn bevy::reflect::erased_serde::Deserializer,
    load_context: &mut LoadContext,
) -> Result<Box<dyn Reflect>, bevy::reflect::erased_serde::Error> {
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
            AssetPath::try_parse(&v.to_owned())
                .map(AssetPath::into_owned)
                .map_err(|err| de::Error::custom(format!("not a valid asset path: {err:#}")))
        }
    }

    let handle_info = registration.data::<ReflectHandle>().unwrap();
    let asset_type_id = handle_info.asset_type_id();
    let asset_path = deserializer.deserialize_str(AssetPathVisitor)?;
    let untyped_handle = load_context
        .loader()
        .with_asset_type_id(asset_type_id)
        .untyped()
        .load(asset_path);
    // this is actually a `Handle<LoadedUntypedAsset>`, not a `Handle<T>`
    // we'll correct that in the AnimationGraphLoader...
    Ok(Box::new(untyped_handle))
}

impl<'de> DeserializeSeed<'de> for AnimationNodeLoadDeserializer<'_, '_> {
    type Value = AnimationNode;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NodeInnerDeserializer<'a, 'b, 'c> {
            type_registry: &'a TypeRegistry,
            load_context: &'a mut LoadContext<'b>,
            ty: &'c str,
        }

        impl<'de> DeserializeSeed<'de> for NodeInnerDeserializer<'_, '_, '_> {
            type Value = Box<dyn NodeLike>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let Self {
                    type_registry,
                    load_context,
                    ty,
                } = self;

                let type_registration =
                    type_registry
                        .get_with_type_path(self.ty)
                        .ok_or(de::Error::custom(format!(
                            "no type registration for `{ty}`"
                        )))?;
                let node_like = type_registration
                    .data::<ReflectNodeLike>()
                    .ok_or(de::Error::custom(format!("`{ty}` is not a `NodeLike`")))?;
                let from_reflect =
                    type_registration
                        .data::<ReflectFromReflect>()
                        .ok_or(de::Error::custom(format!(
                            "`{ty}` cannot be created from reflection"
                        )))?;

                let mut processor = ValueProcessor {
                    can_deserialize: Box::new(|registration| {
                        registration.data::<ReflectHandle>().is_some()
                    }),
                    deserialize: Box::new(|registration, deserializer| {
                        deserialize_handle(registration, deserializer, load_context)
                    }),
                };

                let reflect_deserializer = TypedReflectDeserializer::new_with_processor(
                    type_registration,
                    type_registry,
                    &mut processor,
                );
                let inner = reflect_deserializer.deserialize(deserializer)?;

                let inner = from_reflect.from_reflect(&*inner).unwrap_or_else(|| {
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
        const TY: &str = "ty";
        const INNER: &str = "inner";

        enum Field {
            Name,
            Ty,
            Inner,
            _Ignore,
        }

        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
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
                    1 => Ok(Field::Ty),
                    2 => Ok(Field::Inner),
                    _ => Ok(Field::_Ignore),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    NAME => Ok(Field::Name),
                    TY => Ok(Field::Ty),
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
                let ty = seq
                    .next_element::<&str>()?
                    .ok_or(de::Error::invalid_length(1, &INVALID_LENGTH))?;
                let inner = seq
                    .next_element_seed(NodeInnerDeserializer {
                        type_registry: self.type_registry,
                        load_context: self.load_context,
                        ty,
                    })?
                    .ok_or(de::Error::invalid_length(2, &INVALID_LENGTH))?;

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
                // unfortunately, this code is field-order-dependent
                // `ty` MUST be defined before `inner`
                let mut name = None::<String>;
                let mut ty = None::<&str>;
                let mut inner = None::<Box<dyn NodeLike>>;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field(NAME));
                            }
                            name = Some(map.next_value::<String>()?);
                        }
                        Field::Ty => {
                            if ty.is_some() {
                                return Err(de::Error::duplicate_field(TY));
                            }
                            ty = Some(map.next_value::<&str>()?);
                        }
                        Field::Inner => {
                            if inner.is_some() {
                                return Err(de::Error::duplicate_field(INNER));
                            }

                            inner = Some(map.next_value_seed(NodeInnerDeserializer {
                                type_registry: self.type_registry,
                                load_context: self.load_context,
                                ty: ty.ok_or(de::Error::custom(
                                    "`ty` must be defined before `inner`",
                                ))?,
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
            type_registry: &self.type_registry,
            load_context: &mut self.load_context,
        };
        deserializer.deserialize_struct("AnimationNode", &[NAME, TY, INNER], visitor)
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

    pub default_parameters: OrderedMap<PinIdSerial, DataValue>,
    pub input_times: OrderedMap<PinIdSerial, ()>,
    pub output_parameters: OrderedMap<PinIdSerial, DataSpec>,
    pub output_time: Option<()>,

    pub extra: Extra,
}

pub struct AnimationGraphLoadDeserializer<'a, 'b> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'b>,
}

// auto-generated by macro, manually modified in "manual impl" comment blocks

#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
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
        ) -> _serde::__private::Result<Self::Value, __D::Error>
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

            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        4u64 => _serde::__private::Ok(__Field::__field4),
                        5u64 => _serde::__private::Ok(__Field::__field5),
                        6u64 => _serde::__private::Ok(__Field::__field6),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "nodes" => _serde::__private::Ok(__Field::__field0),
                        "edges_inverted" => _serde::__private::Ok(__Field::__field1),
                        "default_parameters" => _serde::__private::Ok(__Field::__field2),
                        "input_times" => _serde::__private::Ok(__Field::__field3),
                        "output_parameters" => _serde::__private::Ok(__Field::__field4),
                        "output_time" => _serde::__private::Ok(__Field::__field5),
                        "extra" => _serde::__private::Ok(__Field::__field6),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"nodes" => _serde::__private::Ok(__Field::__field0),
                        b"edges_inverted" => _serde::__private::Ok(__Field::__field1),
                        b"default_parameters" => _serde::__private::Ok(__Field::__field2),
                        b"input_times" => _serde::__private::Ok(__Field::__field3),
                        b"output_parameters" => _serde::__private::Ok(__Field::__field4),
                        b"output_time" => _serde::__private::Ok(__Field::__field5),
                        b"extra" => _serde::__private::Ok(__Field::__field6),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            // manual impl start - add seed fields
            #[doc(hidden)]
            struct __Visitor<'de, 'a, 'b> {
                marker: _serde::__private::PhantomData<AnimationGraphSerial>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
                type_registry: &'a TypeRegistry,
                load_context: &'a mut LoadContext<'b>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de, '_, '_> {
                // manual impl end
                type Value = AnimationGraphSerial;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct AnimationGraphSerial",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    // manual impl start - use seed
                    let __field0 = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        NodesDeserializer {
                            type_registry: self.type_registry,
                            load_context: self.load_context,
                        },
                    )? {
                        // manual impl end
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ))
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        HashMap<TargetPinSerial, SourcePinSerial>,
                    >(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ))
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<
                        OrderedMap<PinIdSerial, DataValue>,
                    >(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ))
                        }
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<
                        OrderedMap<PinIdSerial, ()>,
                    >(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ))
                        }
                    };
                    let __field4 = match _serde::de::SeqAccess::next_element::<
                        OrderedMap<PinIdSerial, DataSpec>,
                    >(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                4usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ))
                        }
                    };
                    let __field5 =
                        match _serde::de::SeqAccess::next_element::<Option<()>>(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    5usize,
                                    &"struct AnimationGraphSerial with 7 elements",
                                ))
                            }
                        };
                    let __field6 = match _serde::de::SeqAccess::next_element::<Extra>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                6usize,
                                &"struct AnimationGraphSerial with 7 elements",
                            ))
                        }
                    };
                    _serde::__private::Ok(AnimationGraphSerial {
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
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<Vec<AnimationNode>> =
                        _serde::__private::None;
                    let mut __field1: _serde::__private::Option<
                        HashMap<TargetPinSerial, SourcePinSerial>,
                    > = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<
                        OrderedMap<PinIdSerial, DataValue>,
                    > = _serde::__private::None;
                    let mut __field3: _serde::__private::Option<OrderedMap<PinIdSerial, ()>> =
                        _serde::__private::None;
                    let mut __field4: _serde::__private::Option<OrderedMap<PinIdSerial, DataSpec>> =
                        _serde::__private::None;
                    let mut __field5: _serde::__private::Option<Option<()>> =
                        _serde::__private::None;
                    let mut __field6: _serde::__private::Option<Extra> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("nodes"),
                                    );
                                }
                                __field0 =
                                    // manual impl start - use seed
                                    _serde::__private::Some(_serde::de::MapAccess::next_value_seed(
                                        &mut __map,
                                        NodesDeserializer {
                                            type_registry: self.type_registry,
                                            load_context: self.load_context,
                                        }
                                    )?);
                                // manual impl end
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "edges_inverted",
                                        ),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        HashMap<TargetPinSerial, SourcePinSerial>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "default_parameters",
                                        ),
                                    );
                                }
                                __field2 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        OrderedMap<PinIdSerial, DataValue>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "input_times",
                                        ),
                                    );
                                }
                                __field3 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        OrderedMap<PinIdSerial, ()>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field4 => {
                                if _serde::__private::Option::is_some(&__field4) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "output_parameters",
                                        ),
                                    );
                                }
                                __field4 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        OrderedMap<PinIdSerial, DataSpec>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field5 => {
                                if _serde::__private::Option::is_some(&__field5) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "output_time",
                                        ),
                                    );
                                }
                                __field5 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        Option<()>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field6 => {
                                if _serde::__private::Option::is_some(&__field6) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("extra"),
                                    );
                                }
                                __field6 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        Extra,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        // manual impl start
                        _serde::__private::None => Default::default(),
                        // manual impl end
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("edges_inverted")?
                        }
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("default_parameters")?
                        }
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("input_times")?
                        }
                    };
                    let __field4 = match __field4 {
                        _serde::__private::Some(__field4) => __field4,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("output_parameters")?
                        }
                    };
                    let __field5 = match __field5 {
                        _serde::__private::Some(__field5) => __field5,
                        _serde::__private::None => {
                            _serde::__private::de::missing_field("output_time")?
                        }
                    };
                    let __field6 = match __field6 {
                        _serde::__private::Some(__field6) => __field6,
                        _serde::__private::None => _serde::__private::de::missing_field("extra")?,
                    };
                    _serde::__private::Ok(AnimationGraphSerial {
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
            const FIELDS: &'static [&'static str] = &[
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
                    marker: _serde::__private::PhantomData::<AnimationGraphSerial>,
                    lifetime: _serde::__private::PhantomData,
                    // manual impl start - add seed fields
                    load_context: self.load_context,
                    type_registry: self.type_registry,
                    // manual impl end
                },
            )
        }
    }
};
