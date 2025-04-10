use crate::pixels::Colorspace;
use crate::pixels::PixelFormat;
use crate::surface::SurfaceRef;
use crate::sys;
use crate::CameraSubsystem;
use crate::Error;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::mem::MaybeUninit;

impl CameraSubsystem {
    /// Returns a list of currently connected camera devices.
    pub fn cameras(&self) -> Result<Vec<CameraId>, Error> {
        unsafe {
            let mut count = 0;
            let ptr = sys::SDL_GetCameras(&raw mut count);
            if ptr.is_null() {
                return Err(Error::new());
            }
            let count = usize::try_from(count)?;
            let vec = core::slice::from_raw_parts(ptr, count).to_vec();
            sys::SDL_free(ptr as _);
            Ok(vec)
        }
    }

    /// Equivalent to [`Camera::open`].
    pub fn open_camera(&self, id: CameraId, spec: Option<CameraSpec>) -> Result<Camera, Error> {
        Camera::open(self, id, spec)
    }

    /// Returns the human-readable device name for a camera.
    pub fn camera_name(&self, id: CameraId) -> Result<String, Error> {
        let ptr = unsafe { sys::SDL_GetCameraName(id) };
        if ptr.is_null() {
            return Err(Error::new());
        }
        Ok(unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned())
    }

    /// Returns the position of the camera in relation to the system.
    ///
    /// Most platforms will report [`CameraPosition::Unknown`], but mobile devices, like phones, can
    /// often make a distinction between cameras on the front of the device (that points towards the
    /// user, for taking "selfies") and cameras on the back (for filming in the direction the user
    /// is facing).
    pub fn camera_position(&self, id: CameraId) -> CameraPosition {
        unsafe { CameraPosition::from_ll_unchecked(sys::SDL_GetCameraPosition(id)) }
    }

    /// Returns the list of native formats/sizes a camera supports.
    ///
    /// This returns a list of all formats and frame sizes that a specific camera can offer. This is
    /// useful if your app can accept a variety of image formats and sizes and so want to find the
    /// optimal spec that doesn't require conversion.
    ///
    /// This function isn't strictly required; if you call [`CameraSubsystem::open_camera`] with a
    /// `None` spec, SDL will choose a native format for you, and if you instead specify a desired
    /// format, it will transparently convert to the requested format on your behalf.
    ///
    /// Note that it's legal for a camera to supply an empty list. This is what will happen on
    /// Emscripten builds, since that platform won't tell _anything_ about available cameras until
    /// you've opened one, and won't even tell if there _is_ a camera until the user has given you
    /// permission to check through a scary warning popup.
    pub fn camera_supported_formats(&self, id: CameraId) -> Result<Vec<CameraSpec>, Error> {
        unsafe {
            let mut count = 0;
            let ptr = sys::SDL_GetCameraSupportedFormats(id, &raw mut count);
            if ptr.is_null() {
                return Err(Error::new());
            }
            let len = usize::try_from(count)?;
            let slice = core::slice::from_raw_parts(ptr, len);
            let mut vec = Vec::with_capacity(len);
            for &ptr in slice {
                vec.push(CameraSpec::from_ll(*ptr));
            }
            Ok(vec)
        }
    }

    /// Get the name of the current camera driver.
    ///
    /// The names of drivers are all simple, low-ASCII identifiers, like "v4l2", "coremedia" or "android".
    /// These never have Unicode characters, and are not meant to be proper names.
    pub fn current_camera_driver(&self) -> Result<String, Error> {
        let ptr = unsafe { sys::SDL_GetCurrentCameraDriver() };
        if ptr.is_null() {
            return Err(Error::new());
        }
        Ok(unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned())
    }
}

pub type CameraId = sys::SDL_CameraID;

/// The structure used to identify an opened SDL camera.
pub struct Camera {
    subsystem: CameraSubsystem,
    ptr: *mut sys::SDL_Camera,
}

