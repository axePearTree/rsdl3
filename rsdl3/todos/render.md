Functions

- [x] SDL_CreateRenderer
- [x] SDL_CreateSoftwareRenderer
- [x] SDL_CreateTexture
- [x] SDL_CreateTextureFromSurface
- [x] SDL_DestroyRenderer
- [x] SDL_DestroyTexture
- [x] SDL_GetCurrentRenderOutputSize
- [x] SDL_GetRenderOutputSize
- [x] SDL_GetNumRenderDrivers
- [x] SDL_GetRenderClipRect
- [x] SDL_GetRenderColorScale
- [x] SDL_GetRenderDrawBlendMode
- [x] SDL_GetRenderDrawColor
- [x] SDL_GetRenderDrawColorFloat
- [x] SDL_SetRenderDrawColor
- [x] SDL_SetRenderDrawColorFloat
- [x] SDL_SetRenderDrawBlendMode
- [x] SDL_GetRendererName
- [x] SDL_GetRenderDriver
- [x] SDL_GetRenderWindow
- [x] SDL_GetRenderVSync
- [x] SDL_SetRenderVSync
- [x] SDL_RenderClear
- [x] SDL_RenderTexture
- [x] SDL_RenderFillRect
- [x] SDL_RenderFillRects
- [x] SDL_RenderDebugText
- [x] SDL_FlushRenderer
- [x] SDL_GetRenderSafeArea
- [x] SDL_GetRenderLogicalPresentation
- [x] SDL_SetRenderLogicalPresentation
- [x] SDL_ConvertEventToRenderCoordinates
- [x] SDL_RenderCoordinatesFromWindow
- [x] SDL_RenderCoordinatesToWindow
- [x] SDL_GetTextureAlphaMod
- [x] SDL_GetTextureAlphaModFloat
- [x] SDL_GetRenderScale
- [x] SDL_GetRenderTarget -- part of "replace_render_target" -- maybe we should expose a zst TextureRef??? is it useful to get a reference to the render target?
- [x] SDL_GetTextureBlendMode
- [x] SDL_GetTextureColorMod
- [x] SDL_GetTextureColorModFloat
- [x] SDL_SetTextureAlphaMod
- [x] SDL_SetTextureAlphaModFloat

- [ ] SDL_AddVulkanRenderSemaphores
- [ ] SDL_CreateRendererWithProperties
- [ ] SDL_CreateTextureWithProperties
- [ ] SDL_CreateWindowAndRenderer
- [ ] SDL_GetDefaultTextureScaleMode
- [ ] SDL_GetRendererProperties
- [ ] SDL_GetRenderLogicalPresentationRect
- [ ] SDL_GetRenderMetalCommandEncoder
- [ ] SDL_GetRenderMetalLayer
- [ ] SDL_GetRenderViewport
- [ ] SDL_GetTextureProperties
- [ ] SDL_GetTextureScaleMode
- [ ] SDL_GetTextureSize
- [ ] SDL_LockTexture
- [ ] SDL_LockTextureToSurface
- [ ] SDL_RenderClipEnabled
- [ ] SDL_RenderGeometry
- [ ] SDL_RenderGeometryRaw
- [ ] SDL_RenderLine
- [ ] SDL_RenderLines
- [ ] SDL_RenderPoint
- [ ] SDL_RenderPoints
- [ ] SDL_RenderPresent
- [ ] SDL_RenderReadPixels
- [ ] SDL_RenderRect
- [ ] SDL_RenderRects
- [ ] SDL_RenderTexture9Grid
- [ ] SDL_RenderTexture9GridTiled
- [ ] SDL_RenderTextureAffine
- [ ] SDL_RenderTextureRotated
- [ ] SDL_RenderTextureTiled
- [ ] SDL_RenderViewportSet
- [ ] SDL_SetDefaultTextureScaleMode
- [ ] SDL_SetRenderClipRect
- [ ] SDL_SetRenderColorScale
- [ ] SDL_SetRenderScale
- [ ] SDL_SetRenderTarget
- [ ] SDL_SetRenderViewport


- [ ] SDL_SetTextureBlendMode
- [ ] SDL_SetTextureColorMod
- [ ] SDL_SetTextureColorModFloat
- [ ] SDL_SetTextureScaleMode

- [ ] SDL_UnlockTexture
- [ ] SDL_UpdateNVTexture
- [ ] SDL_UpdateTexture
- [ ] SDL_UpdateYUVTexture

Datatypes

- [ ] SDL_Renderer

Structs

- [ ] SDL_Texture
- [ ] SDL_Vertex

Enums

- [ ] SDL_RendererLogicalPresentation
- [ ] SDL_TextureAccess

Not anytime soon

- [ ] SDL_RenderDebugTextFormat -- not impossible just weird - different from println! macro since it's a printf like function
- [ ] SDL_GetRenderer -- doesn't make any sense with our abstractions.
- [ ] SDL_GetRendererFromTexture -- can't make this safe, really.
