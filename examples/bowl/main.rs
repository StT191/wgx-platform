
use instant::Instant;
use platform::winit::{dpi::PhysicalSize, window::Window, event_loop::{ControlFlow}, event::{*}};
use platform::{*, Event};
use wgx::{*, cgmath::*};

const LOG_LEVEL: LogLevel = LogLevel::Warn;


async fn run(window: &'static Window, event_loop: EventLoop) {

  const DEPTH_TESTING:bool = true;
  const MSAA:u32 = 4;
  const ALPHA_BLENDING:Option<Blend> = None;

  window.set_title("WgFx");

  let PhysicalSize {width, height} = window.inner_size();

  #[cfg(not(target_family = "wasm"))] let (limits, features) = (Limits::default(), Features::MULTI_DRAW_INDIRECT);
  #[cfg(target_family = "wasm")] let (limits, features) = (Limits::downlevel_webgl2_defaults(), Features::empty());

  let (gx, surface) = unsafe {Wgx::new(Some(window), features, limits)}.await.unwrap();
  let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


  // pipeline
  let shader = gx.load_wgsl(include_wgsl_module!("./v3d_inst_text_diff.wgsl"));


  // triangle pipeline
  let pipeline = target.render_pipeline(&gx,
    None, &[
      vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3),
      vertex_desc!(Instance, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4)
    ],
    (&shader, "vs_main", Primitive::default()),
    (&shader, "fs_main", ALPHA_BLENDING),
  );

  // colors
  let color_texture = TextureLot::new_2d_with_data(&gx, (1, 1), 1, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING, [255u8, 0, 0, 255]);

  let sampler = gx.default_sampler();


  let clip_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);
  let light_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);

  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, &clip_buffer),
    bind!(1, Buffer, &light_buffer),
    bind!(2, TextureView, &color_texture.view),
    bind!(3, Sampler, &sampler),
  ]);


  // vertexes
  let steps = 12usize;
  let smooth = false;


  let step_a = steps as f32 / std::f32::consts::FRAC_PI_2; // step angle

  let mut mesh:Vec<[[f32;3];3]> = Vec::with_capacity(2 * 2 * 3 * steps * steps);

  let t_c = [1.0, 1.0, 0.0];


  for k in 0..steps {

    let fi_a0 = k as f32 / step_a;
    let fi_a1 = (k as f32 + 1.0) / step_a;

    let cos_a0 = f32::cos(fi_a0);
    let cos_a1 = f32::cos(fi_a1);

    let sin_a0 = f32::sin(fi_a0);
    let sin_a1 = f32::sin(fi_a1);

    for j in 0..steps {

      let fi_b0 = j as f32 / step_a;
      let fi_b1 = (j as f32 + 1.0) / step_a;

      let cos_b0 = f32::cos(fi_b0);
      let cos_b1 = f32::cos(fi_b1);

      let sin_b0 = f32::sin(fi_b0);
      let sin_b1 = f32::sin(fi_b1);

      let a = [cos_a0*sin_b0, sin_a0, cos_a0*cos_b0];
      let b = [cos_a1*sin_b0, sin_a1, cos_a1*cos_b0];

      let c = [cos_a1*sin_b1, sin_a1, cos_a1*cos_b1];
      let d = [cos_a0*sin_b1, sin_a0, cos_a0*cos_b1];

      if smooth {
        mesh.push([a, t_c, a]);
        mesh.push([d, t_c, d]);
        mesh.push([c, t_c, c]);

        mesh.push([a, t_c, a]);
        mesh.push([c, t_c, c]);
        mesh.push([b, t_c, b]);
      }
      else {
        let n = normal_from_triangle(a, d, c).into();

        mesh.push([a, t_c, n]);
        mesh.push([d, t_c, n]);
        mesh.push([c, t_c, n]);

        mesh.push([a, t_c, n]);
        mesh.push([c, t_c, n]);
        mesh.push([b, t_c, n]);
      }
    }
  }

  let instance_data = [
    Matrix4::<f32>::from_nonuniform_scale( 1.0, 1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0, 1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale( 1.0,-1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0,-1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale( 1.0, 1.0,-1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0, 1.0,-1.0),
    Matrix4::<f32>::from_nonuniform_scale( 1.0,-1.0,-1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0,-1.0,-1.0),
  ];

  let indirect_buffer = gx.buffer_from_data(BufUse::INDIRECT, [
    DrawIndirect::try_from_ranges(0..mesh.len(), 0..instance_data.len()).unwrap(),
  ]);

  let vertex_buffer = gx.buffer_from_data(BufUse::VERTEX, mesh);
  let instance_buffer = gx.buffer_from_data(BufUse::VERTEX, instance_data);


  // matrix
  const DA:f32 = 5.0;
  const DS:f32 = 50.0;

  let fov_deg = 45.0;

  let (width, height) = (width as f32, height as f32);

  // let mut scale = 1.0;
  // let (mut w, mut h) = (0.4, 0.4);

  let fov = FovProjection::window(fov_deg, width, height);
  let mut projection = fov.projection * fov.translation;

  // let camera_correction = fov.translation;

  let obj_mat =
    // Matrix4::identity()
    Matrix4::from_scale(0.25 * height)
    // Matrix4::from_translation((0.0, -0.7 * height, 0.0).into())
  ;

  // let light_matrix = Matrix4::<f32>::from_angle_x(Deg(-45.0)) * Matrix4::from_angle_y(Deg(45.0));
  let light_matrix = Matrix4::<f32>::identity();

  // let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

  let mut rot_matrix = Matrix4::identity();
  let mut world_matrix = Matrix4::identity();

  let clip_matrix = projection * obj_mat;

  // gx.write_buffer(&world_buffer, 0, AsRef::<[f32; 16]>::as_ref(&world_matrix));
  gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
  gx.write_buffer(&light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&(light_matrix)));
  // gx.write_buffer(&viewport_buffer, 0, &[width, height]);


  // event loop
  event_loop.run(move |event, _, control_flow| {

    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
        *control_flow = ControlFlow::Exit;
      },

      Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
        target.update(&gx, (size.width, size.height));

        let width = size.width as f32;
        let height = size.height as f32;

        let fov = FovProjection::window(fov_deg, width, height);
        projection = fov.projection * fov.translation;

        // projection
        let clip_matrix = projection * rot_matrix * world_matrix * obj_mat;

        gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
        // gx.write_buffer(&mut viewport_buffer, 0, &[width, height]);

        window.request_redraw();
      },

      Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
      }, ..}, ..} => {
        let mut redraw = true;
        match keycode {

          VirtualKeyCode::I => { apply!(rot_matrix, Matrix4::from_angle_x(Deg( DA))); },
          VirtualKeyCode::K => { apply!(rot_matrix, Matrix4::from_angle_x(Deg(-DA))); },
          VirtualKeyCode::J => { apply!(rot_matrix, Matrix4::from_angle_y(Deg( DA))); },
          VirtualKeyCode::L => { apply!(rot_matrix, Matrix4::from_angle_y(Deg(-DA))); },
          VirtualKeyCode::U => { apply!(rot_matrix, Matrix4::from_angle_z(Deg( DA))); },
          VirtualKeyCode::O => { apply!(rot_matrix, Matrix4::from_angle_z(Deg(-DA))); },

          VirtualKeyCode::A => { apply!(world_matrix, Matrix4::from_translation((-DS, 0.0, 0.0).into())); },
          VirtualKeyCode::D => { apply!(world_matrix, Matrix4::from_translation(( DS, 0.0, 0.0).into())); },
          VirtualKeyCode::W => { apply!(world_matrix, Matrix4::from_translation((0.0, 0.0,  DS).into())); },
          VirtualKeyCode::S => { apply!(world_matrix, Matrix4::from_translation((0.0, 0.0, -DS).into())); },
          VirtualKeyCode::Q => { apply!(world_matrix, Matrix4::from_translation((0.0, -DS, 0.0).into())); },
          VirtualKeyCode::E => { apply!(world_matrix, Matrix4::from_translation((0.0,  DS, 0.0).into())); },

          VirtualKeyCode::Y => { apply!(world_matrix, Matrix4::from_scale(0.9)); },
          VirtualKeyCode::X => { apply!(world_matrix, Matrix4::from_scale(1.1)); },

          VirtualKeyCode::R => {
            rot_matrix = Matrix4::identity();
            world_matrix = Matrix4::identity();
          },

          _ => { redraw = false; }
        } {
          if redraw {

            let clip_matrix = projection * rot_matrix * world_matrix * obj_mat;
            // let light_matrix = rot_matrix * light_matrix;
            let light_matrix = light_matrix;

            gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
            gx.write_buffer(&light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&light_matrix));

            window.request_redraw();
          }
        }
      },

      Event::RedrawRequested(_) => {

        let then = Instant::now();

        target.with_encoder_frame(&gx, |encoder, frame| {
          encoder.with_render_pass(frame.attachments(Some(Color::BLACK), Some(1.0)), |mut rpass| {
            rpass.set_pipeline(&pipeline);
            rpass.set_bind_group(0, &binding, &[]);
            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, instance_buffer.slice(..));

            rpass.draw_indirect(&indirect_buffer, 0);

            // #[cfg(not(target_family = "wasm"))]
            // rpass.multi_draw_indirect(&group.indirect.buffer, 0, group.indirect.len() as u32);

            // #[cfg(target_family = "wasm")]
            // for indirect in &group.indirect.data {
            //   rpass.draw(indirect.vertex_range().unwrap(), indirect.instance_range().unwrap());
            // }

          });
        }).expect("frame error");

        log::warn!("{:?}", then.elapsed());
      },

      _ => {}
    }
  });
}

fn main() {
  platform::main(run, LOG_LEVEL);
}