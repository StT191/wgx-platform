
use std::sync::Arc;
use crate::winit::{window::Window, event::WindowEvent};
use crate::frame_ctx::FrameCtx;

use wgx::{Wgx, SurfaceTarget, Limits, Features};


#[derive(Debug)]
pub struct GxCtx { pub gx: Wgx, pub target: SurfaceTarget }


impl GxCtx {

  pub async fn new(window: Arc<Window>, features: Features, limits: Limits, msaa: u32, depth_testing: bool) -> Self {

    let size = window.inner_size();

    let (gx, surface) = Wgx::new(Some(window), features, limits).await.unwrap();

    let target = SurfaceTarget::new(&gx, surface.unwrap(), (size.width, size.height), msaa, depth_testing).unwrap();

    Self { gx, target }
  }

  pub fn run(mut self,
    mut event_handler: impl FnMut(&mut FrameCtx, &mut Self, &WindowEvent) + 'static,
  )
    -> impl FnMut(&mut FrameCtx, &WindowEvent)
  {
    move |frame_ctx, event| {

      // resize handler
      if let WindowEvent::Resized(size) = &event {
        self.target.update(&self.gx, (size.width as u32, size.height as u32));
      }

      event_handler(frame_ctx, &mut self, event);

    }
  }
}
