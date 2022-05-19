#![warn(clippy::pedantic)]
#![deny(rust_2018_idioms)]

use args::Args;
use color_eyre::Result;
use renderer::{RenderMode, Renderer};
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

pub mod args;
mod data;
mod output;
pub mod renderer;

pub async fn preview() -> Result<()> {
    let mut input = WinitInputHelper::new();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut renderer = Renderer::new(RenderMode::Preview(&window)).await?;

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_released(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size)
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                renderer.update();
                // TODO: this is cringe
                let res = pollster::block_on(renderer.render());

                if let Err(e) = res {
                    if let Some(e) = e.downcast_ref::<wgpu::SurfaceError>() {
                        match e {
                            // Reconfigure the surface if lost
                            // XXX Fix this
                            // wgpu::SurfaceError::Lost => renderer.resize(renderer.size),
                            // The system is out of memory, we should probably quit
                            wgpu::SurfaceError::OutOfMemory => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                    eprintln!("{e:?}");
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            _ => {}
        }
    });
}

pub async fn output(args: Args) -> Result<()> {
    let mut renderer = Renderer::new(RenderMode::Output { args }).await?;

    for _ in 0..1200 {
        renderer.render().await?;
    }

    renderer.finish()?;
    Ok(())
}
