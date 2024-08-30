
use ::platform::winit::{
  window::WindowBuilder,
  event::{WindowEvent, KeyEvent, ElementState},
  keyboard::{PhysicalKey, KeyCode},
};
use wgx::{*, /*cgmath::**/};

use platform::{*, time::*, error::*};

main_app_closure! {
  LogLevel::Warn,
  WindowBuilder::new(),
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, &AppEvent) {

  let window = ctx.window_clone();

  let (gx, mut target) = wgx_ctx::init(window.clone(), (features!(), limits!(), 4, None)).await;

  log::warn!("{:?}", gx.adapter.get_info());

  let shader = gx.load_wgsl(wgsl_modules::inline!("$shader" <= {
    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
      let x = f32(i32(index) - 1);
      let y = f32(i32(index & 1u) * 2 - 1);
      return vec4f(x, y, 0.0, 1.0);
    }

    @fragment
    fn fs_main() -> @location(0) vec4f {
      return vec4f(1.0, 0.0, 0.0, 1.0);
    }
  }));

  let pipeline = target.render_pipeline(&gx,
    None, &[],
    (&shader, "vs_main", Primitive::default()),
    (&shader, "fs_main", None),
  );


  move |_ctx: &mut AppCtx, event: &AppEvent| match event {

    AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
      target.update(&gx, [size.width, size.height]);
    },

    AppEvent::WindowEvent(WindowEvent::KeyboardInput { event: KeyEvent {
      state: ElementState::Pressed, physical_key: PhysicalKey::Code(KeyCode::KeyR), ..
    }, ..}) => {
      window.request_redraw();
    },

    AppEvent::WindowEvent(WindowEvent::RedrawRequested) => {

      let then = Instant::now();

      target.with_frame(None, |frame| gx.with_encoder(|encoder| {

        encoder.with_render_pass(frame.attachments(Some(Color::BLACK), None, None), |rpass| {
          rpass.set_pipeline(&pipeline);
          rpass.draw(0..3, 0..1);
        });

      })).unwrap_or_else(log_warn);

      log_warn_debug(then.elapsed());
    }

    _ => {},

  }

}