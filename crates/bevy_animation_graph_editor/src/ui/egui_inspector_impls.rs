use bevy::{
    app::Plugin,
    ecs::{prelude::AppTypeRegistry, system::Resource},
    prelude::{App, Reflect},
    reflect::{FromReflect, PartialReflect, TypePath, TypeRegistry},
    utils::HashMap,
};
use bevy_animation_graph::core::{
    animation_clip::EntityPath,
    animation_graph::{AnimationGraph, PinId},
    edge_data::{BoneMask, DataSpec, DataValue},
};
use bevy_inspector_egui::{
    egui,
    inspector_egui_impls::InspectorEguiImpl,
    reflect_inspector::{InspectorUi, ProjectorReflect},
    restricted_world_view::RestrictedWorldView,
};
use core::default::Default;
use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
};

use super::reflect_widgets;

#[derive(Clone, Reflect, Debug)]
pub struct OrderedMap<K, V> {
    pub order: HashMap<K, i32>,
    pub values: HashMap<K, V>,
}

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        OrderedMap {
            order: Default::default(),
            values: Default::default(),
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone> OrderedMap<K, V> {
    pub fn to_vec(&self) -> Vec<(K, V)> {
        let mut vals: Vec<(K, V)> = self.values.clone().into_iter().collect();

        vals.sort_by_key(|(k, _)| self.order.get(k).copied().unwrap_or(0));

        vals
    }

    pub fn from_vec(vec: &[(K, V)]) -> Self {
        let mut order_map = OrderedMap::default();

        for (i, (k, v)) in vec.iter().enumerate() {
            order_map.order.insert(k.clone(), i as i32);
            order_map.values.insert(k.clone(), v.clone());
        }

        order_map
    }
}

pub struct BetterInspectorPlugin;

impl Plugin for BetterInspectorPlugin {
    fn build(&self, app: &mut App) {
        // This shall replace this plugin in due time
        // Currently using both for legacy reasons (too much effort to migrate everything at once)
        app.add_plugins(reflect_widgets::plugin::BetterInspectorPlugin);

        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<PinId, DataValue>,
            Vec<(PinId, DataValue)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<EntityPath, f32>,
            Vec<(EntityPath, f32)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<PinId, DataSpec>,
            Vec<(PinId, DataSpec)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<PinId, ()>,
            Vec<(PinId, ())>,
        >::default());

        app.insert_resource(EguiInspectorBuffers::<String>::default());
        app.insert_resource(EguiInspectorBuffers::<EntityPath, String>::default());

        app.register_type::<HashMap<EntityPath, f32>>();
        app.register_type::<OrderedMap<PinId, DataValue>>();
        app.register_type::<OrderedMap<PinId, DataSpec>>();
        app.register_type::<OrderedMap<PinId, ()>>();

        let type_registry = app.world().resource::<AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        let type_registry = &mut type_registry;

        add_no_many::<String>(type_registry, string_mut, string_readonly);
        add_no_many::<AnimationGraph>(type_registry, animation_graph_mut, todo_readonly_ui);
        add_no_many::<BoneMask>(type_registry, bone_mask_ui_mut, todo_readonly_ui);
        add_no_many::<HashMap<EntityPath, f32>>(
            type_registry,
            better_hashmap::<EntityPath, f32>,
            todo_readonly_ui,
        );
        add_no_many::<OrderedMap<PinId, DataValue>>(
            type_registry,
            better_ordered_map::<PinId, DataValue>,
            todo_readonly_ui,
        );
        add_no_many::<OrderedMap<PinId, DataSpec>>(
            type_registry,
            better_ordered_map::<PinId, DataSpec>,
            todo_readonly_ui,
        );
        add_no_many::<OrderedMap<PinId, ()>>(
            type_registry,
            better_ordered_map::<PinId, ()>,
            todo_readonly_ui,
        );
    }
}

#[derive(Resource)]
struct EguiInspectorBuffers<T, B = T> {
    bufs: HashMap<egui::Id, B>,
    _marker: std::marker::PhantomData<T>,
}

impl<T, B> Default for EguiInspectorBuffers<T, B> {
    fn default() -> Self {
        Self {
            bufs: HashMap::default(),
            _marker: std::marker::PhantomData,
        }
    }
}

fn get_buffered<'w, T, B>(
    world: &mut RestrictedWorldView<'w>,
    id: egui::Id,
    default: impl FnOnce() -> B,
) -> &'w mut B
where
    T: Send + Sync + Clone + 'static,
    B: Send + Sync + 'static,
{
    let mut res = world
        .get_resource_mut::<EguiInspectorBuffers<T, B>>()
        .unwrap();
    // SAFETY: This is safe because the buffers are only accessed from the inspector
    //         which should only be accessed from one thread. Additionally, every
    //         item ID should only be accessed once, mutably. There are never multiple references
    //         to any buffer.
    if res.bufs.contains_key(&id) {
        unsafe { &mut *(res.bufs.get_mut(&id).unwrap() as *mut B) }
    } else {
        res.bufs.insert(id, default());
        unsafe { &mut *(res.bufs.get_mut(&id).unwrap() as *mut B) }
    }
}

