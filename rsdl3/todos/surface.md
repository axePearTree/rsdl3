# Surface

## Functions

- [x] SDL_BlitSurface
- [x] SDL_BlitSurfaceScaled
- [x] SDL_ClearSurface
- [x] SDL_CreateSurface
- [x] SDL_LockSurface
- [x] SDL_DestroySurface
- [x] SDL_DuplicateSurface
- [x] SDL_CreateSurfaceFrom
- [x] SDL_ConvertSurface
- [x] SDL_FillSurfaceRect
- [x] SDL_FillSurfaceRects
- [x] SDL_FlipSurface
- [x] SDL_GetSurfaceAlphaMod
- [x] SDL_GetSurfaceBlendMode
- [x] SDL_GetSurfaceClipRect
- [x] SDL_GetSurfaceColorKey
- [x] SDL_GetSurfaceColorMod
- [x] SDL_GetSurfaceColorspace
- [x] SDL_BlitSurface9Grid
- [x] SDL_BlitSurfaceTiled
- [x] SDL_BlitSurfaceTiledWithScale
- [x] SDL_SetSurfaceAlphaMod
- [x] SDL_SetSurfaceBlendMode
- [x] SDL_SetSurfaceClipRect
- [x] SDL_SetSurfaceColorKey
- [x] SDL_SetSurfaceColorMod
- [x] SDL_SetSurfaceColorspace
- [x] SDL_UnlockSurface
- [x] SDL_WriteSurfacePixel
- [x] SDL_WriteSurfacePixelFloat
- [x] SDL_SurfaceHasColorKey
- [x] SDL_SetSurfacePalette
- [x] SDL_GetSurfacePalette
- [x] SDL_ReadSurfacePixel
- [x] SDL_ReadSurfacePixelFloat
- [x] SDL_MapSurfaceRGB
- [x] SDL_MapSurfaceRGBA
- [x] SDL_PremultiplySurfaceAlpha
- [x] SDL_ScaleSurface
- [x] SDL_LoadBMP
- [x] SDL_SaveBMP
- [x] SDL_LoadBMP_IO
- [x] SDL_SaveBMP_IO
- [x] SDL_SetSurfaceRLE
- [x] SDL_SurfaceHasRLE

## Extra annoying:

- [ ] SDL_CreateSurfacePalette -- ?

No safety limitations on these functions. Just too many params. Have to settle on an API.

- [ ] SDL_PremultiplyAlpha
- [ ] SDL_ConvertSurfaceAndColorspace
- [ ] SDL_ConvertPixels
- [ ] SDL_ConvertPixelsAndColorspace

- [ ] SDL_GetSurfaceProperties -- not doing SDL properties for now.
- [ ] SDL_GetSurfaceImages -- nope

## Probably not gonna happen:

- [ ] SDL_BlitSurfaceUnchecked
- [ ] SDL_BlitSurfaceUncheckedScaled

Need to check whether or not Surfaces are RC'd
- [ ] SDL_AddSurfaceAlternateImage
- [ ] SDL_SurfaceHasAlternateImages
- [ ] SDL_RemoveSurfaceAlternateImages
