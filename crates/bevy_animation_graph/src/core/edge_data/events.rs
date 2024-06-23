use bevy::reflect::{std_traits::ReflectDefault, Reflect};
use serde::{Deserialize, Serialize};

/// Event data
#[derive(Clone, Debug, Reflect, Serialize, Deserialize, Default)]
#[reflect(Default)]
pub struct AnimationEvent {
    pub id: String,
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
}

impl Default for SampledEvent {
    fn default() -> Self {
        Self {
            event: AnimationEvent::default(),
            weight: 1.,
            percentage: 1.,
        }
    }
}

impl SampledEvent {
    pub fn instant(event: AnimationEvent) -> Self {
        Self {
            event,
            weight: 1.,
            percentage: 1.,
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
}
