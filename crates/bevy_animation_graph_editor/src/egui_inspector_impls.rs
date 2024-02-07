use bevy::{
    app::Plugin,
    asset::{Asset, AssetServer, Assets, Handle, UntypedAssetId},
    ecs::system::Resource,
    reflect::{FromReflect, Reflect, TypePath, TypeRegistry},
    utils::HashMap,
};
use bevy_animation_graph::{
    core::{
        animation_clip::{EntityPath, GraphClip},
        animation_graph::{AnimationGraph, PinId},
        frame::PoseSpec,
        parameters::{BoneMask, ParamSpec, ParamValue},
    },
    prelude::OrderedMap,
};
use bevy_inspector_egui::{
    egui, inspector_egui_impls::InspectorEguiImpl, reflect_inspector::InspectorUi,
    restricted_world_view::RestrictedWorldView,
};
use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    path::PathBuf,
};

pub struct BetterInspectorPlugin;
impl Plugin for BetterInspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(EguiInspectorBuffers::<
            HashMap<EntityPath, f32>,
            Vec<(EntityPath, f32)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<PinId, ParamValue>,
            Vec<(PinId, ParamValue)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<PinId, ParamSpec>,
            Vec<(PinId, ParamSpec)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<
            OrderedMap<PinId, PoseSpec>,
            Vec<(PinId, PoseSpec)>,
        >::default());
        app.insert_resource(EguiInspectorBuffers::<String>::default());
        app.insert_resource(EguiInspectorBuffers::<EntityPath, String>::default());
        app.register_type::<HashMap<EntityPath, f32>>();
        app.register_type::<OrderedMap<PinId, ParamValue>>();
        app.register_type::<OrderedMap<PinId, ParamSpec>>();
        app.register_type::<OrderedMap<PinId, PoseSpec>>();

        let type_registry = app.world.resource::<bevy::ecs::prelude::AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        let type_registry = &mut type_registry;

        add_no_many::<String>(type_registry, string_mut, string_readonly);
        add_no_many::<Handle<AnimationGraph>>(
            type_registry,
            asset_picker_ui::<AnimationGraph>,
            todo_readonly_ui,
        );
        add_no_many::<Handle<GraphClip>>(
            type_registry,
            asset_picker_ui::<GraphClip>,
            todo_readonly_ui,
        );
        add_no_many::<EntityPath>(type_registry, entity_path_mut, entity_path_readonly);
        add_no_many::<AnimationGraph>(type_registry, animation_graph_mut, todo_readonly_ui);
        add_no_many::<BoneMask>(type_registry, bone_mask_ui_mut, todo_readonly_ui);
        add_no_many::<HashMap<EntityPath, f32>>(
            type_registry,
            better_hashmap::<EntityPath, f32>,
            todo_readonly_ui,
        );
        add_no_many::<OrderedMap<PinId, ParamValue>>(
            type_registry,
            better_ordered_map::<PinId, ParamValue>,
            todo_readonly_ui,
        );
        add_no_many::<OrderedMap<PinId, ParamSpec>>(
            type_registry,
            better_ordered_map::<PinId, ParamSpec>,
            todo_readonly_ui,
        );
        add_no_many::<OrderedMap<PinId, PoseSpec>>(
            type_registry,
            better_ordered_map::<PinId, PoseSpec>,
            todo_readonly_ui,
        );
    }
}

#[derive(Resource, Default)]
struct EguiInspectorBuffers<T, B = T> {
    bufs: HashMap<egui::Id, B>,
    _marker: std::marker::PhantomData<T>,
}

fn get_buffered<'w, T, B>(
    world: &mut RestrictedWorldView<'w>,
    id: egui::Id,
    default: impl FnOnce() -> B,
) -> &'w mut B
where
    T: Send + Sync + Default + Clone + 'static,
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
    _values: &mut [&mut dyn Reflect],
    _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
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
        *value = buffered.clone();
        true
    } else if !response.has_focus() {
        *buffered = value.clone();
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

pub fn entity_path_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<EntityPath>().unwrap();
    let buffered =
        get_buffered::<EntityPath, String>(env.context.world.as_mut().unwrap(), id, || {
            value.to_slashed_string()
        });

    let response = ui.text_edit_singleline(buffered);

    if response.lost_focus() {
        *value = EntityPath::from_slashed_string(buffered.clone());
        true
    } else if !response.has_focus() {
        *buffered = value.to_slashed_string();
        false
    } else {
        false
    }
}
pub fn entity_path_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    mut _env: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<EntityPath>().unwrap();
    let slashed_path = value.to_slashed_string();
    ui.label(slashed_path);
}

