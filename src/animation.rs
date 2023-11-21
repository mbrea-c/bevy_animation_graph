use crate::sampling::linear::SampleLinear;
use bevy::app::{App, Plugin, PostUpdate};
use bevy::asset::{Asset, AssetApp, Assets, Handle};
use bevy::core::Name;
use bevy::ecs::prelude::*;
use bevy::hierarchy::{Children, Parent};
use bevy::math::{Quat, Vec3};
use bevy::prelude::PreUpdate;
use bevy::reflect::{FromReflect, Reflect, TypePath};
use bevy::render::mesh::morph::MorphWeights;
use bevy::time::Time;
use bevy::transform::{prelude::Transform, TransformSystem};
use bevy::utils::{tracing::warn, HashMap};
use std::ops::Deref;

/// List of keyframes for one of the attribute of a [`Transform`].
#[derive(Reflect, Clone, Debug)]
pub enum Keyframes {
    /// Keyframes for rotation.
    Rotation(Vec<Quat>),
    /// Keyframes for translation.
    Translation(Vec<Vec3>),
    /// Keyframes for scale.
    Scale(Vec<Vec3>),
    /// Keyframes for morph target weights.
    ///
    /// Note that in `.0`, each contiguous `target_count` values is a single
    /// keyframe representing the weight values at given keyframe.
    ///
    /// This follows the [glTF design].
    ///
    /// [glTF design]: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#animations
    Weights(Vec<f32>),
}

/// Vertical slice of a [`Keyframes`] that represents an instant in an animation [`Transform`].
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct BonePose {
    pub(crate) rotation: Option<Quat>,
    pub(crate) translation: Option<Vec3>,
    pub(crate) scale: Option<Vec3>,
    pub(crate) weights: Option<Vec<f32>>,
}

#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct ValueFrame<T: FromReflect + TypePath> {
    pub(crate) prev: T,
    pub(crate) prev_timestamp: f32,
    pub(crate) next: T,
    pub(crate) next_timestamp: f32,
    pub(crate) next_is_wrapped: bool,
}

impl<T: FromReflect + TypePath> ValueFrame<T> {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.prev_timestamp = f(self.prev_timestamp);
        self.next_timestamp = f(self.next_timestamp);
    }
}

#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct BoneFrame {
    pub(crate) rotation: Option<ValueFrame<Quat>>,
    pub(crate) translation: Option<ValueFrame<Vec3>>,
    pub(crate) scale: Option<ValueFrame<Vec3>>,
    pub(crate) weights: Option<ValueFrame<Vec<f32>>>,
}

impl BoneFrame {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.rotation.as_mut().map(|v| v.map_ts(&f));
        self.translation.as_mut().map(|v| v.map_ts(&f));
        self.scale.as_mut().map(|v| v.map_ts(&f));
        self.weights.as_mut().map(|v| v.map_ts(&f));
    }
}

/// Describes how an attribute of a [`Transform`] or [`MorphWeights`] should be animated.
///
/// `keyframe_timestamps` and `keyframes` should have the same length.
#[derive(Reflect, Clone, Debug)]
pub struct VariableCurve {
    /// Timestamp for each of the keyframes.
    pub keyframe_timestamps: Vec<f32>,
    /// List of the keyframes.
    pub keyframes: Keyframes,
}

/// Path to an entity, with [`Name`]s. Each entity in a path must have a name.
#[derive(Reflect, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct EntityPath {
    /// Parts of the path
    pub parts: Vec<Name>,
}

/// A list of [`VariableCurve`], and the [`EntityPath`] to which they apply.
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct AnimationClip {
    pub(crate) curves: Vec<Vec<VariableCurve>>,
    pub(crate) paths: HashMap<EntityPath, usize>,
    pub(crate) duration: f32,
}

impl AnimationClip {
    #[inline]
    /// [`VariableCurve`]s for each bone. Indexed by the bone ID.
    pub fn curves(&self) -> &Vec<Vec<VariableCurve>> {
        &self.curves
    }

