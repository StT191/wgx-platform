
// iced and winit dependencies
use iced_wgpu::{Viewport, Renderer, Antialiasing, Settings, Backend};

use iced_winit::winit::{dpi::PhysicalPosition, window::Window, event::{WindowEvent, ModifiersState}};
use iced_winit::{mouse::Interaction, conversion};

#[cfg(target_family = "wasm")]
use iced_winit::winit::event::{KeyboardInput, ElementState, VirtualKeyCode};

use iced_native::{renderer::{Style}, program::{Program, State}, application::StyleSheet, Command, Debug, Size};

#[cfg(target_family = "wasm")]
use iced_native::{Event, keyboard};

// wgpu/wgx and local dependencies
use wgx::wgpu::{CommandEncoder, util::StagingBelt, TextureFormat};
use wgx::{WgxDevice, RenderAttachable};
use super::Clipboard;


// iced renderer constructor

pub fn renderer<T>(gx:&impl WgxDevice, mut settings:Settings, format:TextureFormat, msaa:Option<u32>) -> Renderer<T> {
    settings.antialiasing = match msaa {
        Some(2) => Some(Antialiasing::MSAAx2),
        Some(4) => Some(Antialiasing::MSAAx4),
        Some(8) => Some(Antialiasing::MSAAx8),
        Some(16) => Some(Antialiasing::MSAAx16),
        _ => None,
    };
    Renderer::new(Backend::new(gx.device(), settings, format))
}


// Gui

pub struct Gui<T, P> where
    T: StyleSheet,
    P:'static + Program<Renderer=Renderer<T>>,
{
    renderer: Renderer<T>,
    state: State<P>,
    viewport: Viewport,
    cursor: PhysicalPosition<f64>,
    interaction: Interaction,
    modifiers: ModifiersState,
    pub theme: T,
    pub style: Style,
    clipboard: Clipboard,
    staging_belt: StagingBelt,
    debug: Debug,
}


impl<T, P> Gui<T, P> where
    T: StyleSheet + Default,
    P:'static + Program<Renderer=Renderer<T>>,
{

    pub fn new(mut renderer:Renderer<T>, program:P, (width, height):(u32, u32), window:&Window, clipboard:Clipboard) -> Self {

        let mut debug = Debug::new();

        let viewport = Viewport::with_physical_size(Size::new(width, height), window.scale_factor());

        let cursor = PhysicalPosition::new(-1.0, -1.0);

        let state = State::new(program, viewport.logical_size(), &mut renderer, &mut debug);

        let interaction = state.mouse_interaction();

        Self {
            renderer, state, viewport, cursor, interaction,
            modifiers: ModifiersState::default(),
            theme: T::default(),
            style: Style::default(),
            clipboard,
            staging_belt: StagingBelt::new(10240),
            debug,
        }
    }


    pub fn event(&mut self, event:&WindowEvent) {

        // on wasm we need to track if modifiers changed manually and fire the modifiers changed event manually
        #[cfg(target_family = "wasm")]
        let mut modifiers_changed = false;

        #[cfg(target_family = "wasm")]
        let mut paste = false;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = *position;
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
            }

            // collect modifiers manually on web platform
            #[cfg(target_family = "wasm")]
            #[allow(deprecated)]
            WindowEvent::KeyboardInput { input: KeyboardInput { modifiers, state, virtual_keycode, .. }, ..} => {
                if &self.modifiers != modifiers {
                    self.modifiers = *modifiers;
                    modifiers_changed = true;
                }
                if let (true, Some(VirtualKeyCode::V), ElementState::Pressed) = (modifiers.ctrl(), virtual_keycode, state) {
                    paste = true;
                }
            }

            WindowEvent::Resized(size) => {
                self.viewport = Viewport::with_physical_size(
                    Size::new(size.width, size.height),
                    self.viewport.scale_factor(),
                );
            }

            WindowEvent::ScaleFactorChanged { scale_factor, ref new_inner_size } => {
                self.viewport = Viewport::with_physical_size(
                    Size::new(new_inner_size.width, new_inner_size.height),
                    *scale_factor,
                );
            }

            _ => {}
        }

        #[cfg(target_family = "wasm")]
        if modifiers_changed {
            if let Some(event) = iced_winit::conversion::window_event(
                &WindowEvent::ModifiersChanged(self.modifiers), self.viewport.scale_factor(), self.modifiers,
            ) {
                self.state.queue_event(event);
            }
        }

        #[cfg(target_family = "wasm")]
        if paste && self.clipboard.is_connected() {
            return; // handle events with paste_from_clipboard method
        }

        if let Some(event) = iced_winit::conversion::window_event(
            event, self.viewport.scale_factor(), self.modifiers,
        ) {
            self.state.queue_event(event);
        }
    }


    #[cfg(target_family = "wasm")]
    pub fn paste_from_clipboard(&mut self) {
        self.state.queue_event(Event::Keyboard(keyboard::Event::KeyPressed {
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


    pub fn update(&mut self) -> (bool, Option<Command<P::Message>>) {
        if !self.state.is_queue_empty() {

            let (_events, command) = self.state.update(
                self.viewport.logical_size(),
                conversion::cursor_position(
                    self.cursor,
                    self.viewport.scale_factor(),
                ),
                &mut self.renderer,
                &self.theme,
                &self.style,
                &mut self.clipboard,
                &mut self.debug,
            );

            (true, command)
        }
        else { (false, None) }
    }


    pub fn draw(&mut self, gx:&impl WgxDevice, encoder:&mut CommandEncoder, target:&impl RenderAttachable) {

        // borrow before the closure
        let (staging_belt, viewport, debug) = (&mut self.staging_belt, &self.viewport, &self.debug);

        self.renderer.with_primitives(|backend, primitive| {
            backend.present(
                gx.device(),
                staging_belt,
                encoder,
                target.color_views().0,
                primitive,
                viewport,
                &debug.overlay(),
            );
        });

        self.staging_belt.finish();
    }


    pub fn recall_staging_belt(&mut self) {
        self.staging_belt.recall();
    }
}