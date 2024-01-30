use bevy::{
    reflect::{
        map_apply, map_partial_eq, utility::GenericTypeInfoCell, DynamicMap, FromReflect, FromType,
        GetTypeRegistration, Map, MapInfo, MapIter, Reflect, ReflectFromPtr, ReflectMut,
        ReflectOwned, ReflectRef, TypeInfo, TypePath, TypeRegistration, Typed,
    },
    utils::AHasher,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Serializer};
use std::{
    any::Any,
    hash::{BuildHasher, BuildHasherDefault, Hash},
    ops::{Deref, DerefMut},
};

#[derive(TypePath, Debug, Clone, Default)]
pub struct OrderedMap<K, V, S = BuildHasherDefault<AHasher>>(IndexMap<K, V, S>);

impl<K, V, S: Default> OrderedMap<K, V, S> {
    pub fn new() -> Self {
        Self(IndexMap::default())
    }
}

impl<const N: usize, K: Hash + Eq, V, S: BuildHasher + Default> From<[(K, V); N]>
    for OrderedMap<K, V, S>
{
    fn from(value: [(K, V); N]) -> Self {
        OrderedMap::from_iter(value)
    }
}

impl<K, V, S> FromIterator<(K, V)> for OrderedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(IndexMap::from_iter(iter))
    }
}

impl<'a, K, V, S> IntoIterator for &'a OrderedMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = <&'a IndexMap<K, V, S> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut OrderedMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = <&'a mut IndexMap<K, V, S> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<K, V, S> IntoIterator for OrderedMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = <IndexMap<K, V, S> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V, S> Serialize for OrderedMap<K, V, S>
where
    K: Serialize + Eq + Hash,
    V: Serialize,
    S: BuildHasher,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, K, V, S> Deserialize<'de> for OrderedMap<K, V, S>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IndexMap::deserialize(deserializer).map(|im| OrderedMap(im))
    }
}

impl<K, V, S> Deref for OrderedMap<K, V, S> {
    type Target = IndexMap<K, V, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for OrderedMap<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --- Manual impl of `impl_reflect_for_hashmap`
// ----------------------------------------------------------------------------------------
impl<K, V, S> Map for OrderedMap<K, V, S>
where
    K: FromReflect + TypePath + Eq + Hash,
    V: FromReflect + TypePath,
    S: TypePath + BuildHasher + Send + Sync,
{
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        key.downcast_ref::<K>()
            .and_then(|key| Self::get(self, key))
            .map(|value| value as &dyn Reflect)
    }

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect> {
        key.downcast_ref::<K>()
            .and_then(move |key| Self::get_mut(self, key))
            .map(|value| value as &mut dyn Reflect)
    }

    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.iter()
            .nth(index)
            .map(|(key, value)| (key as &dyn Reflect, value as &dyn Reflect))
    }

    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
        self.iter_mut()
            .nth(index)
            .map(|(key, value)| (key as &dyn Reflect, value as &mut dyn Reflect))
    }

    fn len(&self) -> usize {
        self.deref().len()
    }

    fn iter(&self) -> MapIter {
        MapIter::new(self)
    }

    fn drain(self: Box<Self>) -> Vec<(Box<dyn Reflect>, Box<dyn Reflect>)> {
        self.0
            .into_iter()
            .map(|(key, value)| {
                (
                    Box::new(key) as Box<dyn Reflect>,
                    Box::new(value) as Box<dyn Reflect>,
                )
            })
            .collect()
    }

    fn clone_dynamic(&self) -> DynamicMap {
        let mut dynamic_map = DynamicMap::default();
        dynamic_map.set_represented_type(self.get_represented_type_info());
        for (k, v) in self.deref() {
            let key = K::from_reflect(k).unwrap_or_else(|| {
                panic!(
                    "Attempted to clone invalid key of type {}.",
                    k.reflect_type_path()
                )
            });
            dynamic_map.insert_boxed(Box::new(key), v.clone_value());
        }
        dynamic_map
    }

    fn insert_boxed(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Option<Box<dyn Reflect>> {
        let key = K::take_from_reflect(key).unwrap_or_else(|key| {
            panic!(
                "Attempted to insert invalid key of type {}.",
                key.reflect_type_path()
            )
        });
        let value = V::take_from_reflect(value).unwrap_or_else(|value| {
            panic!(
                "Attempted to insert invalid value of type {}.",
                value.reflect_type_path()
            )
        });
        self.insert(key, value)
            .map(|old_value| Box::new(old_value) as Box<dyn Reflect>)
    }

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let mut from_reflect = None;
        key.downcast_ref::<K>()
            .or_else(|| {
                from_reflect = K::from_reflect(key);
                from_reflect.as_ref()
            })
            .and_then(|key| self.remove(key))
            .map(|value| value as Box<dyn Reflect>)
    }
}

impl<K, V, S> Reflect for OrderedMap<K, V, S>
where
    K: FromReflect + TypePath + Eq + Hash,
    V: FromReflect + TypePath,
    S: TypePath + BuildHasher + Send + Sync,
{
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(<Self as Typed>::type_info())
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn apply(&mut self, value: &dyn Reflect) {
        map_apply(self, value);
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Map(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Map(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Map(self)
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic())
    }

    fn reflect_partial_eq(&self, value: &dyn Reflect) -> Option<bool> {
        map_partial_eq(self, value)
    }
}

impl<K, V, S> Typed for OrderedMap<K, V, S>
where
    K: FromReflect + TypePath + Eq + Hash,
    V: FromReflect + TypePath,
    S: TypePath + BuildHasher + Send + Sync,
{
    fn type_info() -> &'static TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
        CELL.get_or_insert::<Self, _>(|| TypeInfo::Map(MapInfo::new::<Self, K, V>()))
    }
}

impl<K, V, S> GetTypeRegistration for OrderedMap<K, V, S>
where
    K: FromReflect + TypePath + Eq + Hash,
    V: FromReflect + TypePath,
    S: TypePath + BuildHasher + Send + Sync,
{
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration
    }
}

impl<K, V, S> FromReflect for OrderedMap<K, V, S>
where
    K: FromReflect + TypePath + Eq + Hash,
    V: FromReflect + TypePath,
    S: TypePath + BuildHasher + Default + Send + Sync,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let ReflectRef::Map(ref_map) = reflect.reflect_ref() {
            let mut new_map = IndexMap::with_capacity_and_hasher(ref_map.len(), S::default());
            for (key, value) in ref_map.iter() {
                let new_key = K::from_reflect(key)?;
                let new_value = V::from_reflect(value)?;
                new_map.insert(new_key, new_value);
            }
            Some(OrderedMap(new_map))
        } else {
            None
        }
    }
}
// ----------------------------------------------------------------------------------------