type InspectorEguiImplFn =
    fn(&mut dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>) -> bool;
type InspectorEguiImplFnReadonly =
    fn(&dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>);

fn many_unimplemented(
    _ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn PartialReflect],
    _projector: &dyn ProjectorReflect,
) -> bool {
    false
}

fn add_no_many<T: 'static>(
    type_registry: &mut TypeRegistry,
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::new(
            fn_mut,
            fn_readonly,
            many_unimplemented,
        ));
}

fn string_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<String>().unwrap();
    let buffered =
        get_buffered::<String, String>(env.context.world.as_mut().unwrap(), id, || value.clone());

    let response = ui.text_edit_singleline(buffered);

    if response.lost_focus() {
        value.clone_from(buffered);
        true
    } else if !response.has_focus() {
        buffered.clone_from(value);
        false
    } else {
        false
    }
}

fn string_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    mut _env: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<String>().unwrap();
    ui.label(value);
}

pub fn animation_graph_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value: &mut AnimationGraph = value.downcast_mut::<AnimationGraph>().unwrap();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write_u64(unsafe { std::mem::transmute_copy(&id) });
    hasher.write_usize(0);
    let default_params_id = egui::Id::new(Hasher::finish(&hasher));
    hasher.write_usize(0);
    let output_params_id = egui::Id::new(Hasher::finish(&hasher));
    hasher.write_usize(0);
    let input_poses_id = egui::Id::new(Hasher::finish(&hasher));
    hasher.write_usize(0);
    let output_pose_id = egui::Id::new(Hasher::finish(&hasher));

    let mut default_params_changed = false;
    let mut output_params_changed = false;
    let mut input_poses_changed = false;
    let mut output_pose_changed = false;
    //ui.heading("Default input parameters");
    let mut default_params = OrderedMap {
        order: value.extra.input_param_order.clone(),
        values: value.default_parameters.clone(),
    };
    ui.collapsing("Default input data", |ui| {
        default_params_changed =
            env.ui_for_reflect_with_options(&mut default_params, ui, default_params_id, &());
    });
    if default_params_changed {
        value.default_parameters = default_params.values.clone();
        value.extra.input_param_order = default_params.order.clone();
    }

    let mut output_params = OrderedMap {
        order: value.extra.output_data_order.clone(),
        values: value.output_parameters.clone(),
    };
    ui.collapsing("Output data", |ui| {
        output_params_changed =
            env.ui_for_reflect_with_options(&mut output_params, ui, output_params_id, &());
    });
    if output_params_changed {
        value.output_parameters = output_params.values.clone();
        value.extra.output_data_order = output_params.order.clone();
    }

    let mut input_times = OrderedMap {
        order: value.extra.input_time_order.clone(),
        values: value.input_times.clone(),
    };
    ui.collapsing("Input times", |ui| {
        input_poses_changed =
            env.ui_for_reflect_with_options(&mut input_times, ui, input_poses_id, &());
    });
    if input_poses_changed {
        value.input_times = input_times.values.clone();
        value.extra.input_time_order = input_times.order.clone();
    }

    ui.collapsing("Output time", |ui| {
        output_pose_changed =
            env.ui_for_reflect_with_options(&mut value.output_time, ui, output_pose_id, &());
    });

    default_params_changed || output_params_changed || input_poses_changed || output_pose_changed
}

pub fn bone_mask_ui_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<BoneMask>().unwrap();
    let (curr_val, curr_label) = if matches!(value, BoneMask::Positive { .. }) {
        (0, "Positive".to_string())
    } else {
        (1, "Negative".to_string())
    };

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write_u64(unsafe { std::mem::transmute_copy(&id) });
    hasher.write_usize(0);
    let bones_id = egui::Id::new(Hasher::finish(&hasher));

    let mut new_val = curr_val;

    egui::ComboBox::new(id, "Mask type")
        .selected_text(curr_label)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut new_val, 0, "Positive")
                .on_hover_ui(|ui| {
                    ui.label("Bones not in the bone mask are given a weight of 0.");
                });
            ui.selectable_value(&mut new_val, 1, "Negative")
                .on_hover_ui(|ui| {
                    ui.label("Bones not in the bone mask are given a weight of 1.");
                });
        });
    if new_val != curr_val {
        match new_val {
            0 => {
                *value = BoneMask::Positive {
                    bones: Default::default(),
                };
                *value = BoneMask::Negative {
                    bones: Default::default(),
                };
            }
            1 => {}
            _ => unreachable!(),
        }
        return true;
    }

    let bones = match value {
        BoneMask::Positive { bones } => bones,
        BoneMask::Negative { bones } => bones,
    };

    ui.horizontal(|ui| {
        ui.label("Bones");
        env.ui_for_reflect_with_options(bones, ui, bones_id, &())
    })
    .inner
}

