
use std::future::Future;
use crate::winit::{event_loop::{EventLoop as WinitEventLoop, EventLoopBuilder}, event::Event as WinitEvent, window::Window as WinitWindow};
use crate::LogLevel;

#[derive(Debug)]
pub enum EventExt {
    #[cfg(target_family="wasm")] ClipboardFilled,
}

pub type EventLoop = WinitEventLoop<EventExt>;
pub type Event<'a> = WinitEvent<'a, EventExt>;


pub fn main<F: Future<Output=()> + 'static>(run: impl FnOnce(&'static WinitWindow, EventLoop) -> F, level: LogLevel) {

    // init + logger

    #[cfg(not(target_family="wasm"))] {
        simple_logger::init_with_level(level).unwrap();
    }

    #[cfg(target_family="wasm")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(level).expect("could not initialize logger");
    }


    // setup event_loop and window
    let event_loop = EventLoopBuilder::with_user_event().build();

    // further init and run

    #[cfg(not(target_family="wasm"))] {
        let window = &*Box::leak(WinitWindow::new(&event_loop).expect("couldn't create window").into());
        pollster::block_on(run(window, event_loop));
    }

    #[cfg(target_family="wasm")] {

        use crate::winit::{platform::web::{WindowExtWebSys, WindowBuilderExtWebSys}, window::WindowBuilder, dpi::PhysicalSize};
        use wasm_bindgen::{JsValue, closure::Closure};
        use web_sys::{Window as WebWindow, Event};

        let window = &*Box::leak(
            WindowBuilder::new()
            .with_prevent_default(false)
            .build(&event_loop)
            .expect("couldn't create window").into()
        );

        let set_window_size = |web_window: &WebWindow| {

            let width = web_window.inner_width().unwrap().as_f64().unwrap() as f32;
            let height = web_window.inner_height().unwrap().as_f64().unwrap() as f32;

            let sf = window.scale_factor() as f32;

            window.set_inner_size(PhysicalSize::<f32>::from((sf*width, sf*height)));
        };

        web_sys::window().and_then(|web_window| {

            let body = web_window.document().and_then(|document| document.body()).unwrap();

            // remove previous elements
            while let Some(child) = body.last_child() {
                body.remove_child(&child).unwrap();
            }

            // set css styles
            body.set_attribute("style", "margin: 0; overflow: hidden;").unwrap();


            // resize event handling closure
            let closure: Box<dyn Fn(Event)> = Box::new(move |evt| {
                set_window_size(&JsValue::from(evt.target().unwrap()).into());
            });

            let event_listener = Closure::wrap(closure).into_js_value().into();

            web_window.add_event_listener_with_callback("resize", &event_listener).unwrap();

            set_window_size(&web_window);


            // append canvas to body
            let canvas_element = web_sys::HtmlElement::from(window.canvas());
            canvas_element.set_attribute("style", "touch-action: none; max-width: 100vw; outline: none").unwrap();

            body.append_child(&canvas_element).and_then(|_| {
                canvas_element.focus() // initial focus
            }).ok()
        })
        .expect("couldn't append canvas to document body");

        wasm_bindgen_futures::spawn_local(run(window, event_loop));
    }
}