    /// Gets the curves for a bone.
    ///
    /// Returns `None` if the bone is invalid.
    #[inline]
    pub fn get_curves(&self, bone_id: usize) -> Option<&'_ Vec<VariableCurve>> {
        self.curves.get(bone_id)
    }

    /// Gets the curves by it's [`EntityPath`].
    ///
    /// Returns `None` if the bone is invalid.
    #[inline]
    pub fn get_curves_by_path(&self, path: &EntityPath) -> Option<&'_ Vec<VariableCurve>> {
        self.paths.get(path).and_then(|id| self.curves.get(*id))
    }

    /// Duration of the clip, represented in seconds
    #[inline]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Add a [`VariableCurve`] to an [`EntityPath`].
    pub fn add_curve_to_path(&mut self, path: EntityPath, curve: VariableCurve) {
        // Update the duration of the animation by this curve duration if it's longer
        self.duration = self
            .duration
            .max(*curve.keyframe_timestamps.last().unwrap_or(&0.0));
        if let Some(bone_id) = self.paths.get(&path) {
            self.curves[*bone_id].push(curve);
        } else {
            let idx = self.curves.len();
            self.curves.push(vec![curve]);
            self.paths.insert(path, idx);
        }
    }

    /// Whether this animation clip can run on entity with given [`Name`].
    pub fn compatible_with(&self, name: &Name) -> bool {
        self.paths.keys().any(|path| &path.parts[0] == name)
    }
}

impl From<bevy::animation::AnimationClip> for AnimationClip {
    fn from(value: bevy::animation::AnimationClip) -> Self {
        // HACK: to get the corret type, since bevy's AnimationClip
        // does not expose its internals
        unsafe { std::mem::transmute(value) }
    }
}

/// Vertical slice of an [`AnimationClip`]
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct Pose {
    pub(crate) bones: Vec<BonePose>,
    pub(crate) paths: HashMap<EntityPath, usize>,
}

impl Pose {
    pub fn add_bone(&mut self, pose: BonePose, path: EntityPath) {
        let id = self.bones.len();
        self.bones.insert(id, pose);
        self.paths.insert(path, id);
    }
    //
    // pub fn interpolate_linear(&self, other: &Self, f: f32) -> Pose {
    //     if f.is_nan() {
    //         return self.clone();
    //     }
    //
    //     let mut result = Pose::default();
    //
    //     for (path, bone_id) in self.paths.iter() {
    //         let Some(other_bone_id) = other.paths.get(path) else {
    //             continue;
    //         };
    //
    //         result.add_bone(
    //             self.bones[*bone_id].interpolate_linear(&other.bones[*other_bone_id], f),
    //             path.clone(),
    //         );
    //     }
    //
    //     result
    // }
}

#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct PoseFrame {
    pub(crate) bones: Vec<BoneFrame>,
    pub(crate) paths: HashMap<EntityPath, usize>,
}

impl PoseFrame {
    pub(crate) fn add_bone(&mut self, frame: BoneFrame, path: EntityPath) {
        let id = self.bones.len();
        self.bones.insert(id, frame);
        self.paths.insert(path, id);
    }

    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.bones.iter_mut().for_each(|v| v.map_ts(&f));
    }
}

#[derive(Reflect, Clone, Copy, Debug)]
pub enum EdgeSpec {
    PoseFrame,
    F32,
}

#[derive(Reflect, Clone, Debug)]
pub enum EdgeValue {
    PoseFrame(PoseFrame),
    F32(f32),
}

impl EdgeValue {
    pub fn unwrap_pose_frame(self) -> PoseFrame {
        match self {
            Self::PoseFrame(p) => p,
            Self::F32(_) => panic!("Edge value is not a pose frame"),
        }
    }
}

impl From<EdgeValue> for EdgeSpec {
    fn from(value: EdgeValue) -> Self {
        match value {
            EdgeValue::PoseFrame(_) => Self::PoseFrame,
            EdgeValue::F32(_) => Self::F32,
        }
    }
}

pub type NodeInput = String;
pub type NodeOutput = String;

pub trait AnimationNode: Send + Sync {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32>;
    fn forward(&self, time: f32) -> HashMap<NodeInput, f32>;
    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue>;

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec>;
    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec>;
}

