use crate::core::animation_clip::AnimationClip;
use crate::core::animation_graph::{AnimationGraph, EdgeSpec};
use crate::core::animation_player::AnimationPlayer;
use crate::core::systems::{animation_player, replace_animation_players};
use bevy::app::{App, Plugin, PostUpdate};
use bevy::asset::AssetApp;
use bevy::ecs::prelude::*;
use bevy::prelude::PreUpdate;
use bevy::reflect::Reflect;
use bevy::transform::TransformSystem;
use bevy::utils::HashMap;

#[derive(Clone, Copy, Reflect, Debug)]
pub enum InterpolationMode {
    Constant,
    Linear,
}

pub enum WrapEnd {
    Loop,
    Extend,
}

pub trait HashMapJoinExt<K, V> {
    type Val;

    fn fill_up<F>(&mut self, other: &HashMap<K, V>, mapper: &F) -> &mut Self
    where
        F: Fn(&V) -> Self::Val;
}

impl HashMapJoinExt<String, EdgeSpec> for HashMap<String, EdgeSpec> {
    type Val = EdgeSpec;

    fn fill_up<F>(&mut self, other: &HashMap<String, EdgeSpec>, mapper: &F) -> &mut Self
    where
        F: Fn(&EdgeSpec) -> Self::Val,
    {
        for (k, v) in other {
            if !self.contains_key(k) {
                self.insert(k.clone(), mapper(v));
            }
        }
        self
    }
}

impl<T> HashMapJoinExt<String, T> for HashMap<String, ()> {
    type Val = ();

    fn fill_up<F>(&mut self, other: &HashMap<String, T>, _: &F) -> &mut Self
    where
        F: Fn(&T) -> Self::Val,
    {
        for (k, _) in other {
            if !self.contains_key(k) {
                self.insert(k.clone(), ());
            }
        }
        self
    }
}

/// Adds animation support to an app
#[derive(Default)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app //
            .init_asset::<AnimationClip>()
            .init_asset::<AnimationGraph>()
            .register_asset_reflect::<AnimationClip>()
            .register_type::<AnimationPlayer>()
            .add_systems(PreUpdate, replace_animation_players)
            .add_systems(
                PostUpdate,
                animation_player.before(TransformSystem::TransformPropagate),
            );
    }
}
