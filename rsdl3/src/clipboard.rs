use crate::sys;
use crate::Error;
use crate::VideoSubsystem;
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ffi::CStr;

/// Methods from SDL's clipboard API.
impl VideoSubsystem {
    /// Query whether the clipboard exists and contains a non-empty text string.
    pub fn has_clipboard_text(&self) -> bool {
        unsafe { sys::SDL_HasClipboardText() }
    }

    /// Query whether the primary selection exists and contains a non-empty text string.
    pub fn has_primary_selection_text(&self) -> bool {
        unsafe { sys::SDL_HasPrimarySelectionText() }
    }

    /// Query whether there is data in the clipboard for the provided mime type.
    ///
    /// Returns an `Error` if `mime_type` contains an interior nul byte.
    pub fn has_clipboard_data(&self, mime_type: &str) -> Result<bool, Error> {
        let c_str = CString::new(mime_type)
            .map_err(|_| Error::register(c"Invalid mime type string format."))?;
        Ok(unsafe { sys::SDL_HasClipboardData(c_str.as_ptr()) })
    }

    /// Returns the UTF-8 text from the clipboard.
    ///
    /// This functions returns `None` if there was not enough memory left for a copy of the clipboard's content.
    pub fn clipboard_text(&self) -> Option<String> {
        unsafe {
            let ptr = sys::SDL_GetClipboardText();
            if ptr.is_null() {
                return None;
            }
            let c_str = CStr::from_ptr(ptr);
            if c_str.to_bytes().len() == 0 {
                return None;
            }
            // We're choosing to allocate here instead of converting to a &str because we're not
            // sure whether or not SDL will preserve the contents of the string for the lifetime
            // of &self.
            Some(c_str.to_string_lossy().into_owned())
        }
    }

    pub fn clipboard_data(&self, mime_type: &str) -> Result<Vec<u8>, Error> {
        unsafe {
            let mime_type = CString::new(mime_type)
                .map_err(|_| Error::register(c"Invalid string format for mime_type."))?;
            let mut size = 0;
            let ptr = sys::SDL_GetClipboardData(mime_type.as_ptr(), &raw mut size);
            if ptr.is_null() {
                return Err(Error);
            }
            Ok(core::slice::from_raw_parts(ptr as *mut u8, size).to_vec())
        }
    }

    /// Put UTF-8 text into the clipboard.
    pub fn set_clipboard_text(&mut self, text: &str) -> Result<(), Error> {
        let c_str = CString::new(text).map_err(|_| Error::register(c"Invalid string format."))?;
        let result = unsafe { sys::SDL_SetClipboardText(c_str.as_ptr()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Put UTF-8 text into the primary selection.
    pub fn set_primary_selection_text(&mut self, text: &str) -> Result<(), Error> {
        let c_str = CString::new(text).map_err(|_| Error::register(c"Invalid string format."))?;
        let result = unsafe { sys::SDL_SetPrimarySelectionText(c_str.as_ptr()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Clear the clipboard data.
    pub fn clear_clipboard_data(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_ClearClipboardData() };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Get UTF-8 text from the primary selection.
    ///
    /// This functions returns `Nonestring if there was not enough memory left for a copy of the
    /// primary selection's content.
    pub fn primary_selection_text(&self) -> Option<String> {
        unsafe {
            let ptr = sys::SDL_GetPrimarySelectionText();
            if ptr.is_null() {
                return None;
            }
            let c_str = CStr::from_ptr(ptr);
            if c_str.to_bytes().len() == 0 {
                return None;
            }
            // We're choosing to allocate here instead of converting to a &str because we're not
            // sure whether or not SDL will preserve the contents of the string for the lifetime
            // of &self.
            Some(c_str.to_string_lossy().into_owned())
        }
    }

    /// Retrieve the list of mime types available in the clipboard.
    pub fn mime_types(&self) -> Result<Vec<String>, Error> {
        let mut len = 0;
        unsafe {
            let mime_types = sys::SDL_GetClipboardMimeTypes(&raw mut len);
            if mime_types.is_null() {
                return Err(Error);
            }
            let array = core::slice::from_raw_parts(mime_types, len).iter();
            let mut vec = Vec::with_capacity(len);
            for &ptr in array {
                let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                vec.push(s);
            }
            sys::SDL_free(mime_types as *mut c_void);
            Ok(vec)
        }
    }
}