pub fn better_hashmap<
    K: Clone + FromReflect + TypePath + Default + Hash + Eq + PartialEq,
    T: Clone + FromReflect + TypePath + Default,
>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<HashMap<K, T>>().unwrap();
    let buffered =
        get_buffered::<HashMap<K, T>, Vec<(K, T)>>(env.context.world.as_mut().unwrap(), id, || {
            value.clone().into_iter().collect()
        });

    let mut should_write_back = false;

    ui.vertical(|ui| {
        let mut i = 0;
        while i < buffered.len() {
            let mut skip_push = false;
            ui.push_id(i, |ui| {
                ui.separator();
                let removed = ui.button("- Remove").clicked();
                let mut key_changed = false;
                let mut value_changed = false;
                let (mut k, mut v) = buffered[i].clone();
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                hasher.write_u64(unsafe { std::mem::transmute_copy(&id) });
                hasher.write_usize(i);
                let key_id = egui::Id::new(Hasher::finish(&hasher));
                hasher.write_usize(0);
                let value_id = egui::Id::new(Hasher::finish(&hasher));

                ui.horizontal(|ui| {
                    ui.label("Key");
                    key_changed = env.ui_for_reflect_with_options(&mut k, ui, key_id, &());
                });

                ui.horizontal(|ui| {
                    ui.label("Value");
                    value_changed = env.ui_for_reflect_with_options(&mut v, ui, value_id, &());
                });

                if removed {
                    should_write_back = true;
                    buffered.remove(i);
                    skip_push = true;
                } else if key_changed || value_changed {
                    should_write_back = true;
                    buffered[i] = (k.clone(), v.clone());
                }
            });

            if !skip_push {
                i += 1;
            }
        }
        ui.separator();
        if ui.button("+ Add").clicked() {
            should_write_back = true;
            buffered.push((K::default(), T::default()));
        }
        ui.separator();
    });

    if should_write_back {
        *value = buffered.clone().into_iter().collect();
        *buffered = value.clone().into_iter().collect();
        true
    } else {
        false
    }
}

pub fn better_ordered_map<
    K: Clone + FromReflect + TypePath + Default + Hash + Eq + PartialEq,
    T: Clone + FromReflect + TypePath + Default,
>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value: &mut OrderedMap<K, T> = value.downcast_mut::<OrderedMap<K, T>>().unwrap();
    let buffered = get_buffered::<OrderedMap<K, T>, Vec<(K, T)>>(
        env.context.world.as_mut().unwrap(),
        id,
        || value.to_vec(),
    );

    let mut should_write_back = false;

    ui.vertical(|ui| {
        let mut i = 0;
        while i < buffered.len() {
            let mut skip_push = false;
            let mut end_loop = false;
            ui.push_id(i, |ui| {
                ui.separator();
                let (removed, push_up, push_down) = ui
                    .horizontal(|ui| {
                        (
                            ui.button("- Remove").clicked(),
                            ui.button("⬆ Move up").clicked(),
                            ui.button("⬇ Move down").clicked(),
                        )
                    })
                    .inner;
                let mut key_changed = false;
                let mut value_changed = false;
                let (mut k, mut v) = buffered[i].clone();
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                hasher.write_u64(unsafe { std::mem::transmute_copy(&id) });
                hasher.write_usize(i);
                let key_id = egui::Id::new(Hasher::finish(&hasher));
                hasher.write_usize(0);
                let value_id = egui::Id::new(Hasher::finish(&hasher));

                ui.horizontal(|ui| {
                    ui.label("Key");
                    key_changed = env.ui_for_reflect_with_options(&mut k, ui, key_id, &());
                });

                ui.horizontal(|ui| {
                    ui.label("Value");
                    value_changed = env.ui_for_reflect_with_options(&mut v, ui, value_id, &());
                });

                if removed {
                    should_write_back = true;
                    buffered.remove(i);
                    skip_push = true;
                } else if push_up && i > 0 {
                    should_write_back = true;
                    buffered.swap(i, i - 1);
                    end_loop = true;
                } else if push_down && i < buffered.len() - 1 {
                    should_write_back = true;
                    buffered.swap(i, i + 1);
                    end_loop = true;
                } else if key_changed || value_changed {
                    should_write_back = true;
                    buffered[i] = (k.clone(), v.clone());
                }
            });

            if !skip_push {
                i += 1;
            }
            if end_loop {
                break;
            }
        }
        ui.separator();
        if ui.button("+ Add").clicked() {
            should_write_back = true;
            buffered.push((K::default(), T::default()));
        }
        ui.separator();
    });

    if should_write_back {
        *value = OrderedMap::from_vec(buffered);
        *buffered = value.to_vec();
        true
    } else {
        false
    }
}

pub fn todo_readonly_ui(
    _value: &dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    mut _env: InspectorUi<'_, '_>,
) {
    ui.label("TODO: Asset picker readonly. If you see this please report an issue.");
}
