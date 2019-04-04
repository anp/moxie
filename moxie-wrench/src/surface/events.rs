use {
    crate::{events::WindowEvents, position::Position},
    futures::{future::AbortHandle, stream::StreamExt},
    log::*,
    moxie::Sender,
    std::task::Waker,
    winit::WindowId,
};

#[derive(Debug, Clone)]
pub struct CursorMoved {
    pub position: Position,
}

pub(crate) async fn dispatch(
    this_window: WindowId,
    mut events: WindowEvents,
    waker: Waker,
    top_level_exit: AbortHandle,
    mut send_mouse_positions: Sender<CursorMoved>,
) {
    'top: while let Some(event) = await!(events.next()) {
        let event = match event.inner {
            winit::Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == this_window => event,
            // we only care about events for this particular window
            _ => continue 'top,
        };
        trace!("handling event {:?}", event);

        use winit::WindowEvent::*;
        match event {
            CloseRequested | Destroyed => {
                info!("close requested or window destroyed. exiting.");
                top_level_exit.abort();
                futures::pending!(); // so nothing else in this task fires accidentally
            }
            Resized(new_size) => {
                debug!("resized: {:?}", new_size);
            }
            Moved(_new_position) => {}
            DroppedFile(_path) => {}
            HoveredFile(_path) => {}
            HoveredFileCancelled => {}
            ReceivedCharacter(_received_char) => {}
            Focused(_in_focus) => {}
            KeyboardInput {
                device_id: _device_id,
                input: _input,
            } => {}
            CursorMoved {
                device_id: _device_id,
                position,
                modifiers: _modifiers,
            } => {
                await!(send_mouse_positions.send(self::CursorMoved {
                    position: (*position).into(),
                }));
            }
            CursorEntered {
                device_id: _device_id,
            } => {}
            CursorLeft {
                device_id: _device_id,
            } => {}
            MouseWheel {
                device_id: _device_id,
                delta: _delta,
                phase: _phase,
                modifiers: _modifiers,
            } => {}

            MouseInput {
                device_id: _device_id,
                state: _state,
                button: _button,
                modifiers: _modifiers,
            } => {}

            TouchpadPressure {
                device_id: _device_id,
                pressure: _pressure,
                stage: _stage,
            } => {}

            AxisMotion {
                device_id: _device_id,
                axis: _axis,
                value: _value,
            } => {}

            Refresh => {
                waker.wake();
            }

            Touch(_touch) => {}
            HiDpiFactorChanged(new_factor) => {
                info!("DPI factor changed, is now {}", new_factor);
            }
        }

        waker.wake();
    }
}
