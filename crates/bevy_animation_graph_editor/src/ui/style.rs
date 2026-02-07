use std::any::{Any, TypeId};

use bevy::platform::collections::{HashMap, HashSet};

#[derive(Hash, Clone, Copy)]
pub struct StyleModifiers(u64);

bitflags::bitflags! {
    impl StyleModifiers: u64 {
        const HOVERED  = 0b00000001;
        const SELECTED = 0b00000010;
    }
}

pub trait StyleObject: Default + Clone + Any + Send + Sync {
    fn merge(&self, other: &Self) -> Self;
    fn base() -> Self;
}

#[derive(Clone)]
pub struct StyleRule<T> {
    pub modifiers: StyleModifiers,
    pub classes: HashSet<String>,
    pub value: T,
}

impl<T> StyleRule<T> {
    pub fn val(value: T) -> Self {
        Self {
            modifiers: StyleModifiers::empty(),
            classes: HashSet::new(),
            value,
        }
    }

    pub fn with_modifiers(mut self, modifiers: StyleModifiers) -> Self {
        self.modifiers |= modifiers;
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.classes.insert(class.into());
        self
    }
}

trait DynCloneRule: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn DynCloneRule>;
}

impl<T: StyleObject + Clone> DynCloneRule for StyleRule<T> {
    fn clone_box(&self) -> Box<dyn DynCloneRule> {
        Box::new(self.clone())
    }
}

impl<T> StyleRule<T> {
    pub fn matches(&self, modifiers: StyleModifiers, classes: &HashSet<String>) -> bool {
        modifiers.contains(self.modifiers)
            && self.classes.iter().all(|class| classes.contains(class))
    }
}

pub struct DynStyleRule {
    value: Box<dyn DynCloneRule + Send + Sync>,
}

impl DynStyleRule {
    pub fn new<T>(rule: StyleRule<T>) -> Self
    where
        T: StyleObject + Clone + Send + Sync,
    {
        Self {
            value: Box::new(rule),
        }
    }

    pub fn get_static<T: StyleObject + Clone>(&self) -> Option<&StyleRule<T>> {
        let val: &dyn Any = self.value.as_ref();
        val.downcast_ref::<StyleRule<T>>()
    }
}

impl Clone for DynStyleRule {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone_box(),
        }
    }
}

#[derive(Default, Clone)]
pub struct StyleEngine {
    by_type: HashMap<TypeId, Vec<DynStyleRule>>,
}

impl StyleEngine {
    pub fn add_rule<T: StyleObject>(&mut self, rule: StyleRule<T>) -> &mut Self {
        let ty = TypeId::of::<T>();
        self.by_type
            .entry(ty)
            .or_default()
            .push(DynStyleRule::new(rule));
        self
    }

    pub fn evaluate<T: StyleObject>(
        &self,
        modifiers: StyleModifiers,
        classes: &HashSet<String>,
    ) -> T {
        let ty = TypeId::of::<T>();
        self.by_type
            .get(&ty)
            .map(|rules| {
                rules
                    .iter()
                    .filter_map(|dynrule| dynrule.get_static::<T>())
                    .filter(|rule| rule.matches(modifiers, classes))
                    .map(|rule| &rule.value)
                    .fold(T::base(), |acc: T, v| acc.merge(v))
            })
            .unwrap_or_else(|| T::base())
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn path_stroke(
    stroke: egui::Stroke,
    kind: egui::epaint::StrokeKind,
) -> egui::epaint::PathStroke {
    egui::epaint::PathStroke {
        width: stroke.width,
        color: egui::epaint::ColorMode::Solid(stroke.color),
        kind,
    }
}