impl Camera {
    /// Open a video recording device (a "camera").
    ///
    /// You can open the device with any reasonable spec, and if the hardware can't directly support it,
    /// it will convert data seamlessly to the requested format. This might incur overhead, including
    /// scaling of image data.
    ///
    /// If you would rather accept whatever format the device offers, you can pass a `None` spec here and
    /// it will choose one for you (and you can use [`crate::surface::Surface`] conversion/scaling
    /// functions directly if necessary).
    ///
    /// You can call [`Camera::format`] to get the actual data format if passing a `None` spec here. You
    /// can see the exact specs a device can support without conversion with [`Camera::supported_formats`].
    ///
    /// SDL will not attempt to emulate framerate; it will try to set the hardware to the rate closest to
    /// the requested speed, but it won't attempt to limit or duplicate frames artificially; call
    /// [`Camera::format`] to see the actual framerate of the opened the device, and check your timestamps
    /// if this is crucial to your app!
    ///
    /// Note that the camera is not usable until the user approves its use! On some platforms, the operating
    /// system will prompt the user to permit access to the camera, and they can choose Yes or No at that
    /// point. Until they do, the camera will not be usable. The app should either wait for an [`Event`]
    /// with payload [`EventPayload::Camera(CameraEvent::DeviceApproved)`] (or
    /// [`EventPayload::Camera(CameraEvent::DeviceDenied)`]) event, or poll [`Camera::permission_state`]
    /// occasionally until it returns [`CameraPermissionState::Approved`]. On platforms that don't require
    /// explicit user approval (and perhaps in places where the user previously permitted access), the
    /// approval event might come immediately, but it might come seconds, minutes, or hours later!
    ///
    /// [`Event`]: crate::events::Event
    /// [`EventPayload`]: crate::events::EventPayload
    /// [`CameraEvent`]: crate::events::CameraEvent
    pub fn open(
        subsystem: &CameraSubsystem,
        id: CameraId,
        spec: Option<CameraSpec>,
    ) -> Result<Self, Error> {
        // SAFETY: CameraSpec has repr(transparent) and can be treated like a sys::SDL_CameraSpec.
        let spec = spec
            .as_ref()
            .map(CameraSpec::raw)
            .unwrap_or(core::ptr::null());
        let ptr = unsafe { sys::SDL_OpenCamera(id, spec) };
        if ptr.is_null() {
            return Err(Error::new());
        }
        Ok(Self {
            subsystem: subsystem.clone(),
            ptr,
        })
    }

    /// Returns the instance ID of an opened camera.
    pub fn id(&self) -> Result<CameraId, Error> {
        let result = unsafe { sys::SDL_GetCameraID(self.ptr) };
        if result == 0 {
            return Err(Error::new());
        }
        Ok(result)
    }

    /// Returns the human-readable device name for a camera.
    pub fn name(&self) -> Result<String, Error> {
        self.subsystem.camera_name(self.id()?)
    }

    /// Returns the position of the camera in relation to the system.
    ///
    /// Most platforms will report [`CameraPosition::Unknown`], but mobile devices, like phones, can
    /// often make a distinction between cameras on the front of the device (that points towards the
    /// user, for taking "selfies") and cameras on the back (for filming in the direction the user
    /// is facing).
    pub fn position(&self) -> Result<CameraPosition, Error> {
        Ok(self.subsystem.camera_position(self.id()?))
    }

    /// Query if camera access has been approved by the user.
    ///
    /// Cameras will not function between when the device is opened by the app and when the user permits
    /// access to the hardware. On some platforms, this presents as a popup dialog where the user has to
    /// explicitly approve access; on others the approval might be implicit and not alert the user at
    /// all.
    ///
    /// This function can be used to check the status of that approval. It will return
    /// `None` if still waiting for user response, `Some(CameraPermissionState::Approved)` if the camera
    /// is approved for use, and `Some(CameraPermissionState::Denied)` if the user denied access.
    ///
    /// Instead of polling with this function, you can wait for an [`Event`] with payload
    /// [`EventPayload::Camera(CameraEvent::DeviceApproved)`] (or
    /// [`EventPayload::Camera(CameraEvent::DeviceDenied)`]) event in the standard SDL event loop, which
    /// is guaranteed to be sent once when permission to use the camera is decided.
    ///
    /// If a camera is declined, there's nothing to be done but drop the `Camera` to dispose of it.
    pub fn permission_state(&self) -> Option<CameraPermissionState> {
        let result = unsafe { sys::SDL_GetCameraPermissionState(self.ptr) };
        CameraPermissionState::from_ll(result)
    }

