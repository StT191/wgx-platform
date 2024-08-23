
use std::sync::Arc;
use winit::window::Window;
use wgx::{Wgx, SurfaceTarget, Limits, Features};

pub type WgxCtx = (Wgx, SurfaceTarget);
pub type InitData = (Features, Limits, u32, bool);

pub async fn init(window: Arc<Window>, (features, limits, msaa, depth_testing): InitData) -> WgxCtx {
  let size = window.inner_size();
  let (gx, surface) = Wgx::new(Some(window), features, limits).await.unwrap();
  let target = SurfaceTarget::new(&gx, surface.unwrap(), [size.width, size.height], msaa, depth_testing).unwrap();
  (gx, target)
}