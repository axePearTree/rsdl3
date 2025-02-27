use crate::{sys, Error};

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
    pub fn try_from_ll(value: u32) -> Result<Option<Self>, Error> {
        match value {
            sys::SDL_BLENDMODE_BLEND => Ok(Some(Self::Blend)),
            sys::SDL_BLENDMODE_BLEND_PREMULTIPLIED => Ok(Some(Self::BlendPremultiplied)),
            sys::SDL_BLENDMODE_ADD => Ok(Some(Self::Add)),
            sys::SDL_BLENDMODE_ADD_PREMULTIPLIED => Ok(Some(Self::AddPremultipled)),
            sys::SDL_BLENDMODE_MOD => Ok(Some(Self::Mod)),
            sys::SDL_BLENDMODE_MUL => Ok(Some(Self::Mul)),
            sys::SDL_BLENDMODE_INVALID => Ok(Some(Self::Invalid)),
            sys::SDL_BLENDMODE_NONE => Ok(None),
            _ => Err(Error::new("Unknown blend mode.")),
        }
    }

    #[inline]
    pub fn to_ll(&self) -> u32 {
        *self as u32
    }
}