    /// Returns the spec that a camera is using when generating images.
    ///
    /// Note that this might not be the native format of the hardware, as SDL might be converting to
    /// this format behind the scenes.
    ///
    /// If the system is waiting for the user to approve access to the camera, as some platforms require,
    /// this will return false, but this isn't necessarily a fatal error; you should either wait for an
    /// [`Event`] with payload [`EventPayload::Camera(CameraEvent::DeviceApproved)`] (or
    /// [`EventPayload::Camera(CameraEvent::DeviceDenied)`]) event, or poll [`Camera::permission_state`]
    /// occasionally until it returns [`CameraPermissionState::Approved`].
    ///
    /// [`Event`]: crate::events::Event
    /// [`EventPayload`]: crate::events::EventPayload
    /// [`CameraEvent`]: crate::events::CameraEvent
    pub fn format(&self) -> Option<CameraSpec> {
        let mut spec: MaybeUninit<sys::SDL_CameraSpec> = MaybeUninit::uninit();
        let result = unsafe { sys::SDL_GetCameraFormat(self.ptr, spec.as_mut_ptr()) };
        if !result {
            return None;
        }
        Some(CameraSpec::from_ll(unsafe { spec.assume_init() }))
    }

    /// Returns the list of native formats/sizes a camera supports.
    ///
    /// This returns a list of all formats and frame sizes that a specific camera can offer. This is
    /// useful if your app can accept a variety of image formats and sizes and so want to find the
    /// optimal spec that doesn't require conversion.
    ///
    /// This function isn't strictly required; if you call [`CameraSubsystem::open_camera`] with a
    /// `None` spec, SDL will choose a native format for you, and if you instead specify a desired
    /// format, it will transparently convert to the requested format on your behalf.
    ///
    /// Note that it's legal for a camera to supply an empty list. This is what will happen on
    /// Emscripten builds, since that platform won't tell _anything_ about available cameras until
    /// you've opened one, and won't even tell if there _is_ a camera until the user has given you
    /// permission to check through a scary warning popup.
    pub fn supported_formats(&self) -> Result<Vec<CameraSpec>, Error> {
        self.subsystem.camera_supported_formats(self.id()?)
    }

    /// Acquire a frame.
    ///
    /// The frame is a memory pointer to the image data, whose size and format are given by the
    /// spec requested when opening the device.
    ///
    /// This is a non blocking API. If there is a frame available, a `CameraFrame` is returned,
    /// and the frame's timestamp will be filled with a non-zero value.
    ///
    /// An `Ok(None)` by itself signifies that a new frame is not yet available. Note that even if a
    /// camera device fails outright (a USB camera is unplugged while in use, etc), SDL will
    /// send an event separately to notify the app, but continue to provide blank frames at
    /// ongoing intervals until the `Camera` gets dropped, so real failure here is almost
    /// always an out of memory condition.
    ///
    /// If the system is waiting for the user to approve access to the camera, as some platforms
    /// require, this will return `Ok(None)` (no frames available); you should either wait for an
    /// [`EventPayload::Camera(CameraEvent::DeviceApproved)`] or
    /// [`EventPayload::Camera(CameraEvent::DeviceDenied)`] event, or poll [`Camera::permission_state`]
    /// occasionally until it returns [`CameraPermissionState::Approved`].
    ///
    /// [`EventPayload`]: crate::events::EventPayload
    /// [`CameraEvent`]: crate::events::CameraEvent
    pub fn acquire_frame(&mut self) -> Result<Option<CameraFrame>, Error> {
        let mut timestamp = 0;
        unsafe {
            let surface = sys::SDL_AcquireCameraFrame(self.ptr, &raw mut timestamp);
            if surface.is_null() {
                return Ok(None);
            }
            if timestamp == 0 {
                return Err(Error::new());
            }
            if surface.is_null() {
                return Ok(None);
            }
            Ok(Some(CameraFrame {
                camera: self,
                surface: SurfaceRef::from_mut_ptr(surface),
                timestamp,
            }))
        }
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            sys::SDL_CloseCamera(self.ptr);
        }
    }
}

