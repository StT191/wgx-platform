
use std::future::Future;
use crate::winit::{event_loop::EventLoop, window::Window as WinitWindow};
use crate::{Static, LogLevel};


pub fn main<F: Future<Output=()> + 'static>(run: impl FnOnce(Static<WinitWindow>, EventLoop<()>) -> F, level: LogLevel) {

  // init + logger

  #[cfg(not(target_family="wasm"))] {
    simple_logger::init_with_level(level).unwrap();
  }

  #[cfg(target_family="wasm")] {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(level).expect("could not initialize logger");
  }


  // setup window
  let event_loop = EventLoop::new();
  let window: Static<_> = WinitWindow::new(&event_loop).unwrap().into();


  // further init and run

  #[cfg(not(target_family="wasm"))] {
    pollster::block_on(run(window, event_loop));
  }

  #[cfg(target_family="wasm")] {

    use crate::winit::{platform::web::WindowExtWebSys, dpi::PhysicalSize};
    use wasm_bindgen::{JsValue, closure::Closure};
    use web_sys::{Event, Window as WebWindow};


    fn set_canvas_size(w_win: Static<WinitWindow>, dom_win: &WebWindow) {

      let width = dom_win.inner_width().unwrap().as_f64().unwrap() as f32;
      let height = dom_win.inner_height().unwrap().as_f64().unwrap() as f32;

      let sf = w_win.scale_factor() as f32;

      w_win.set_inner_size(PhysicalSize::<f32>::from((sf*width, sf*height)));
    }

    web_sys::window()
    .and_then(|win| {
      set_canvas_size(window, &win);
      win.document()
    })
    .and_then(|doc| doc.body())
    .and_then(|body| {

      // remove previous elements
      while let Some(child) = body.last_child() {
        body.remove_child(&child).unwrap();
      }

      // set css styles
      body.set_attribute("style", "margin: 0; overflow: hidden;").unwrap();
      body.set_inner_html("<style>canvas{touch-action:none; max-width:100vw}</style>");

      // resize event handling closure
      let clos: Box<dyn Fn(Event)> = Box::new(move |evt: Event| {
        let dom_win: WebWindow = JsValue::from(evt.target().unwrap()).into();
        set_canvas_size(window, &dom_win);
      });

      body.set_onresize(Some(&Closure::wrap(clos).into_js_value().into()));

      // append canvas to body
      body.append_child(&web_sys::Element::from(window.canvas())).ok()
    })
    .expect("couldn't append canvas to document body");

    wasm_bindgen_futures::spawn_local(run(window, event_loop));
  }
}
