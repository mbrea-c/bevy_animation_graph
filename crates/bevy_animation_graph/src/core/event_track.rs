use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::edge_data::{AnimationEvent, SampledEvent};

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub struct TrackItem {
    pub event: AnimationEvent,
    pub start_time: f32,
    pub end_time: f32,
}

impl TrackItem {
    fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }

    fn time_at_percentage(&self, percentage: f32) -> f32 {
        self.start_time + self.duration() * percentage
    }

    fn percentage_at_time(&self, time: f32) -> f32 {
        (time - self.start_time) / self.duration()
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
    pub fn add_item(&mut self, item: TrackItem) {
        // The items are sorted by start time
        for i in 0..self.events.len() {
            if self.events[i].start_time > item.start_time {
                self.events.insert(i, item);
                return;
            }
        }
        self.events.push(item)
    }

    /// Get a list of events active at the current time
    pub fn sample(&self, time: f32) -> Vec<SampledEvent> {
        self.events
            .iter()
            .filter(|ev| ev.start_time <= time && ev.end_time > time)
            .map(|ev| SampledEvent {
                event: ev.event.clone(),
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
        let track_item = self.events.iter().filter(|ev| &ev.event == event).next()?;
        Some(track_item.time_at_percentage(percent))
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