pub struct NodeWrapper {
    node: Box<dyn AnimationNode>,
    // Below are cached values for performance
    duration_cached: Option<Option<f32>>,
}

impl NodeWrapper {
    pub fn new(node: Box<dyn AnimationNode>) -> Self {
        Self {
            node,
            duration_cached: None,
        }
    }
}

impl From<Box<dyn AnimationNode>> for NodeWrapper {
    fn from(value: Box<dyn AnimationNode>) -> Self {
        Self {
            node: value,
            duration_cached: None,
        }
    }
}

pub enum InterpolationMode {
    Constant,
    Linear,
}

#[derive(Asset, TypePath)]
pub struct AnimationGraph {
    nodes: HashMap<String, NodeWrapper>,
    /// Inverted, indexed by output node name.
    edges: HashMap<(String, String), (String, String)>,
    out_node: String,
    out_edge: String,
    output_interpolation: InterpolationMode,
}

impl AnimationGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            out_node: "".into(),
            out_edge: "".into(),
            output_interpolation: InterpolationMode::Constant,
        }
    }

    pub fn set_interpolation(&mut self, interpolation: InterpolationMode) {
        self.output_interpolation = interpolation;
    }

    pub fn set_output(&mut self, node: String, edge: String) {
        self.out_node = node;
        self.out_edge = edge;
    }

    pub fn add_node(
        &mut self,
        node_name: String,
        node: NodeWrapper,
        make_out_edge: Option<String>,
    ) {
        self.nodes.insert(node_name.clone(), node);
        if let Some(out_edge) = make_out_edge {
            self.out_node = node_name;
            self.out_edge = out_edge;
        }
    }

    pub fn add_edge(
        &mut self,
        source_node: String,
        source_edge: String,
        target_node: String,
        target_edge: String,
    ) {
        self.edges
            .insert((target_node, target_edge), (source_node, source_edge));
    }

    pub fn query(&self, time: f32) -> Pose {
        self.forward_pass(time)
    }

    pub fn duration_pass(&mut self) {
        // First clear all duration caches
        for wrapper in self.nodes.values_mut() {
            wrapper.duration_cached = None;
        }

        self.update_duration_for(&self.out_node.clone());
    }

    fn update_duration_for(&mut self, node: &str) -> Option<f32> {
        if let Some(duration) = self.nodes.get(node).unwrap().duration_cached {
            return duration;
        }

        let input_spec = self.nodes.get(node).unwrap().node.input_spec();
        let input_keys = input_spec.keys();

        let mut mapped_keys = HashMap::new();

        for k in input_keys {
            mapped_keys.insert(
                k.clone(),
                self.edges.get(&(node.into(), k.into())).unwrap().clone(),
            );
        }

        let mut input_durations = HashMap::new();

        for (k, (prev_node, _prev_key)) in mapped_keys {
            input_durations.insert(k.clone(), self.update_duration_for(&prev_node));
        }

        let wrapper = self.nodes.get_mut(node).unwrap();
        wrapper.duration_cached = Some(wrapper.node.duration(input_durations));
        wrapper.duration_cached.unwrap()
    }

    fn forward_pass(&self, time: f32) -> Pose {
        match self
            .forward_pass_for(&self.out_node, time)
            .remove(&self.out_edge)
            .unwrap()
        {
            EdgeValue::PoseFrame(p) => match self.output_interpolation {
                InterpolationMode::Constant => todo!(),
                InterpolationMode::Linear => p.sample_linear(time),
            },
            EdgeValue::F32(_) => panic!("Output edge did not output a pose"),
        }
    }

    fn forward_pass_for(&self, node: &str, time: f32) -> HashMap<String, EdgeValue> {
        let wrapper = self.nodes.get(node).unwrap();

        let time_query = wrapper.node.forward(time);
        let extended_time_query = time_query
            .iter()
            .map(|(k, v)| (k, self.edges.get(&(node.into(), k.into())).unwrap(), *v));

        let backward_inputs = if !time_query.is_empty() {
            extended_time_query
                .map(|(input_k, (node_k, edge_k), t)| {
                    (
                        input_k.into(),
                        self.forward_pass_for(node_k, t).remove(edge_k).unwrap(),
                    )
                })
                .collect()
        } else {
            HashMap::<String, EdgeValue>::new()
        };

        wrapper.node.backward(time, backward_inputs)
    }
}

