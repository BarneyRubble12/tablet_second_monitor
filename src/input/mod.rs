use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::error::AppError;
use core_graphics::event::{CGEvent, CGEventType, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use core_graphics::event::CGEventTapLocation;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TouchEvent {
    Start { x: f64, y: f64 },
    Move { x: f64, y: f64 },
    End,
}

pub struct InputHandler {
    tx: broadcast::Sender<TouchEvent>,
}

impl InputHandler {
    pub fn new(tx: broadcast::Sender<TouchEvent>) -> Self {
        Self { tx }
    }

    pub async fn handle_event(&self, event: TouchEvent) -> Result<(), AppError> {
        self.tx.send(event.clone())
            .map_err(|e| AppError::InputError(format!("Failed to send event: {}", e)))?;
        simulate_mouse_event(&event)?;
        Ok(())
    }
}

fn simulate_mouse_event(event: &TouchEvent) -> Result<(), AppError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .expect("Failed to create event source");

    match event {
        TouchEvent::Start { x, y } => {
            let point = CGPoint { x: *x, y: *y };
            let event = CGEvent::new_mouse_event(
                source,
                CGEventType::LeftMouseDown,
                point,
                CGMouseButton::Left,
            ).expect("Failed to create mouse event");
            event.post(CGEventTapLocation::HID);
        }
        TouchEvent::Move { x, y } => {
            let point = CGPoint { x: *x, y: *y };
            let event = CGEvent::new_mouse_event(
                source,
                CGEventType::LeftMouseDragged,
                point,
                CGMouseButton::Left,
            ).expect("Failed to create mouse event");
            event.post(CGEventTapLocation::HID);
        }
        TouchEvent::End => {
            let point = CGPoint { x: 0.0, y: 0.0 };
            let event = CGEvent::new_mouse_event(
                source,
                CGEventType::LeftMouseUp,
                point,
                CGMouseButton::Left,
            ).expect("Failed to create mouse event");
            event.post(CGEventTapLocation::HID);
        }
    }

    Ok(())
} 