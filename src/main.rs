use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::WindowResized;

pub struct Plugins;

impl PluginGroup for Plugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(bevy::log::LogPlugin::default())
            .add(bevy::core::CorePlugin)
            .add(bevy::transform::TransformPlugin)
            .add(bevy::app::ScheduleRunnerPlugin)
            .add(bevy::diagnostic::DiagnosticsPlugin::default())
            .add(bevy::input::InputPlugin::default())
            .add(bevy::window::WindowPlugin::default())
            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::scene::ScenePlugin::default())
            .add(bevy::winit::WinitPlugin::default())
            .add(bevy::gilrs::GilrsPlugin::default());
    }
}

struct RenderSystemOutput {
    dropped: bool,
}

fn graphics_setup_system(mut commands: Commands, windows: Res<bevy::window::Windows>) {
    info!("Initializing WGPU.");

    let instance = wgpu::Instance::new(wgpu::Backends::all());

    info!("Initialize WGPU surface...");

    let (main_window_surface, size) = {
        let primary_window = windows.get_primary().unwrap();

        let surface = unsafe {
            let handle = primary_window.raw_window_handle().get_handle();
            instance.create_surface(&handle)
        };

        (
            surface,
            (
                primary_window.physical_width(),
                primary_window.physical_height(),
            ),
        )
    };

    info!("Initialize WGPU adapter...");

    let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&main_window_surface),
        force_fallback_adapter: false,
    });

    // WGPU can't get an adapter for us right away....
    let adapter = pollster::block_on(adapter_future).unwrap();

    info!("Initialize WGPU device...");

    let request_future = adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None, // Trace path
    );

    let (device, queue) = pollster::block_on(request_future).unwrap();

    info!("First time configure WGPU surface...");

    let preferred_format = main_window_surface.get_preferred_format(&adapter).unwrap();

    let main_window_surface_configuration = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: preferred_format,
        width: size.0,
        height: size.1,
        present_mode: wgpu::PresentMode::Fifo,
    };

    main_window_surface.configure(&device, &main_window_surface_configuration);

    info!("Done initializing renderer.");

    // Insert base resources
    commands.insert_resource(instance);
    commands.insert_resource(device);
    commands.insert_resource(queue);
    commands.insert_resource(main_window_surface);
    commands.insert_resource(main_window_surface_configuration);
    commands.insert_resource(RenderSystemOutput{ dropped: true })
}

fn handle_window_resize_system(
    mut resize_event: EventReader<WindowResized>,
    device: Res<wgpu::Device>,
    surface: Res<wgpu::Surface>,
    mut surface_config: ResMut<wgpu::SurfaceConfiguration>,
    render_system_output: Res<RenderSystemOutput>,
) {
    let mut surface_needs_reconfigure = render_system_output.dropped;

    for event in resize_event.iter() {
        if event.id.is_primary() {
            if event.width == 0. || event.height == 0. {
                // The game has been minimized. Ignore this resize.
                return;
            }

            // Surface
            surface_config.width = event.width as u32;
            surface_config.height = event.height as u32;

            surface_needs_reconfigure = true;
        }
    }

    if surface_needs_reconfigure {
        info!(
            "Window resized to {}x{}, reconfiguring WGPU surface and render texture",
            surface_config.width, surface_config.height
        );

        surface.configure(&device, surface_config.as_mut());
    }
}

fn graphics_render_system(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    surface: Res<wgpu::Surface>,
    mut render_system_output: ResMut<RenderSystemOutput>,
) {
    let surface_texture = surface.get_current_texture();

    match surface_texture {
        Err(error) => {
            error!(
                "Frame dropped due to error getting surface texture: {}",
                error
            );
            render_system_output.dropped = true;
            return;
        }
        _ => {
            info!("Frame successfully rendered");
            render_system_output.dropped = false;
        }
    }

    let surface_texture = surface_texture.unwrap();
    let draw_to = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Perform draw
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &draw_to,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        // We DO NOT want to see this red! If it's visible, the render quad is NOT covering the screen!
                        r: 1.,
                        g: 0.,
                        b: 0.,
                        a: 1.,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
    }

    // submit will accept anything that implements IntoIter
    queue.submit(std::iter::once(encoder.finish()));

    surface_texture.present();
}

fn main() {
    App::new()
        .add_plugins(Plugins)
        .add_startup_system(graphics_setup_system)
        .add_system(handle_window_resize_system)
        .add_system_to_stage(CoreStage::PostUpdate, graphics_render_system)
        .run();
}
