use crate::{sys, Error};

/// A set of blend modes used in drawing operations.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
    Blend = sys::SDL_BLENDMODE_BLEND,
    BlendPremultiplied = sys::SDL_BLENDMODE_BLEND_PREMULTIPLIED,
    Add = sys::SDL_BLENDMODE_ADD,
    AddPremultipled = sys::SDL_BLENDMODE_ADD_PREMULTIPLIED,
    Mod = sys::SDL_BLENDMODE_MOD,
    Mul = sys::SDL_BLENDMODE_MUL,
    Invalid = sys::SDL_BLENDMODE_INVALID,
}

impl BlendMode {
    /// Converts a [`sys::SDL_BlendMode`] into a [`BlendMode`].
    /// Returns an [`Error`] if `value` is invalid.
    pub fn try_from_ll(value: sys::SDL_BlendMode) -> Result<Option<Self>, Error> {
        match value {
            sys::SDL_BLENDMODE_BLEND => Ok(Some(Self::Blend)),
            sys::SDL_BLENDMODE_BLEND_PREMULTIPLIED => Ok(Some(Self::BlendPremultiplied)),
            sys::SDL_BLENDMODE_ADD => Ok(Some(Self::Add)),
            sys::SDL_BLENDMODE_ADD_PREMULTIPLIED => Ok(Some(Self::AddPremultipled)),
            sys::SDL_BLENDMODE_MOD => Ok(Some(Self::Mod)),
            sys::SDL_BLENDMODE_MUL => Ok(Some(Self::Mul)),
            sys::SDL_BLENDMODE_INVALID => Ok(Some(Self::Invalid)),
            sys::SDL_BLENDMODE_NONE => Ok(None),
            _ => Err(Error::UnknownBlendMode(value)),
        }
    }

    /// Converts a [`BlendMode`] into [`sys::SDL_BlendMode`].
    #[inline]
    pub fn to_ll(&self) -> sys::SDL_BlendMode {
        *self as u32
    }
}