/// A camera frame.
///
/// The surface containing the contents of this frame can be obtained by
/// accessing the `surface` field. The timestamp can be obtained by calling
/// [`CameraFrame::timestamp`].
pub struct CameraFrame<'a> {
    camera: &'a Camera,
    timestamp: u64,
    pub surface: &'a SurfaceRef,
}

impl CameraFrame<'_> {
    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl Drop for CameraFrame<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::SDL_ReleaseCameraFrame(self.camera.ptr, self.surface.raw());
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct CameraSpec(sys::SDL_CameraSpec);

impl CameraSpec {
    /// Creates a `CameraSpec`.
    pub fn new(
        format: PixelFormat,
        colorspace: Colorspace,
        width: u32,
        height: u32,
        framerate_numerator: u32,
        framerate_denominator: u32,
    ) -> Result<Self, Error> {
        let width = i32::try_from(width)?;
        let height = i32::try_from(height)?;
        let framerate_numerator = i32::try_from(framerate_numerator)?;
        let framerate_denominator = i32::try_from(framerate_denominator)?;
        if framerate_denominator == 0 {
            return Err(Error::register(c"Invalid zero framerate_denominator."));
        }
        Ok(Self(sys::SDL_CameraSpec {
            format: format.to_ll(),
            colorspace: colorspace.to_ll(),
            width,
            height,
            framerate_numerator,
            framerate_denominator,
        }))
    }

    #[inline]
    pub fn format(&self) -> PixelFormat {
        unsafe { PixelFormat::from_ll_unchecked(self.0.format) }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.0.width.max(0) as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.0.height.max(0) as u32
    }

    #[inline]
    pub fn framerate_numerator(&self) -> u32 {
        self.0.framerate_numerator as u32
    }

    #[inline]
    pub fn framerate_denominator(&self) -> u32 {
        self.0.framerate_denominator as u32
    }

    #[inline]
    pub fn colorspace(&self) -> Colorspace {
        Colorspace::from_ll(self.0.colorspace)
    }

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_CameraSpec {
        self.0
    }

    #[inline]
    pub fn raw(&self) -> *const sys::SDL_CameraSpec {
        self as *const Self as *const sys::SDL_CameraSpec
    }

    #[inline]
    fn from_ll(ll: sys::SDL_CameraSpec) -> Self {
        Self(ll)
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum CameraPermissionState {
    Denied = -1,
    Approved = 1,
}

impl CameraPermissionState {
    fn from_ll(ll: i32) -> Option<Self> {
        if ll == -1 {
            Some(Self::Denied)
        } else if ll == 1 {
            Some(Self::Approved)
        } else {
            None
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum CameraPosition {
    FrontFacing = sys::SDL_CameraPosition_SDL_CAMERA_POSITION_FRONT_FACING,
    BackFacing = sys::SDL_CameraPosition_SDL_CAMERA_POSITION_BACK_FACING,
    Unknown = sys::SDL_CameraPosition_SDL_CAMERA_POSITION_UNKNOWN,
}

impl CameraPosition {
    /// SAFETY: only call this if the value comes from SDL (guaranteed to be a variant).
    unsafe fn from_ll_unchecked(ll: sys::SDL_CameraPosition) -> Self {
        unsafe { core::mem::transmute(ll) }
    }
}
