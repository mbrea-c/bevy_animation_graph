use bevy::{
    asset::{Asset, AssetServer, Assets, Handle, UntypedAssetId},
    reflect::{FromReflect, Reflect, TypePath, TypeRegistry},
    utils::HashMap,
};
use bevy_animation_graph::core::{
    animation_clip::{EntityPath, GraphClip},
    animation_graph::{AnimationGraph, PinId},
    frame::PoseSpec,
    parameters::{ParamSpec, ParamValue},
};
use bevy_inspector_egui::{
    egui, inspector_egui_impls::InspectorEguiImpl, reflect_inspector::InspectorUi,
};
use std::any::{Any, TypeId};

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

pub fn register_editor_impls(type_registry: &mut TypeRegistry) {
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
    add_no_many::<EntityPath>(type_registry, entity_path_ui_mut, entity_path_ui_readonly);
    add_no_many::<AnimationGraph>(type_registry, graph_ui_mut, todo_readonly_ui);
}

pub fn entity_path_ui_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    mut _env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<EntityPath>().unwrap();
    let mut slashed_path = value.to_slashed_string();

    let response = ui.text_edit_singleline(&mut slashed_path);

    if response.changed() {
        *value = EntityPath::from_slashed_string(slashed_path);
        true
    } else {
        false
    }
}
pub fn entity_path_ui_readonly(
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

pub fn graph_ui_mut(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<AnimationGraph>().unwrap();

    ui.heading("Default input parameters");
    let default_params_changed =
        better_hashmap::<ParamValue>(&mut value.default_parameters, ui, &mut env);
    ui.heading("Output parameters");
    let output_params_changed =
        better_hashmap::<ParamSpec>(&mut value.output_parameters, ui, &mut env);
    ui.heading("Input poses");
    let input_poses_changed = better_hashmap::<PoseSpec>(&mut value.input_poses, ui, &mut env);
    ui.heading("Output pose");
    let output_pose_changed = env.ui_for_reflect(&mut value.output_pose, ui);

    default_params_changed || output_params_changed || input_poses_changed || output_pose_changed
}

pub fn better_hashmap<T: Clone + FromReflect + TypePath + Default>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    env: &mut InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<HashMap<PinId, T>>().unwrap();

    let mut changed = false;

    let mut values_vec: Vec<_> = value.clone().into_iter().collect();
    values_vec.sort_by_key(|(k, _)| k.clone());

    for (i, (mut k, mut v)) in values_vec.into_iter().enumerate() {
        ui.push_id(i, |ui| {
            ui.separator();
            let old_k = k.clone();

            let key_changed = ui.text_edit_singleline(&mut k);
            let value_changed = env.ui_for_reflect(&mut v, ui);

            if key_changed.changed() || value_changed {
                value.remove(&old_k);
                value.insert(k.clone(), v);
                changed = true;
            }
        });
    }
    ui.separator();
    if ui.button("+ Add").clicked() {
        value.insert(PinId::default(), T::default());
        changed = true;
    }
    ui.separator();

    changed
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

pub fn handle_name(handle: UntypedAssetId, asset_server: &AssetServer) -> String {
    asset_server
        .get_path(handle)
        .map_or("Unsaved Asset".to_string(), |p| {
            p.path().to_str().unwrap().into()
        })
}
