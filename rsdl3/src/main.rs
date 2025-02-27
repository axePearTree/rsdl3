use rsdl3::{events::Event, init::Sdl, pixels::Color, video::WindowFlags, Error};

fn main() -> Result<(), Error> {
    let mut sdl = unsafe { Sdl::init() }?;
    let video = sdl.video()?;
    let events = sdl.events()?;
    let mut event_pump = events.event_pump()?;

    let mut renderer = video
        .create_window("SDL on Linux", 860, 640, WindowFlags::HIDDEN)?
        .create_renderer(None)?;

    renderer.set_render_draw_color(Color::new(255, 255, 0, 255))?;
    renderer.clear()?;
    renderer.present()?;
    renderer.as_window_mut().unwrap().show()?;

    'app: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit => break 'app,
                _ => {}
            }
        }
        renderer.clear()?;
        renderer.present()?;
    }

    Ok(())
}
