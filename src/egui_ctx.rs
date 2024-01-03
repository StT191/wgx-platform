
use crate::winit::{window::Window, event::*};
use crate::{Duration};
use wgx::{*, wgpu::{CommandEncoder, RenderPass}};

use egui::*;
use egui_winit::{State, update_viewport_info, process_viewport_commands};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};


// ScreenDescriptor helper
pub fn screen_dsc_from_window(window: &Window) -> ScreenDescriptor {
    let size = window.inner_size();
    ScreenDescriptor { size_in_pixels: [size.width, size.height], pixels_per_point: window.scale_factor() as f32 }
}

pub fn clone_screen_dsc(other: &ScreenDescriptor) -> ScreenDescriptor {
    ScreenDescriptor { size_in_pixels: other.size_in_pixels, pixels_per_point: other.pixels_per_point }
}

pub fn clip_rect_from_screen_dsc(screen_dsc: &ScreenDescriptor) -> Rect {
    let sf = screen_dsc.pixels_per_point;
    let [w, h] = screen_dsc.size_in_pixels;
    [[0.0, 0.0].into(), [w as f32/sf, h as f32/sf].into()].into()
}


// items

pub fn renderer(gx: &impl WgxDevice, target: &impl RenderTarget) -> Renderer {
    Renderer::new(gx.device(), target.view_format(), target.depth_testing(), target.msaa())
}


pub struct EguiCtx {
    pub context: Context,
    pub state: State,
    pub screen_dsc: ScreenDescriptor,
}

pub struct FrameOutput {
    pub clipped_primitives: Vec<ClippedPrimitive>,
    pub textures_delta: TexturesDelta,
    pub screen_dsc: ScreenDescriptor,
    pub commands: Vec<ViewportCommand>,
    pub repaint_delay: Duration,
}


impl EguiCtx {

    pub fn new(window: &Window) -> Self {
        let context = Context::default();
        // install image loaders, need to be added via features in egui_extras
        egui_extras::install_image_loaders(&context);
        Self {
            state: State::new(context.clone(), ViewportId::ROOT, window, None, None),
            screen_dsc: screen_dsc_from_window(window),
            context,
        }
    }

    pub fn event(&mut self, window: &Window, event: &WindowEvent) -> (bool, bool) {

        if matches!(event, WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged {..}) {
            self.screen_dsc = screen_dsc_from_window(window);
        }

        if *event != WindowEvent::RedrawRequested {
            let res = self.state.on_window_event(window, event);
            (res.repaint, res.consumed)
        } else {
            (false, false)
        }
    }

    pub fn run(&mut self, window: &Window, ui_fn: impl FnOnce(&Context)) -> FrameOutput {

        let mut input = self.state.take_egui_input(window);

        let viewport_id = input.viewport_id;

        // update viewport info for context input
        let is_focused = {
            let viewport_info = input.viewports.get_mut(&viewport_id).unwrap();
            update_viewport_info(viewport_info, &self.context, &window);
            viewport_info.focused.unwrap_or(false)
        };

        let mut output = self.context.run(input, ui_fn);

        self.state.handle_platform_output(window, output.platform_output);

        let viewport_output = output.viewport_output.remove(&viewport_id).unwrap();

        if viewport_output.commands.len() != 0 {
            process_viewport_commands(
                &self.context,
                &mut ViewportInfo::default(),
                viewport_output.commands.iter().cloned(),
                window,
                is_focused,
                &mut false,
            );
        }

        FrameOutput {
            clipped_primitives: self.context.tessellate(output.shapes, output.pixels_per_point),
            textures_delta: output.textures_delta,
            screen_dsc: clone_screen_dsc(&self.screen_dsc),
            commands: viewport_output.commands,
            repaint_delay: viewport_output.repaint_delay,
        }
    }
}


impl FrameOutput {

