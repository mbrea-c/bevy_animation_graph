use std::any::Any;

pub trait CloneBuffer<V, M>: Clone + Send + Sync + 'static
where
    V: ?Sized,
{
    fn new(ui: &egui::Ui, meta: &M, value: &V) -> Self;
    fn id(&self, ui: &egui::Ui) -> egui::Id;
    fn is_still_valid(&self, meta: &M, value: &V) -> bool;

    fn id_static(ui: &egui::Ui, meta: &M, value: &V) -> egui::Id {
        Self::new(ui, meta, value).id(ui)
    }
    fn from_ui(ui: &mut egui::Ui, meta: &M, value: &V) -> Self {
        let id = Self::id_static(ui, meta, value);

        if let Some(buffer) = ui.memory_mut(|mem| mem.data.get_temp::<Self>(id))
            && buffer.is_still_valid(meta, value)
        {
            buffer
        } else {
            println!("Buffer invalidated");
            Self::new(ui, meta, value)
        }
    }

    fn save_back(&self, ui: &mut egui::Ui) {
        let id = self.id(ui);
        ui.memory_mut(|mem| {
            mem.data.insert_temp(id, self.clone());
        });
    }
}

pub trait ErasedCloneBuffer: Send + Sync + Any {
    fn clone_box(&self) -> Box<dyn ErasedCloneBuffer>;
}

impl<S: Clone + Send + Sync + 'static> ErasedCloneBuffer for S {
    fn clone_box(&self) -> Box<dyn ErasedCloneBuffer> {
        Box::new(self.clone())
    }
}

pub trait SelfContainedBuffer<V: ?Sized, M>: CloneBuffer<V, M> {
    fn value(&self) -> Box<V>;
}
