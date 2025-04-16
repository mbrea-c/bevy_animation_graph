use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::edge_data::{AnimationEvent, SampledEvent};

#[derive(Debug, Reflect, Clone, Serialize, Deserialize, Default)]
pub struct TrackItem {
    pub id: Uuid,
    pub value: TrackItemValue,
}

#[derive(Debug, Reflect, Clone, Serialize, Deserialize, Default)]
pub struct TrackItemValue {
    pub event: AnimationEvent,
    pub start_time: f32,
    pub end_time: f32,
}

// **IMPORTANT**
//
// Only use this for caching/buffering. There are reasons f32 doesn't implement Hash.
impl core::hash::Hash for TrackItemValue {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.event.hash(state);
        self.start_time.to_bits().hash(state);
        self.end_time.to_bits().hash(state);
    }
}

impl TrackItem {
    pub fn new(value: TrackItemValue) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
        }
    }

    fn duration(&self) -> f32 {
        self.value.end_time - self.value.start_time
    }

    fn time_at_percentage(&self, percentage: f32) -> f32 {
        self.value.start_time + self.duration() * percentage
    }

    fn percentage_at_time(&self, time: f32) -> f32 {
        (time - self.value.start_time) / self.duration()
    }
}

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub struct EventTrack {
    pub name: String,
    pub events: Vec<TrackItem>,
}

impl EventTrack {
    /// Adds an item to the track
    ///
    /// This operation is O(n) on the number of existing items in the track.
    pub fn add_item_preassigned_id(&mut self, item: TrackItem) {
        // The items are sorted by start time
        for i in 0..self.events.len() {
            if self.events[i].value.start_time > item.value.start_time {
                self.events.insert(i, item);
                return;
            }
        }
        self.events.push(item)
    }

    /// Adds an item to the track
    ///
    /// This operation is O(n) on the number of existing items in the track.
    pub fn add_item(&mut self, value: TrackItemValue) {
        self.add_item_preassigned_id(TrackItem::new(value));
    }

    /// Get a list of events active at the current time
    pub fn sample(&self, time: f32) -> Vec<SampledEvent> {
        self.events
            .iter()
            .filter(|ev| ev.value.start_time <= time && ev.value.end_time > time)
            .map(|ev| SampledEvent {
                event: ev.value.event.clone(),
                weight: 1.,
                percentage: ev.percentage_at_time(time),
                track: Some(self.name.clone()),
            })
            .collect()
    }

    /// Given a percentage of the total duration of the event, return the time
    /// in the track that matches such an event.
    ///
    /// If more than one occurrence of the given event is found, the time returned will correspond
    /// to the one with the earliest start time.
    pub fn seek_event(&self, event: &AnimationEvent, percent: f32) -> Option<f32> {
        let track_item = self.events.iter().find(|ev| &ev.value.event == event)?;
        Some(track_item.time_at_percentage(percent))
    }

    pub fn get_track_item(&self, index: usize) -> Option<&TrackItem> {
        self.events.get(index)
    }

    /// To ensure we respect the invariants, this will remove the event, apply the editing, and
    /// re-insert it after.
    pub fn edit_item(&mut self, id: Uuid, f: impl FnOnce(TrackItemValue) -> TrackItemValue) {
        if let Some(index) = self.events.iter().position(|e| e.id == id) {
            let old_item = self.events.remove(index);
            let new_item = f(old_item.value);
            self.add_item_preassigned_id(TrackItem {
                id,
                value: new_item,
            });
        }
    }
}

pub fn sample_tracks<'a>(
    tracks: impl IntoIterator<Item = &'a EventTrack>,
    time: f32,
) -> Vec<SampledEvent> {
    tracks
        .into_iter()
        .flat_map(|track| track.sample(time))
        .collect()
}
