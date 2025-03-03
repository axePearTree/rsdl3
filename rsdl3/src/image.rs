use crate::init::VideoSubsystem;
use crate::surface::Surface;
use crate::sys;
use crate::Error;
use alloc::ffi::CString;

impl VideoSubsystem {
    pub fn load_image(&self, path: &str) -> Result<Surface, Error> {
        let path = CString::new(path)?;
        unsafe {
            let surface = sys::image::IMG_Load(path.as_ptr());
            if surface.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Surface::from_mut_ptr(self, surface))
        }
    }
}
