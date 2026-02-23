//! Experimenting with a custom reflect library

use std::any::{Any, TypeId};

use bevy::{
    ecs::{reflect::AppTypeRegistry, resource::Resource, world::World},
    platform::collections::HashMap,
    reflect::{
        DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant, EnumInfo, Reflect, TypeInfo,
        TypeRegistry, VariantType, prelude::ReflectDefault,
    },
};

use crate::ui::generic_widgets::picker::PickerWidget;

pub mod default_registry;

#[derive(Default, Resource)]
pub struct WidgetRegistry {
    widgets: HashMap<TypeId, DynWidget>,
}

impl WidgetRegistry {
    pub fn add<T>(&mut self, widget: T) -> &mut Self
    where
        T: ReflectWidget,
        <T as ReflectWidget>::Target: 'static,
    {
        self.widgets
            .insert(TypeId::of::<T::Target>(), DynWidget::wrap(widget));
        self
    }

    pub fn layered<'a>(&'a self) -> LayeredRegistry<'a> {
        LayeredRegistry {
            overlay: self,
            base: None,
        }
    }
}

pub struct LayeredRegistry<'a> {
    overlay: &'a WidgetRegistry,
    base: Option<&'a LayeredRegistry<'a>>,
}

impl<'a> LayeredRegistry<'a> {
    pub fn get_for(&self, id: TypeId) -> Option<&DynWidget> {
        self.overlay
            .widgets
            .get(&id)
            .or_else(|| self.base.and_then(|b| b.get_for(id)))
    }
}

#[derive(Default)]
pub struct ExternalContexts<'a> {
    contexts: HashMap<egui::Id, &'a dyn Any>,
}

pub struct ReflectWidgetContext<'a> {
    external: &'a ExternalContexts<'a>,
    registry: &'a LayeredRegistry<'a>,
    bevy_registry: &'a TypeRegistry,
}

impl<'a> ReflectWidgetContext<'a> {
    pub fn draw(&self, ui: &mut egui::Ui, value: &mut (dyn Reflect + 'static)) -> egui::Response {
        let type_id = value.as_any().type_id();

        if let Some(widget) = self.registry.get_for(type_id) {
            return (widget.draw_fn)(widget.widget.as_ref(), ui, value, self);
        }

        let Some(type_info) = self.bevy_registry.get_type_info(type_id) else {
            return ui.label(format!(
                "[ERROR] Reflect type info not found for {}",
                value.reflect_short_type_path()
            ));
        };

        ui.scope(|ui| {
            let mut response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::empty());
            match type_info {
                TypeInfo::Struct(struct_info) => {
                    let r_struct = value.reflect_mut().as_struct().unwrap();

                    egui::Grid::new("struct").show(ui, |ui| {
                        for field_name in struct_info.field_names() {
                            let Some(field) = r_struct
                                .field_mut(field_name)
                                .and_then(|f| f.try_as_reflect_mut())
                            else {
                                continue;
                            };

                            response |= ui.label(format!("{}:", field_name));
                            response |= self.draw(ui, field);

                            ui.end_row();
                        }
                    });
                }
                TypeInfo::TupleStruct(tuple_struct_info) => {
                    let tuple_struct = value.reflect_mut().as_tuple_struct().unwrap();

                    for field_idx in 0..tuple_struct_info.field_len() {
                        let Some(field) = tuple_struct
                            .field_mut(field_idx)
                            .and_then(|f| f.try_as_reflect_mut())
                        else {
                            continue;
                        };

                        response |= self.draw(ui, field);
                    }
                }
                TypeInfo::Tuple(tuple_info) => {
                    response |= ui.label("Tuples not implemented yet");
                }
                TypeInfo::List(list_info) => {
                    response |= ui.label("Lists not implemented yet");
                }
                TypeInfo::Array(array_info) => {
                    response |= ui.label("Arrays not implemented yet");
                }
                TypeInfo::Map(map_info) => {
                    response |= ui.label("Maps not implemented yet");
                }
                TypeInfo::Set(set_info) => {
                    response |= ui.label("Sets not implemented yet");
                }
                TypeInfo::Enum(enum_info) => {
                    let r_enum = value.reflect_mut().as_enum().unwrap();

                    let mut current_value = r_enum.variant_name();

                    let picker_response = PickerWidget::new_salted("enum variant picker").ui(
                        ui,
                        r_enum.variant_name(),
                        |ui| {
                            for variant_name in enum_info.variant_names() {
                                let has_default_value =
                                    self.default_variant(*variant_name, enum_info).is_some();

                                let variant_response = ui
                                    .add_enabled_ui(has_default_value, |ui| {
                                        ui.selectable_value(
                                            &mut current_value,
                                            *variant_name,
                                            *variant_name,
                                        )
                                    })
                                    .inner;

                                if variant_response.changed() {
                                    response.mark_changed();
                                }
                            }
                        },
                    );

                    response |= picker_response.response;

                    if response.changed()
                        && let Some(default_value) = self.default_variant(current_value, enum_info)
                    {
                        response.mark_changed();
                        r_enum.apply(&default_value);
                    }

                    let variant_info = enum_info.variant(r_enum.variant_name()).unwrap();

                    match r_enum.variant_type() {
                        VariantType::Struct => {
                            let struct_variant_info = variant_info.as_struct_variant().unwrap();
                            egui::Grid::new("struct").show(ui, |ui| {
                                for field_name in struct_variant_info.field_names() {
                                    let Some(field) = r_enum
                                        .field_mut(field_name)
                                        .and_then(|f| f.try_as_reflect_mut())
                                    else {
                                        continue;
                                    };

                                    response |= ui.label(format!("{}:", field_name));
                                    response |= self.draw(ui, field);

                                    ui.end_row();
                                }
                            });
                        }
                        VariantType::Tuple => {
                            let tuple_variant_info = variant_info.as_tuple_variant().unwrap();
                            for field_idx in 0..tuple_variant_info.field_len() {
                                let Some(field) = r_enum
                                    .field_at_mut(field_idx)
                                    .and_then(|f| f.try_as_reflect_mut())
                                else {
                                    continue;
                                };

                                response |= self.draw(ui, field);
                            }
                        }
                        VariantType::Unit => {}
                    }
                }
                TypeInfo::Opaque(_opaque_info) => {
                    response |= ui.label("Opaque not implemented yet");
                }
            }
            response
        })
        .inner
    }

