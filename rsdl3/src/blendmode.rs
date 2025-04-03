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
            _ => Err(Error::register(c"Unknown blend mode")),
        }
    }

    /// Compose a custom blend mode for renderers.
    ///
    /// The functions [`crate::render::Renderer::draw_blend_mode`] and [`crate::render::Texture::set_blend_mode`]
    /// accept the `BlendMode` returned by this function if the renderer supports it.
    ///
    /// A blend mode controls how the pixels from a drawing operation (source) get combined with the pixels from
    /// the render target (destination). First, the components of the source and destination pixels get multiplied
    /// with their blend factors. Then, the blend operation takes the two products and calculates the result that
    /// will get stored in the render target.
    ///
    /// Expressed in pseudocode, it would look like this:
    ///
    /// ```c
    /// dst_rgb = color_operation(src_rgb * src_color_factor, dst_rgb * dst_color_factor);
    /// dst_a = alpha_operation(src_a * src_alpha_factor, dst_a * dst_alpha_factor);
    /// ```
    ///
    /// Where the functions `color_operation(src, dst)` and `alpha_operation(src, dst)` can return one of the following:
    ///
    /// - `src + dst`
    /// - `src - dst`
    /// - `dst - src`
    /// - `min(src, dst)`
    /// - `max(src, dst)`
    ///
    /// The red, green, and blue components are always multiplied with the first, second, and third components of the
    /// `BlendFactor`, respectively. The fourth component is not used.
    ///
    /// The alpha component is always multiplied with the fourth component of the `BlendFactor`. The other components
    /// are not used in the alpha calculation.
    ///
    /// Support for these blend modes varies for each renderer. To check if a specific `BlendMode` is supported, create
    /// a renderer and pass it to either [`crate::render::Renderer::draw_blend_mode`] or
    /// [`crate::render::Texture::set_blend_mode`]. They will return an error if the blend mode is not supported.
    ///
    /// This list describes the support of custom blend modes for each renderer. All renderers support the four blend
    /// modes listed in the `BlendMode` enum.
    ///
    /// - **direct3d**: Supports all operations with all factors. However, some factors produce unexpected results with
    /// [`BlendOperation::Minimum`] and [`BlendOperation::Maximum`].
    /// - **direct3d11**: Same as Direct3D 9.
    /// - **opengl**: Supports the [`BlendOperation::Add`] operation with all factors. OpenGL versions 1.1, 1.2, and 1.3
    /// do not work correctly here.
    /// - **opengles2**: Supports the [`BlendOperation::Add`], [`BlendOperation::Subtract`], [`BlendOperation::RevSubtract`]
    /// operations with all factors.
    /// - **psp**: No custom blend mode support.
    /// - **software**: No custom blend mode support.
    ///
    /// Some renderers do not provide an alpha component for the default render target. The [`BlendFactor::DstAlpha`] and
    /// [`BlendFactor::OneMinusDstAlpha`] factors do not have an effect in this case.
    pub fn compose_custom(
        src_color_factor: BlendFactor,
        dst_color_factor: BlendFactor,
        color_operation: BlendOperation,
        src_alpha_factor: BlendFactor,
        dst_alpha_factor: BlendFactor,
        alpha_operation: BlendOperation,
    ) -> Result<Option<Self>, Error> {
        Self::try_from_ll(unsafe {
            sys::SDL_ComposeCustomBlendMode(
                src_color_factor.to_ll(),
                dst_color_factor.to_ll(),
                color_operation.to_ll(),
                src_alpha_factor.to_ll(),
                dst_alpha_factor.to_ll(),
                alpha_operation.to_ll(),
            )
        })
    }

    /// Converts a [`BlendMode`] into [`sys::SDL_BlendMode`].
    #[inline]
    pub fn to_ll(&self) -> sys::SDL_BlendMode {
        *self as u32
    }

    pub fn option_to_ll(mode: Option<BlendMode>) -> sys::SDL_BlendMode {
        match mode {
            Some(mode) => mode.to_ll(),
            None => sys::SDL_BLENDMODE_NONE,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum BlendFactor {
    /// 0, 0, 0, 0
    Zero = sys::SDL_BlendFactor_SDL_BLENDFACTOR_ZERO,
    /// 1, 1, 1, 1
    One = sys::SDL_BlendFactor_SDL_BLENDFACTOR_ONE,
    /// srcR, srcG, srcB, srcA
    SrcColor = sys::SDL_BlendFactor_SDL_BLENDFACTOR_SRC_COLOR,
    /// 1-srcR, 1-srcG, 1-srcB, 1-srcA
    OneMinusSrcColor = sys::SDL_BlendFactor_SDL_BLENDFACTOR_ONE_MINUS_SRC_COLOR,
    /// srcA, srcA, srcA, srcA
    SrcAlpha = sys::SDL_BlendFactor_SDL_BLENDFACTOR_SRC_ALPHA,
    /// 1-srcA, 1-srcA, 1-srcA, 1-srcA
    OneMinusSrcAlpha = sys::SDL_BlendFactor_SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
    /// dstR, dstG, dstB, dstA
    DstColor = sys::SDL_BlendFactor_SDL_BLENDFACTOR_DST_COLOR,
    /// 1-dstR, 1-dstG, 1-dstB, 1-dstA
    OneMinusDstColor = sys::SDL_BlendFactor_SDL_BLENDFACTOR_ONE_MINUS_DST_COLOR,
    /// dstA, dstA, dstA, dstA
    DstAlpha = sys::SDL_BlendFactor_SDL_BLENDFACTOR_DST_ALPHA,
    /// 1-dstA, 1-dstA, 1-dstA, 1-dstA
    OneMinusDstAlpha = sys::SDL_BlendFactor_SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
}

impl BlendFactor {
    /// Converts a [`BlendOperation`] into an integer.
    #[inline]
    pub fn to_ll(&self) -> u32 {
        *self as u32
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum BlendOperation {
    /// dst + src: supported by all renderers
    Add = sys::SDL_BlendOperation_SDL_BLENDOPERATION_ADD,
    /// src - dst : supported by D3D, OpenGL, OpenGLES, and Vulkan
    Subtract = sys::SDL_BlendOperation_SDL_BLENDOPERATION_SUBTRACT,
    /// dst - src : supported by D3D, OpenGL, OpenGLES, and Vulkan
    RevSubtract = sys::SDL_BlendOperation_SDL_BLENDOPERATION_REV_SUBTRACT,
    /// min(dst, src) : supported by D3D, OpenGL, OpenGLES, and Vulkan
    Minimum = sys::SDL_BlendOperation_SDL_BLENDOPERATION_MINIMUM,
    /// max(dst, src) : supported by D3D, OpenGL, OpenGLES, and Vulkan
    Maximum = sys::SDL_BlendOperation_SDL_BLENDOPERATION_MAXIMUM,
}

impl BlendOperation {
    /// Converts a [`BlendOperation`] into an integer.
    #[inline]
    pub fn to_ll(&self) -> u32 {
        *self as u32
    }
}