    pub fn prepare(&self, renderer: &mut Renderer, gx: &impl WgxDeviceQueue, encoder: &mut CommandEncoder) {

        for (id, image_delta) in &self.textures_delta.set {
            renderer.update_texture(gx.device(), gx.queue(), *id, &image_delta);
        }

        let commands = renderer.update_buffers(gx.device(), gx.queue(), encoder, &self.clipped_primitives, &self.screen_dsc);

        if commands.len() != 0 {
            gx.queue().submit(commands);
        }

        for id in &self.textures_delta.free {
            renderer.free_texture(&id);
        }
    }

    pub fn render<'a>(&'a self, renderer: &'a mut Renderer, rpass: &mut RenderPass<'a>) {
        renderer.render(rpass, &self.clipped_primitives, &self.screen_dsc);
    }

}



// epaint drawing context

use epaint::{
    Fonts, text::{FontDefinitions, /*FontId*/}, TextureId, Shape, ClippedShape, ClippedPrimitive,
    TessellationOptions, tessellate_shapes, ImageData, TextureManager, TextureAtlas, Rect,
};


pub struct EpaintCtx {
    pub texture_manager: TextureManager,
    pub fonts: Fonts,
    pub screen_dsc: ScreenDescriptor,
}




pub fn clip_shapes(shapes: impl IntoIterator<Item=Shape>, clip_rect: Rect) -> impl Iterator<Item=ClippedShape> {
    shapes.into_iter().map(move |shape| ClippedShape {shape, clip_rect})
}


impl EpaintCtx {

    pub fn new(screen_dsc: ScreenDescriptor, max_texture_side: usize, fonts: Option<Fonts>) -> Self {

        let mut texture_manager = TextureManager::default();
        let fonts = fonts.unwrap_or(Fonts::new(screen_dsc.pixels_per_point, max_texture_side, FontDefinitions::default()));

        assert_eq!(
            texture_manager.alloc("font_texture".to_string(), ImageData::Font(fonts.image()), TextureAtlas::texture_options()),
            TextureId::default(),
        );

        Self { texture_manager, fonts, screen_dsc }
    }

    pub fn begin_frame(&mut self, screen_dsc: Option<ScreenDescriptor>, max_texture_side: Option<usize>) {
        if let Some(screen_dsc) = screen_dsc {
            self.screen_dsc = screen_dsc;
        }
        let max_texture_side = max_texture_side.unwrap_or(self.fonts.max_texture_side());
        self.fonts.begin_frame(self.screen_dsc.pixels_per_point, max_texture_side);
    }

    pub fn clip_shapes(&self, shapes: impl IntoIterator<Item=Shape>, clip_rect: Option<Rect>) -> impl Iterator<Item=ClippedShape> {
        clip_shapes(shapes, clip_rect.unwrap_or(clip_rect_from_screen_dsc(&self.screen_dsc)))
    }

    pub fn tessellate_shapes(&self, shapes: Vec<ClippedShape>) -> Vec<ClippedPrimitive> {
        tessellate_shapes(
            self.screen_dsc.pixels_per_point, TessellationOptions::default(), self.fonts.image().size,
            self.fonts.texture_atlas().lock().prepared_discs(), shapes,
        )
    }

    pub fn prepare(&mut self,
        renderer: &mut Renderer, gx: &impl WgxDeviceQueue, encoder: &mut CommandEncoder,
        clipped_primitives: &[ClippedPrimitive],
    ) {
        let textures_delta = self.texture_manager.take_delta();

        for (id, image_delta) in textures_delta.set {
            renderer.update_texture(gx.device(), gx.queue(), id, &image_delta);
        }

        if clipped_primitives.len() != 0 {
            let commands = renderer.update_buffers(gx.device(), gx.queue(), encoder, clipped_primitives, &self.screen_dsc);

            if commands.len() != 0 {
                gx.queue().submit(commands);
            }
        }

        for id in textures_delta.free {
            renderer.free_texture(&id);
        }
    }

    pub fn render<'a>(&'a self,
        renderer: &'a mut Renderer, rpass: &mut RenderPass<'a>,
        clipped_primitives: &'a [ClippedPrimitive],
    ) {
        renderer.render(rpass, clipped_primitives, &self.screen_dsc);
    }
}