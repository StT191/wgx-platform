
// iced and winit dependencies
use iced_wgpu::{Settings, Backend};

use iced_winit::{
    winit::{dpi::PhysicalPosition, window::Window, event::WindowEvent, keyboard::ModifiersState},
    graphics::{Renderer, Viewport, Antialiasing},
    runtime::{program::{Program, State}, Command, Debug},
    core::{Event, mouse::{Interaction, Cursor}, Pixels, Size, Font, renderer::Style, window::Id},
    style::{application::StyleSheet},
    conversion,
};

#[cfg(target_family = "wasm")]
use iced_winit::{
    winit::{event::{KeyEvent, ElementState}, keyboard::{PhysicalKey, KeyCode}},
    runtime::keyboard,
};


// wgpu/wgx and local dependencies
use wgx::wgpu::{CommandEncoder, TextureFormat};
use wgx::{WgxDeviceQueue, RenderAttachable};

use super::Clipboard;


// iced renderer constructor

pub fn renderer<T>(gx:&impl WgxDeviceQueue, mut settings:Settings, format:TextureFormat, msaa:Option<u32>) -> Renderer<Backend, T> {
    settings.antialiasing = match msaa {
        Some(2) => Some(Antialiasing::MSAAx2),
        Some(4) => Some(Antialiasing::MSAAx4),
        Some(8) => Some(Antialiasing::MSAAx8),
        Some(16) => Some(Antialiasing::MSAAx16),
        _ => None,
    };
    Renderer::new(
        Backend::new(gx.device(), gx.queue(), settings, format),
        Font::DEFAULT,
        Pixels(12.0),
    )
}


// Gui

pub struct Gui<T, P> where
    T: StyleSheet,
    P:'static + Program<Renderer=Renderer<Backend, T>>,
{
    renderer: Renderer<Backend, T>,
    state: State<P>,
    scale_factor: f64,
    viewport: Viewport,
    cursor: PhysicalPosition<f64>,
    interaction: Interaction,
    modifiers: ModifiersState,
    pub theme: T,
    pub style: Style,
    clipboard: Clipboard,
    debug: Debug,
}


impl<T, P> Gui<T, P> where
    T: StyleSheet + Default,
    P:'static + Program<Renderer=Renderer<Backend, T>>,
{

    pub fn new(mut renderer:Renderer<Backend, T>, program:P, window:&Window, clipboard:Clipboard) -> Self {

        let mut debug = Debug::new();

        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        let viewport = Viewport::with_physical_size(Size::new(size.width, size.height), scale_factor);

        let cursor = PhysicalPosition::new(-1.0, -1.0);

        let state = State::new(program, viewport.logical_size(), &mut renderer, &mut debug);

        let interaction = state.mouse_interaction();

        Self {
            renderer, state, scale_factor, viewport, cursor, interaction,
            modifiers: ModifiersState::default(),
            theme: T::default(),
            style: Style::default(),
            clipboard,
            // staging_belt: StagingBelt::new(10240),
            debug,
        }
    }


    pub fn event(&mut self, event:&WindowEvent) -> bool {

        // on wasm we need to track if modifiers changed manually and fire the modifiers changed event manually
        /*#[cfg(target_family = "wasm")]
        let mut modifiers_changed = false;*/

        #[cfg(target_family = "wasm")]
        let mut paste = false;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = *position;
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }

            // collect modifiers manually on web platform
            #[cfg(target_family = "wasm")]
            WindowEvent::KeyboardInput { event: KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyV), state: ElementState::Pressed, ..
            }, ..} => {
                paste = true;
            }

            WindowEvent::Resized(size) => {
                self.viewport = Viewport::with_physical_size(
                    Size::new(size.width, size.height),
                    self.viewport.scale_factor(),
                );
            }

            WindowEvent::ScaleFactorChanged { scale_factor, ..} => {
                self.scale_factor = *scale_factor;

                self.viewport = Viewport::with_physical_size(
                    Size::new(self.viewport.physical_width(), self.viewport.physical_height()),
                    *scale_factor,
                );
            }

            _ => {}
        }

        /*#[cfg(target_family = "wasm")]
        if modifiers_changed {
            if let Some(event) = iced_winit::conversion::window_event(
                &WindowEvent::ModifiersChanged(self.modifiers), self.viewport.scale_factor(), self.modifiers,
            ) {
                self.state.queue_event(event);
            }
        }*/

        #[cfg(target_family = "wasm")]
        if paste && self.clipboard.is_connected() {
            return false; // handle events with paste_from_clipboard method
        }

        if let Some(event) = iced_winit::conversion::window_event(
            Id::MAIN, event, self.viewport.scale_factor(), self.modifiers,
        ) {
            self.state.queue_event(event);
            true
        }
        else { false }
    }


    #[cfg(target_family = "wasm")]
    pub fn paste_from_clipboard(&mut self) {
        self.state.queue_event(Event::Keyboard(keyboard::Event::KeyPressed {
            text: None,
            key_code: keyboard::KeyCode::V,
            modifiers: keyboard::Modifiers::CTRL,
        }));
    }


    pub fn program(&mut self) -> &P {
        self.state.program()
    }

    pub fn clipboard(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }


    pub fn message(&mut self, message:P::Message) {
        self.state.queue_message(message)
    }


    pub fn update_cursor(&mut self, window:&Window) {
        let interaction = self.state.mouse_interaction();
        if self.interaction != interaction {
            window.set_cursor_icon(conversion::mouse_interaction(interaction));
            self.interaction = interaction;
        }
    }

    pub fn is_queue_empty(&self) -> bool {
        self.state.is_queue_empty()
    }


    pub fn update(&mut self) -> (Vec<Event>, Option<Command<P::Message>>) {
        self.state.update(
            self.viewport.logical_size(),
            Cursor::Available(conversion::cursor_position(
                self.cursor,
                self.viewport.scale_factor(),
            )),
            &mut self.renderer,
            &self.theme,
            &self.style,
            &mut self.clipboard,
            &mut self.debug,
        )
    }


    pub fn draw(&mut self, gx:&impl WgxDeviceQueue, encoder:&mut CommandEncoder, target:&impl RenderAttachable) {

        // borrow before the closure
        let (viewport, debug) = (&self.viewport, &self.debug);

        self.renderer.with_primitives(|backend, primitive| {
            backend.present(
                gx.device(),
                gx.queue(),
                encoder,
                None,
                target.view_format(),
                target.color_views().0,
                primitive,
                viewport,
                &debug.overlay(),
            );
        });
    }
}