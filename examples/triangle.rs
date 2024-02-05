
use platform::winit::{window::WindowBuilder, event::WindowEvent};
use wgx::{*, /*cgmath::**/};
use platform::{*, frame_ctx::*, error::inspect};

async fn run() {

  let event_loop = event_loop();
  let window = window(WindowBuilder::new(), &event_loop).await;

  let ctx = GxCtx::new(window.clone(), features!(), limits!(), 4, false).await;
  let GxCtx {gx, target} = &ctx;

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


  let pipeline = target.render_pipeline(gx,
    None, &[],
    (&shader, "vs_main", Primitive::default()),
    (&shader, "fs_main", None),
  );


  event_loop.run(FrameCtx::new().run(window, ctx.run(move |_frame_ctx, GxCtx {gx, target}, event| {

    match event {

      WindowEvent::RedrawRequested => {
        target.with_frame(None, |frame| gx.with_encoder(|encoder| {

          encoder.with_render_pass(frame.attachments(Some(Color::BLACK), None), |rpass| {
            rpass.set_pipeline(&pipeline);
            rpass.draw(0..3, 0..1);
          });

        })).unwrap_or_else(inspect);
      }

      _ => {},
    }

  }))).unwrap();

}

fn main() {
  init(LogLevel::Warn);
  spawn_local(run());
}