use bevy::reflect::{Reflect, std_traits::ReflectDefault};
use serde::{Deserialize, Serialize};

use crate::state_machine::high_level::{StateId, TransitionId};

/// Event data
#[derive(Clone, Debug, Reflect, Serialize, Deserialize, PartialEq, Hash)]
#[reflect(Default)]
pub enum AnimationEvent {
    /// Trigger the most specific transition from transitioning into the provided state. That
    /// will be:
    /// * A direct transition, if present, or
    /// * A global transition, if present
    TransitionToState(StateId),
    /// Trigger a specific transition (if possible)
    Transition(TransitionId),
    EndTransition,
    StringId(String),
}

impl Default for AnimationEvent {
    fn default() -> Self {
        Self::StringId("".to_string())
    }
}

/// Structure containing a sampled event and relevant metadata
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Default)]
pub struct SampledEvent {
    /// Event that was sampled
    pub event: AnimationEvent,
    /// Weight of event (is reduced by blending, for example), 0.0 to 1.0
    pub weight: f32,
    /// Percentage of total event duration at sampling time, 0.0 to 1.0
    pub percentage: f32,
    /// If the event comes from a markup track, contains the track id
    pub track: Option<String>,
}

impl Default for SampledEvent {
    fn default() -> Self {
        Self {
            event: AnimationEvent::default(),
            weight: 1.,
            percentage: 1.,
            track: None,
        }
    }
}

impl SampledEvent {
    pub fn instant(event: AnimationEvent) -> Self {
        Self {
            event,
            weight: 1.,
            percentage: 1.,
            track: None,
        }
    }
}

/// Sequence of events
#[derive(Clone, Debug, Reflect, Serialize, Deserialize, Default)]
#[reflect(Default)]
pub struct EventQueue {
    pub events: Vec<SampledEvent>,
}

impl EventQueue {
    pub fn with_events(events: impl Into<Vec<SampledEvent>>) -> Self {
        Self {
            events: events.into(),
        }
    }

    pub fn concat(mut self, other: EventQueue) -> Self {
        self.events.extend(other.events);
        self
    }
}
