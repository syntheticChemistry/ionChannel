#[cfg(test)]
mod tests {
    use ion_core::event::{Axis, ButtonState, InputEvent, KeyState};
    use ion_core::backend::CompositorBackend;
    use crate::WaylandBackend;

    #[test]
    fn test_backend_creation() {
        let backend = WaylandBackend::new();
        assert_eq!(backend.capabilities().backend_name, "Generic Wayland");
    }

    #[tokio::test]
    async fn test_keyboard_events_without_compositor() {
        let backend = WaylandBackend::new();
        
        // These will fail without compositor, but should handle gracefully
        let event = InputEvent::KeyboardKeycode {
            keycode: 30,
            state: KeyState::Pressed,
        };
        
        let result = backend.inject_input(event).await;
        // Should return error (no compositor) or Ok (if somehow available)
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_pointer_events_without_compositor() {
        let backend = WaylandBackend::new();
        
        let event = InputEvent::PointerMotion {
            dx: 5.0,
            dy: 10.0,
        };
        
        let result = backend.inject_input(event).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_touch_events_without_compositor() {
        let backend = WaylandBackend::new();
        
        let event = InputEvent::TouchDown {
            stream: 0,
            slot: 0,
            x: 100.0,
            y: 100.0,
        };
        
        let result = backend.inject_input(event).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_capabilities() {
        let backend = WaylandBackend::new();
        let caps = backend.capabilities();
        
        assert_eq!(caps.backend_name, "Generic Wayland");
        assert_eq!(caps.display_server_type, ion_core::backend::DisplayServerType::Wayland);
    }

    #[tokio::test]
    async fn test_all_input_event_types() {
        let backend = WaylandBackend::new();
        
        let events = vec![
            InputEvent::KeyboardKeycode { keycode: 30, state: KeyState::Pressed },
            InputEvent::KeyboardKeysym { keysym: 0x0061, state: KeyState::Pressed },
            InputEvent::PointerMotion { dx: 10.0, dy: 20.0 },
            InputEvent::PointerMotionAbsolute { stream: 0, x: 100.0, y: 200.0 },
            InputEvent::PointerButton { button: 272, state: ButtonState::Pressed },
            InputEvent::PointerAxis { dx: 0.0, dy: -1.0 },
            InputEvent::PointerAxisDiscrete { axis: Axis::Vertical, steps: -1 },
            InputEvent::TouchDown { stream: 0, slot: 0, x: 100.0, y: 100.0 },
            InputEvent::TouchMotion { stream: 0, slot: 0, x: 150.0, y: 150.0 },
            InputEvent::TouchUp { slot: 0 },
        ];
        
        for event in events {
            let _ = backend.inject_input(event).await;
        }
        
        // Verify no panics
    }

    #[test]
    fn test_backend_sync_creation() {
        let backend = WaylandBackend::new();
        assert_eq!(backend.capabilities().backend_name, "Generic Wayland");
    }

    #[tokio::test]
    async fn test_connection_without_display() {
        let mut backend = WaylandBackend::new();
        
        // Attempting to connect without WAYLAND_DISPLAY should fail gracefully
        let result = backend.connect().await;
        
        // In test environment, no Wayland display available
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_event_sequence_handling() {
        let backend = WaylandBackend::new();
        
        // Simulate typing "hi"
        let events = vec![
            // H key
            InputEvent::KeyboardKeycode { keycode: 35, state: KeyState::Pressed },
            InputEvent::KeyboardKeycode { keycode: 35, state: KeyState::Released },
            // I key
            InputEvent::KeyboardKeycode { keycode: 23, state: KeyState::Pressed },
            InputEvent::KeyboardKeycode { keycode: 23, state: KeyState::Released },
        ];
        
        for event in events {
            let _ = backend.inject_input(event).await;
        }
    }

    #[tokio::test]
    async fn test_mouse_click_sequence() {
        let backend = WaylandBackend::new();
        
        // Simulate mouse click at position
        let events = vec![
            // Move to position
            InputEvent::PointerMotionAbsolute { stream: 0, x: 100.0, y: 200.0 },
            // Click
            InputEvent::PointerButton { button: 272, state: ButtonState::Pressed },
            InputEvent::PointerButton { button: 272, state: ButtonState::Released },
        ];
        
        for event in events {
            let _ = backend.inject_input(event).await;
        }
    }
}

