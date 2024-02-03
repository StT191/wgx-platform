
use std::sync::Arc;
use crate::winit;
use winit::event_loop::{
    EventLoopBuilder, EventLoop as WinitEventLoop,
    EventLoopProxy as WinitEventLoopProxy, EventLoopWindowTarget as WinitEventLoopWindowTarget,
};
use winit::{event::Event as WinitEventType, window::{Window as WinitWindow, WindowBuilder}};
use crate::*;


// exports

#[cfg(target_family="wasm")]
pub use winit::platform::web::{WindowExtWebSys, WindowBuilderExtWebSys};


#[derive(Debug)]
pub enum EventExt {
    #[cfg(target_family="wasm")] ClipboardPaste,
    #[cfg(target_family="wasm")] ClipboardFetch,
}

pub type EventLoop = WinitEventLoop<EventExt>;
pub type EventLoopProxy = WinitEventLoopProxy<EventExt>;
pub type EventLoopWindowTarget = WinitEventLoopWindowTarget<EventExt>;
pub type WinitEvent = WinitEventType<EventExt>;


// platform functions

pub fn init(log_level: LogLevel) {
    #[cfg(not(target_family="wasm"))] {
        simple_logger::init_with_level(log_level).unwrap();
    }

    #[cfg(target_family="wasm")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log_level).expect("could not initialize logger");
    }
}


pub fn event_loop() -> EventLoop {
    EventLoopBuilder::with_user_event().build().unwrap()
}


pub async fn window(window_builder: WindowBuilder, target: &EventLoopWindowTarget) -> Arc<WinitWindow> {

    #[cfg(not(target_family="wasm"))] {
        let window = Arc::new(
            window_builder
            .build(target)
            .unwrap()
        );
        window
    }

    #[cfg(target_family="wasm")] {

        let window = Arc::new(
            window_builder
            .with_prevent_default(false)
            .build(target)
            .unwrap()
        );


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


        // wait for resize events
        wasm_bindgen_futures::JsFuture::from(
            js_sys::Promise::new(&mut |resolve: js_sys::Function, _: js_sys::Function| {
                web_sys::window().unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50).unwrap();
            })
        ).await.unwrap();


        window
    }
}
