use crate::init::VideoSubsystem;
use crate::surface::Surface;
use crate::sys;
use crate::Error;
use alloc::ffi::CString;

impl VideoSubsystem {
    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    /// Loads an image from the specified file path into a [`Surface`].
    /// This method is equivalent to [`Surface::from_image`].
    pub fn load_image(&self, path: &str) -> Result<Surface, Error> {
        Surface::from_image(self, path)
    }
}

impl Surface {
    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    /// Creates a new `Surface` by loading an image from the specified file path.
    pub fn from_image(video: &VideoSubsystem, path: &str) -> Result<Self, Error> {
        let path = CString::new(path)?;
        unsafe {
            let surface = sys::image::IMG_Load(path.as_ptr());
            if surface.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Self::from_mut_ptr(video, surface))
        }
    }
}
