use rsdl3::init::*;
use rsdl3::video::WindowFlags;
use rsdl3::Error;

fn main() -> Result<(), Error> {
    let sdl = unsafe { Sdl::init() }?;
    let video = sdl.video()?;

    let flags = WindowFlags::HIDDEN | WindowFlags::RESIZABLE;
    let window = video.create_window("SDL test", 800, 600, flags)?;

    window.show()?;

    /*
    let mut renderer = window.create_renderer(None)?;

    let mut render_target =
        renderer.create_texture(PixelFormat::RGBA8888, TextureAccess::TARGET, 800, 600)?;

    let texture = {
        let mut surface = video.create_surface(800, 600, PixelFormat::RGBA8888)?;
        surface.clear(Color::new(255, 255, 0, 255).into())?;
        renderer.create_texture_from_surface(&mut surface)?
    };

    renderer.set_render_target(Some(render_target))?;

    renderer.render_texture(&texture, None, None)?;

    render_target = renderer.set_render_target(None)?.unwrap();

    renderer.render_texture(&render_target, None, None)?;
    renderer.present()?;

    renderer.as_window_ref().show()?;
    */

    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}
