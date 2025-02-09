# SDL video category

- [x] SDL_CreateWindow
- [x] SDL_DestroyWindow
- [x] SDL_FlashWindow
- [x] SDL_GetWindowFlags
- [x] SDL_GetWindowID
- [x] SDL_GetWindowOpacity
- [x] SDL_GetWindowPosition
- [x] SDL_GetWindowSize
- [x] SDL_GetWindowTitle
- [x] SDL_HideWindow
- [x] SDL_MaximizeWindow
- [x] SDL_MinimizeWindow
- [x] SDL_RaiseWindow
- [x] SDL_SetWindowOpacity
- [x] SDL_SetWindowResizable
- [x] SDL_SetWindowSize
- [x] SDL_SetWindowTitle
- [x] SDL_ShowWindow
- [x] SDL_RestoreWindow
- [x] SDL_SetWindowFullscreen
- [x] SDL_GetDisplays
- [x] SDL_GetDisplayName
- [x] SDL_GetDisplayForWindow
- [x] SDL_GetWindowMaximumSize
- [x] SDL_GetWindowMinimumSize
- [x] SDL_GetWindowAspectRatio - weird behaviour with aspect ratio returning (0.0, 0.0)
- [x] SDL_SetWindowPosition
- [x] SDL_DisableScreenSaver
- [x] SDL_EnableScreenSaver
- [x] SDL_GetDisplayBounds
- [x] SDL_GetCurrentDisplayMode
- [x] SDL_GetClosestFullscreenDisplayMode
- [x] SDL_SetWindowAspectRatio
- [x] SDL_DestroyWindowSurface
- [x] SDL_GetVideoDriver
- [x] SDL_GetNumVideoDrivers
- [x] SDL_GetCurrentVideoDriver
- [x] SDL_GetWindowPixelFormat
- [x] SDL_WindowHasSurface
- [x] SDL_UpdateWindowSurface
- [x] SDL_SetWindowMaximumSize
- [x] SDL_SetWindowMinimumSize
- [x] SDL_GetDesktopDisplayMode
- [x] SDL_GetWindowDisplayScale
- [x] SDL_GetWindowMouseRect
- [x] SDL_SetWindowMouseRect
- [x] SDL_ScreenSaverEnabled
- [x] SDL_GetPrimaryDisplay
- [x] SDL_GetDisplayUsableBounds
- [x] SDL_GetWindowSurface
- [x] SDL_GetWindowPixelDensity
- [x] SDL_SetWindowBordered
- [x] SDL_SetWindowAlwaysOnTop
- [x] SDL_GetWindowSafeArea
- [x] SDL_GetWindowMouseGrab
- [x] SDL_SetWindowFocusable
- [x] SDL_SetWindowIcon
- [x] SDL_SetWindowMouseGrab
- [x] SDL_GetWindowSizeInPixels
- [x] SDL_GetDisplayForRect
- [x] SDL_GetDisplayForPoint
- [x] SDL_GetDisplayContentScale
- [x] SDL_GetCurrentDisplayOrientation
- [x] SDL_GetNaturalDisplayOrientation
- [x] SDL_GetFullscreenDisplayModes
- [x] SDL_GetWindowBordersSize
- [x] SDL_GetWindowFullscreenMode
- [x] SDL_SetWindowFullscreenMode
- [x] SDL_GetWindowKeyboardGrab
- [x] SDL_SetWindowKeyboardGrab
- [x] SDL_GetWindowSurfaceVSync
- [x] SDL_SetWindowShape
- [x] SDL_SetWindowSurfaceVSync
- [x] SDL_ShowWindowSystemMenu
- [x] SDL_SyncWindow
- [x] SDL_GetSystemTheme
- [x] SDL_UpdateWindowSurfaceRects

## Lower priority

Bunch of extension methods related to EGL and OpenGL. These are important and we should eventually add them!

- [ ] SDL_EGL_GetCurrentConfig
- [ ] SDL_EGL_GetCurrentDisplay
- [ ] SDL_EGL_GetProcAddress
- [ ] SDL_EGL_GetWindowSurface
- [ ] SDL_EGL_SetAttributeCallbacks
- [ ] SDL_GL_CreateContext
- [ ] SDL_GL_DestroyContext
- [ ] SDL_GL_ExtensionSupported
- [ ] SDL_GL_GetAttribute
- [ ] SDL_GL_GetCurrentContext
- [ ] SDL_GL_GetCurrentWindow
- [ ] SDL_GL_GetProcAddress
- [ ] SDL_GL_GetSwapInterval
- [ ] SDL_GL_LoadLibrary
- [ ] SDL_GL_MakeCurrent
- [ ] SDL_GL_ResetAttributes
- [ ] SDL_GL_SetAttribute
- [ ] SDL_GL_SetSwapInterval
- [ ] SDL_GL_SwapWindow
- [ ] SDL_GL_UnloadLibrary

## Need to do, but probably hard.

1. Need to basically create a hierarchy of windows. Always a hard ask because of the borrow checker... Might need to refactor a lot.

- [ ] SDL_CreatePopupWindow
- [ ] SDL_SetWindowParent
- [ ] SDL_GetWindowParent

2. Involves C function callbacks

- [ ] SDL_SetWindowHitTest

3. Involves parsing raw data (we don't know the exact format)

- [ ] SDL_GetWindowICCProfile

## Need to do but DEFINITELY hard

Functions that return window pointers are really really hard to wrap safely as they'll definitely alias with existing windows.

- [ ] SDL_GetWindowFromID
- [ ] SDL_GetWindows
- [ ] SDL_GetGrabbedWindow
- [ ] SDL_SetWindowModal - this one is not hard to implement but makes no sense without nested windows.

## Shit ton of work

Anything involving properties will force us to actually implement properties and that sucks.

- [ ] SDL_CreateWindowWithProperties
- [ ] SDL_GetDisplayProperties
- [ ] SDL_GetWindowProperties
