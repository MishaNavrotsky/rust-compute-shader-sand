use std::sync::Arc;

use crate::graphics::{create_graphics, structures::CursorState, structures::Graphics};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

enum State {
    Ready(Graphics),
    Init(Option<EventLoopProxy<Graphics>>),
}

pub struct App {
    state: State,
}

impl App {
    pub fn new(event_loop: &EventLoop<Graphics>) -> Self {
        Self {
            state: State::Init(Some(event_loop.create_proxy())),
        }
    }

    fn draw(&mut self) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.draw();
            gfx.request_redraw();
        }
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.resize(size);
        }
    }

    fn cursor_moved(&mut self, position: &PhysicalPosition<f64>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.set_mouse_pos([position.x as f32, position.y as f32]);
        }
    }

    fn cursor_clicked(&mut self, cursor_state: CursorState) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.set_cursor_state(cursor_state);
        }
    }
}

impl ApplicationHandler<Graphics> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let State::Init(proxy) = &mut self.state {
            if let Some(proxy) = proxy.take() {
                let mut win_attr = Window::default_attributes();

                win_attr = win_attr.with_title("WebGPU example");

                let window = Arc::new(
                    event_loop
                        .create_window(win_attr)
                        .expect("create window err."),
                );
                pollster::block_on(create_graphics(window, proxy));
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
        graphics.request_redraw();
        self.state = State::Ready(graphics);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    self.cursor_clicked(CursorState::Pressed);
                }
                (MouseButton::Left, ElementState::Released) => {
                    self.cursor_clicked(CursorState::Default);
                }
                _ => {}
            },
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => self.cursor_moved(&position),
            _ => {}
        }
    }
}