    fn default_variant(&self, variant_name: &str, enum_info: &EnumInfo) -> Option<DynamicEnum> {
        let Some(variant_info) = enum_info.variant(variant_name) else {
            return None;
        };

        match variant_info.variant_type() {
            VariantType::Struct => {
                let struct_variant_info = variant_info.as_struct_variant().unwrap();

                let mut dyn_struct = DynamicStruct::default();

                for field_name in struct_variant_info.field_names() {
                    let Some(field_info) = struct_variant_info.field(field_name) else {
                        return None;
                    };
                    let Some(field_type_registration) =
                        self.bevy_registry.get(field_info.type_id())
                    else {
                        return None;
                    };

                    let Some(reflect_default) = field_type_registration.data::<ReflectDefault>()
                    else {
                        return None;
                    };

                    dyn_struct.insert_boxed(*field_name, reflect_default.default());
                }

                Some(DynamicEnum::new(variant_name, dyn_struct))
            }
            VariantType::Tuple => {
                let tuple_variant_info = variant_info.as_tuple_variant().unwrap();

                let mut dyn_tuple = DynamicTuple::default();

                for field_idx in 0..tuple_variant_info.field_len() {
                    let Some(field_info) = tuple_variant_info.field_at(field_idx) else {
                        return None;
                    };
                    let Some(field_type_registration) =
                        self.bevy_registry.get(field_info.type_id())
                    else {
                        return None;
                    };

                    let Some(reflect_default) = field_type_registration.data::<ReflectDefault>()
                    else {
                        return None;
                    };

                    dyn_tuple.insert_boxed(reflect_default.default());
                }

                Some(DynamicEnum::new(variant_name, dyn_tuple))
            }
            VariantType::Unit => Some(DynamicEnum::new(variant_name, DynamicVariant::Unit)),
        }
    }

    pub fn scope<T>(world: &'a World, scoped: impl FnOnce(&ReflectWidgetContext) -> T) -> T {
        let contexts = ExternalContexts::default();

        let out = Self::scope_with_context(world, &contexts, scoped);

        out
    }

    pub fn scope_with_context<T>(
        world: &'a World,
        external_contexts: &ExternalContexts<'a>,
        scoped: impl FnOnce(&ReflectWidgetContext) -> T,
    ) -> T {
        let bevy_registry = world.resource::<AppTypeRegistry>().read();
        let registry = world.resource::<WidgetRegistry>().layered();

        let ctx = ReflectWidgetContext {
            external: external_contexts,
            registry: &registry,
            bevy_registry: &bevy_registry,
        };

        let out = scoped(&ctx);

        out
    }
}

pub trait ReflectWidget: Send + Sync + 'static {
    type Target;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        ctx: &ReflectWidgetContext,
    ) -> egui::Response;
}

pub struct DynWidget {
    widget: Box<dyn Any + Send + Sync>,
    draw_fn: fn(
        widget: &dyn Any,
        ui: &mut egui::Ui,
        value: &mut dyn Any,
        ctx: &ReflectWidgetContext,
    ) -> egui::Response,
}

impl DynWidget {
    pub fn wrap<T>(reflect_widget: T) -> Self
    where
        T: ReflectWidget + Send + Sync + 'static,
        <T as ReflectWidget>::Target: 'static,
    {
        Self {
            widget: Box::new(reflect_widget),
            draw_fn: Self::draw_fn::<T>,
        }
    }

    fn draw_fn<T>(
        widget: &dyn Any,
        ui: &mut egui::Ui,
        value: &mut dyn Any,
        ctx: &ReflectWidgetContext,
    ) -> egui::Response
    where
        T: ReflectWidget + 'static,
        <T as ReflectWidget>::Target: 'static,
    {
        let widget = widget.downcast_ref::<T>().unwrap();
        let value = value
            .downcast_mut::<<T as ReflectWidget>::Target>()
            .unwrap();

        widget.draw(ui, value, ctx)
    }
}
