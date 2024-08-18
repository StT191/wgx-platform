
use winit::event_loop::EventLoopBuilder;
use winit::window::{Window as WinitWindow, WindowBuilder};

#[cfg(target_family="wasm")]
use winit::platform::web::{WindowExtWebSys, WindowBuilderExtWebSys};

use crate::*;


pub fn init(log_level: LogLevel) {
    #[cfg(not(target_family="wasm"))] {
        simple_logger::init_with_level(log_level).unwrap();
    }

    #[cfg(target_family="wasm")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log_level).expect("could not initialize logger");
    }
}


pub fn event_loop() -> PlatformEventLoop {
    EventLoopBuilder::with_user_event().build().unwrap()
}


pub fn window(event_loop: &PlatformEventLoopWindowTarget, window_builder: WindowBuilder) -> WinitWindow {

    #[cfg(not(target_family="wasm"))] {
        window_builder.build(event_loop).unwrap()
    }

    #[cfg(target_family="wasm")] {
        window_builder.with_prevent_default(false).build(event_loop).unwrap()
    }
}


#[allow(unused_variables)]
pub fn mount_window(window: &WinitWindow) {
    #[cfg(target_family="wasm")] {

        // web
        let web_window = web_sys::window().unwrap();

        let body = web_window.document().and_then(|document| document.body()).unwrap();

        // remove previous elements
        while let Some(child) = body.last_child() {
            body.remove_child(&child).unwrap();
        }

        // set css styles
        body.set_attribute("style", "margin: 0; overflow: hidden;").unwrap();

        // append canvas to body
        let canvas_element = web_sys::HtmlElement::from(window.canvas().unwrap());

        canvas_element.set_attribute("style", "touch-action: none; width: 100vw; height: 100vh; outline: none").unwrap();

        body.append_child(&canvas_element).unwrap();

        canvas_element.focus().unwrap(); // initial focus
    }
}