pub fn animation_graph_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<AnimationGraph>().unwrap();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write_u64(unsafe { std::mem::transmute_copy(&id) });
    hasher.write_usize(0);
    let default_params_id = egui::Id::new(hasher.finish());
    hasher.write_usize(0);
    let output_params_id = egui::Id::new(hasher.finish());
    hasher.write_usize(0);
    let input_poses_id = egui::Id::new(hasher.finish());
    hasher.write_usize(0);
    let output_pose_id = egui::Id::new(hasher.finish());

    let mut default_params_changed = false;
    let mut output_params_changed = false;
    let mut input_poses_changed = false;
    let mut output_pose_changed = false;
    //ui.heading("Default input parameters");
    ui.collapsing("Default input parameters", |ui| {
        default_params_changed = env.ui_for_reflect_with_options(
            &mut value.default_parameters,
            ui,
            default_params_id,
            &(),
        );
    });
    ui.collapsing("Output parameters", |ui| {
        output_params_changed = env.ui_for_reflect_with_options(
            &mut value.output_parameters,
            ui,
            output_params_id,
            &(),
        );
    });
    ui.collapsing("Input poses", |ui| {
        input_poses_changed =
            env.ui_for_reflect_with_options(&mut value.input_poses, ui, input_poses_id, &());
    });
    ui.collapsing("Output pose", |ui| {
        output_pose_changed =
            env.ui_for_reflect_with_options(&mut value.output_pose, ui, output_pose_id, &());
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
    let bones_id = egui::Id::new(hasher.finish());

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
                let key_id = egui::Id::new(hasher.finish());
                hasher.write_usize(0);
                let value_id = egui::Id::new(hasher.finish());

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
    let value = value.downcast_mut::<OrderedMap<K, T>>().unwrap();
    let buffered = get_buffered::<OrderedMap<K, T>, Vec<(K, T)>>(
        env.context.world.as_mut().unwrap(),
        id,
        || value.clone().into_iter().collect(),
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
                let key_id = egui::Id::new(hasher.finish());
                hasher.write_usize(0);
                let value_id = egui::Id::new(hasher.finish());

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
        *value = buffered.clone().into_iter().collect();
        *buffered = value.clone().into_iter().collect();
        true
    } else {
        false
    }
}

pub fn asset_picker_ui<T: Asset>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Handle<T>>().unwrap();
    let value_id = value.id();

    let (mut world_t_assets, world) = env
        .context
        .world
        .as_mut()
        .unwrap()
        .split_off_resource(TypeId::of::<Assets<T>>());
    let t_assets = world_t_assets.get_resource_mut::<Assets<T>>().unwrap();
    let (asset_server, _) = world.split_off_resource_typed::<AssetServer>().unwrap();

    let mut selected = value_id;
    egui::ComboBox::from_id_source(&value)
        .selected_text(if t_assets.contains(selected) {
            asset_server
                .get_path(selected)
                .map(|p| p.path().to_string_lossy().into())
                .unwrap_or("Unsaved Asset".to_string())
        } else {
            "None".to_string()
        })
        .show_ui(ui, |ui| {
            let mut assets_ids: Vec<_> = t_assets.ids().collect();
            assets_ids.sort();
            for asset_id in assets_ids {
                ui.selectable_value(
                    &mut selected,
                    asset_id,
                    asset_server
                        .get_path(asset_id)
                        .unwrap()
                        .path()
                        .to_str()
                        .unwrap(),
                );
            }
        });
    if selected != value_id {
        *value = asset_server.get_id_handle(selected).unwrap();
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

pub fn handle_path(handle: UntypedAssetId, asset_server: &AssetServer) -> PathBuf {
    asset_server
        .get_path(handle)
        .map_or("Unsaved Asset".into(), |p| p.path().to_owned())
}

pub fn handle_name(handle: UntypedAssetId, asset_server: &AssetServer) -> String {
    asset_server
        .get_path(handle)
        .map_or("Unsaved Asset".to_string(), |p| {
            p.path().to_str().unwrap().into()
        })
}
