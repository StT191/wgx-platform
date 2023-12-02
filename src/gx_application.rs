
use crate::winit::{window::{Window, WindowBuilder}, event::*};
use crate::*;
use wgx::{Wgx, SurfaceTarget, SurfaceFrame, Limits, Features, wgpu::CommandEncoder};


#[derive(Debug)]
pub struct GxApp<App: GxApplication> {
    gx: Wgx, target: SurfaceTarget, app: App,
}

#[derive(Debug)]
pub struct Ctx<'a> {
    pub window: &'a Window, pub gx: &'a Wgx, pub target: &'a mut SurfaceTarget
}

#[derive(Debug)]
pub struct RenderCtx<'a> { pub window: &'a Window, pub gx: &'a Wgx } // without target



pub trait GxApplication: Sized + 'static {

    // from platform Application

    fn log_level() -> LogLevel { LogLevel::Warn }

    fn wake_lock(&self) -> bool { false }

    fn frame_duration(&self) -> Duration { STD_FRAME_DURATION }

    fn animate(&self) -> bool { false }

    fn with_window_builder(builder: WindowBuilder) -> WindowBuilder { builder }

    // wgx

    fn limits() -> Limits {
        #[cfg(not(target_family = "wasm"))] let limits = Limits::default();
        #[cfg(target_family = "wasm")] let limits = Limits::downlevel_webgl2_defaults();
        limits
    }

    fn features() -> Features { Features::empty() }

    fn msaa() -> u32 { 1 }
    fn depth_testing() -> bool { false }


    // event handling

    fn init(ctx: Ctx) -> Self;

    fn event(&mut self, _ctx: Ctx, _event: &WindowEvent) {}

    fn before_frame(&mut self, _ctx: Ctx) {}

    fn draw_frame(&mut self, ctx: RenderCtx, encoder: &mut CommandEncoder, frame: &SurfaceFrame);

    fn after_frame(&mut self, _ctx: Ctx) {}
}



#[async_trait(?Send)]
impl <App: GxApplication> Application for GxApp<App> {

    // pass on
    fn log_level() -> LogLevel { App::log_level() }

    fn wake_lock(&self) -> bool { self.app.wake_lock() }

    fn frame_duration(&self) -> Duration { self.app.frame_duration() }

    fn animate(&self) -> bool { self.app.animate() }

    fn with_window_builder(builder: WindowBuilder) -> WindowBuilder {
        App::with_window_builder(builder)
    }


    async fn init(window: &Window) -> Self {

        // wgx instance and surface
        let (gx, surface) = unsafe { Wgx::new(Some(window), App::features(), App::limits()) }.await.unwrap();

        let size = window.inner_size();

        let mut target = SurfaceTarget::new(&gx,
            surface.unwrap(), (size.width, size.height),
            App::msaa(), App::depth_testing(),
        ).unwrap();


        let app = App::init(Ctx { window, gx: &gx, target: &mut target});

        // instantiate app
        Self { gx, target, app }
    }


    fn draw_frame(&mut self, window: &Window) {

        self.app.before_frame(Ctx { window, gx: &self.gx, target: &mut self.target });

        self.target.with_encoder_frame(&self.gx, |encoder, frame| {
            self.app.draw_frame(RenderCtx { window, gx: &self.gx }, encoder, frame);
        }).unwrap_or_else(error::inspect);

        self.app.after_frame(Ctx { window, gx: &self.gx, target: &mut self.target });
    }


    fn event(&mut self, window: &Window, event: &WindowEvent) {

        self.app.event(Ctx { window, gx: &self.gx, target: &mut self.target }, event);

        // resize handler
        match event {

          WindowEvent::Resized(size)  => {
            self.target.update(&self.gx, (size.width as u32, size.height as u32));
          },

          _ => {}
        }
    }

}