pub enum WrapEnd {
    Loop,
    Extend,
}

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationPlayer {
    paused: bool,
    animation: Option<Handle<AnimationGraph>>,
    elapsed: f32,
}

impl AnimationPlayer {
    /// Start playing an animation, resetting state of the player.
    /// This will use a linear blending between the previous and the new animation to make a smooth transition.
    pub fn start(&mut self, handle: Handle<AnimationGraph>) -> &mut Self {
        self.animation = Some(handle);
        self.elapsed = 0.;
        self.paused = false;
        self
    }

    pub fn pause(&mut self) -> &mut Self {
        self.paused = true;
        self
    }

    pub fn resume(&mut self) -> &mut Self {
        self.paused = false;
        self
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn reset(&mut self) -> &mut Self {
        self.elapsed = 0.;
        self
    }
}

fn entity_from_path(
    root: Entity,
    path: &EntityPath,
    children: &Query<&Children>,
    names: &Query<&Name>,
) -> Option<Entity> {
    // PERF: finding the target entity can be optimised
    let mut current_entity = root;

    let mut parts = path.parts.iter().enumerate();

    // check the first name is the root node which we already have
    let Some((_, root_name)) = parts.next() else {
        return None;
    };
    if names.get(current_entity) != Ok(root_name) {
        return None;
    }

    for (_idx, part) in parts {
        let mut found = false;
        let children = children.get(current_entity).ok()?;
        if !found {
            for child in children.deref() {
                if let Ok(name) = names.get(*child) {
                    if name == part {
                        // Found a children with the right name, continue to the next part
                        current_entity = *child;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            warn!("Entity not found for path {:?} on part {:?}", path, part);
            return None;
        }
    }
    Some(current_entity)
}

/// Verify that there are no ancestors of a given entity that have an [`AnimationPlayer`].
fn verify_no_ancestor_player(
    player_parent: Option<&Parent>,
    parents: &Query<(Has<AnimationPlayer>, Option<&Parent>)>,
) -> bool {
    let Some(mut current) = player_parent.map(Parent::get) else {
        return true;
    };
    loop {
        let Ok((has_player, parent)) = parents.get(current) else {
            return true;
        };
        if has_player {
            return false;
        }
        if let Some(parent) = parent {
            current = parent.get();
        } else {
            return true;
        }
    }
}

/// System that will play all animations, using any entity with a [`AnimationPlayer`]
/// and a [`Handle<AnimationClip>`] as an animation root
#[allow(clippy::too_many_arguments)]
pub fn animation_player(
    time: Res<Time>,
    graphs: Res<Assets<AnimationGraph>>,
    children: Query<&Children>,
    names: Query<&Name>,
    transforms: Query<&mut Transform>,
    morphs: Query<&mut MorphWeights>,
    parents: Query<(Has<AnimationPlayer>, Option<&Parent>)>,
    mut animation_players: Query<(Entity, Option<&Parent>, &mut AnimationPlayer)>,
) {
    animation_players
        .par_iter_mut()
        .for_each(|(root, maybe_parent, player)| {
            run_animation_player(
                root,
                player,
                &time,
                &graphs,
                &names,
                &transforms,
                &morphs,
                maybe_parent,
                &parents,
                &children,
            );
        });
}

#[allow(clippy::too_many_arguments)]
fn run_animation_player(
    root: Entity,
    mut player: Mut<AnimationPlayer>,
    time: &Time,
    graphs: &Assets<AnimationGraph>,
    names: &Query<&Name>,
    transforms: &Query<&mut Transform>,
    morphs: &Query<&mut MorphWeights>,
    maybe_parent: Option<&Parent>,
    parents: &Query<(Has<AnimationPlayer>, Option<&Parent>)>,
    children: &Query<&Children>,
) {
    let paused = player.paused;
    // Continue if paused unless the `AnimationPlayer` was changed
    // This allow the animation to still be updated if the player.elapsed field was manually updated in pause
    if paused || player.animation.is_none() {
        return;
    }

    player.elapsed += time.delta_seconds();

    // Apply the main animation
    apply_pose(
        &graphs
            .get(player.animation.as_ref().unwrap())
            .unwrap()
            .query(player.elapsed),
        root,
        names,
        transforms,
        morphs,
        maybe_parent,
        parents,
        children,
    );
}

/// Update `weights` based on weights in `keyframe` with a linear interpolation
/// on `key_lerp`.
fn lerp_morph_weights(weights: &[f32], new_weights: &[f32], key_lerp: f32) -> Vec<f32> {
    weights
        .iter()
        .zip(new_weights)
        .map(|(old, new)| (new - old) * key_lerp)
        .collect()
}

/// Update `weights` based on weights in `keyframe` with a linear interpolation
/// on `key_lerp`.
fn apply_morph_weights(weights: &mut [f32], new_weights: &[f32]) {
    let zipped = weights.iter_mut().zip(new_weights);
    for (morph_weight, keyframe) in zipped {
        *morph_weight = *keyframe;
    }
}

/// Extract a keyframe from a list of keyframes by index.
///
/// # Panics
///
/// When `key_index * target_count` is larger than `keyframes`
///
/// This happens when `keyframes` is not formatted as described in
/// [`Keyframes::Weights`]. A possible cause is [`AnimationClip`] not being
/// meant to be used for the [`MorphWeights`] of the entity it's being applied to.
pub(crate) fn get_keyframe(target_count: usize, keyframes: &[f32], key_index: usize) -> &[f32] {
    let start = target_count * key_index;
    let end = target_count * (key_index + 1);
    &keyframes[start..end]
}

#[allow(clippy::too_many_arguments)]
fn apply_pose(
    animation_pose: &Pose,
    root: Entity,
    names: &Query<&Name>,
    transforms: &Query<&mut Transform>,
    morphs: &Query<&mut MorphWeights>,
    maybe_parent: Option<&Parent>,
    parents: &Query<(Has<AnimationPlayer>, Option<&Parent>)>,
    children: &Query<&Children>,
) {
    if !verify_no_ancestor_player(maybe_parent, parents) {
        warn!("Animation player on {:?} has a conflicting animation player on an ancestor. Cannot safely animate.", root);
        return;
    }

    let mut any_path_found = false;
    for (path, bone_id) in &animation_pose.paths {
        let Some(target) = entity_from_path(root, path, children, names) else {
            continue;
        };
        any_path_found = true;
        // SAFETY: The verify_no_ancestor_player check above ensures that two animation players cannot alias
        // any of their descendant Transforms.
        //
        // The system scheduler prevents any other system from mutating Transforms at the same time,
        // so the only way this fetch can alias is if two AnimationPlayers are targeting the same bone.
        // This can only happen if there are two or more AnimationPlayers are ancestors to the same
        // entities. By verifying that there is no other AnimationPlayer in the ancestors of a
        // running AnimationPlayer before animating any entity, this fetch cannot alias.
        //
        // This means only the AnimationPlayers closest to the root of the hierarchy will be able
        // to run their animation. Any players in the children or descendants will log a warning
        // and do nothing.
        let Ok(mut transform) = (unsafe { transforms.get_unchecked(target) }) else {
            continue;
        };

        let pose = &animation_pose.bones[*bone_id];
        let mut morphs = unsafe { morphs.get_unchecked(target) };
        if let Some(rotation) = pose.rotation {
            transform.rotation = rotation;
        }
        if let Some(translation) = pose.translation {
            transform.translation = translation;
        }
        if let Some(scale) = pose.scale {
            transform.scale = scale;
        }
        if let Some(weights) = &pose.weights {
            if let Ok(morphs) = &mut morphs {
                apply_morph_weights(morphs.weights_mut(), &weights);
            }
        }
    }

    if !any_path_found {
        warn!("Animation player on {root:?} did not match any entity paths.");
    }
}

fn replace_animation_players(
    mut commands: Commands,
    query: Query<(Entity, &bevy::animation::AnimationPlayer)>,
) {
    for (entity, _player) in &query {
        commands
            .entity(entity)
            .remove::<bevy::animation::AnimationPlayer>()
            .insert(AnimationPlayer::default());
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
