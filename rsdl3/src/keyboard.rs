use core::ffi::CStr;
use core::marker::PhantomData;
use core::ptr::NonNull;

use alloc::string::String;

use crate::sys;
use crate::Error;
use crate::EventsSubsystem;

pub type KeyboardId = sys::SDL_KeyboardID;

/// Methods from SDL's keyboard API.
impl EventsSubsystem {
    /// Returns a list of currently connected keyboards.
    ///
    /// Note that this will include any device or virtual driver that includes keyboard functionality,
    /// including some mice, KVM switches, motherboard power buttons, etc. You should wait for input
    /// from a device before you consider it actively in use.
    pub fn keyboards(&self) -> Result<Keyboards, Error> {
        let mut count = 0;
        let keyboards = unsafe { sys::SDL_GetKeyboards(&raw mut count) };
        let ptr = NonNull::new(keyboards).ok_or(Error::new())?;
        Ok(Keyboards { ptr })
    }

    /// Returns the name of a keyboard.
    ///
    /// This function returns `Ok(None)` if the keyboard doesn't have a name.
    pub fn keyboard_name(&self, id: KeyboardId) -> Result<Option<String>, Error> {
        unsafe {
            let ptr = sys::SDL_GetKeyboardNameForID(id);
            if ptr.is_null() {
                return Err(Error::new());
            }
            let name = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            if name.is_empty() {
                return Ok(None);
            }
            Ok(Some(name))
        }
    }

    /// Returns a snapshot of the current state of the keyboard.
    pub fn keyboard_state(&self) -> Result<KeyboardState, Error> {
        unsafe {
            let mut numkeys = 0;
            let state = sys::SDL_GetKeyboardState(&raw mut numkeys);
            let numkeys = usize::try_from(numkeys)?;
            let ptr = NonNull::new(state as *mut _).ok_or(Error::new())?;
            Ok(KeyboardState { ptr, numkeys })
        }
    }
}

/// A view into the current state of the keyboard.
///
/// Use [`EventPump::pump_events`] to update the internal values.
///
/// This struct gives you the current state after all events have been processed, so if a key
/// or button has been pressed and released before you process events, then the pressed state
/// will never show up in the [`KeyboardState::get`] calls.
pub struct KeyboardState {
    ptr: NonNull<bool>,
    numkeys: usize,
}

impl KeyboardState {
    /// Returns the current state of a given scancode in this keyboard.
    ///
    /// If the scancode is not present in this keyboard, this function will return `None`.
    pub fn get(&self, scancode: Scancode) -> bool {
        let index = scancode.as_index();
        if index >= self.numkeys {
            return false;
        }
        let Ok(offset) = isize::try_from(index) else {
            return false;
        };
        unsafe { *self.ptr.as_ptr().offset(offset) }
    }
}

pub struct Keyboards {
    ptr: NonNull<sys::SDL_KeyboardID>,
}

impl Keyboards {
    pub fn iter(&self) -> KeyboardsIter {
        KeyboardsIter {
            ptr: self.ptr,
            _m: PhantomData,
        }
    }
}

impl Drop for Keyboards {
    fn drop(&mut self) {
        unsafe {
            sys::SDL_free(self.ptr.as_ptr() as *mut _);
        }
    }
}

pub struct KeyboardsIter<'a> {
    ptr: NonNull<sys::SDL_KeyboardID>,
    _m: PhantomData<&'a ()>,
}

impl Iterator for KeyboardsIter<'_> {
    type Item = KeyboardId;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY:
        // * The struct borrows Keyboards for its' lifetime.
        // * The ptr array is null-terminated.
        unsafe {
            let ptr = self.ptr.as_ptr();
            if *ptr == 0 {
                return None;
            }
            self.ptr = self.ptr.offset(1);
            Some(*ptr)
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Scancode {
    Unknown = sys::SDL_Scancode_SDL_SCANCODE_UNKNOWN,
    A = sys::SDL_Scancode_SDL_SCANCODE_A,
    B = sys::SDL_Scancode_SDL_SCANCODE_B,
    C = sys::SDL_Scancode_SDL_SCANCODE_C,
    D = sys::SDL_Scancode_SDL_SCANCODE_D,
    E = sys::SDL_Scancode_SDL_SCANCODE_E,
    F = sys::SDL_Scancode_SDL_SCANCODE_F,
    G = sys::SDL_Scancode_SDL_SCANCODE_G,
    H = sys::SDL_Scancode_SDL_SCANCODE_H,
    I = sys::SDL_Scancode_SDL_SCANCODE_I,
    J = sys::SDL_Scancode_SDL_SCANCODE_J,
    K = sys::SDL_Scancode_SDL_SCANCODE_K,
    L = sys::SDL_Scancode_SDL_SCANCODE_L,
    M = sys::SDL_Scancode_SDL_SCANCODE_M,
    N = sys::SDL_Scancode_SDL_SCANCODE_N,
    O = sys::SDL_Scancode_SDL_SCANCODE_O,
    P = sys::SDL_Scancode_SDL_SCANCODE_P,
    Q = sys::SDL_Scancode_SDL_SCANCODE_Q,
    R = sys::SDL_Scancode_SDL_SCANCODE_R,
    S = sys::SDL_Scancode_SDL_SCANCODE_S,
    T = sys::SDL_Scancode_SDL_SCANCODE_T,
    U = sys::SDL_Scancode_SDL_SCANCODE_U,
    V = sys::SDL_Scancode_SDL_SCANCODE_V,
    W = sys::SDL_Scancode_SDL_SCANCODE_W,
    X = sys::SDL_Scancode_SDL_SCANCODE_X,
    Y = sys::SDL_Scancode_SDL_SCANCODE_Y,
    Z = sys::SDL_Scancode_SDL_SCANCODE_Z,
    Num1 = sys::SDL_Scancode_SDL_SCANCODE_1,
    Num2 = sys::SDL_Scancode_SDL_SCANCODE_2,
    Num3 = sys::SDL_Scancode_SDL_SCANCODE_3,
    Num4 = sys::SDL_Scancode_SDL_SCANCODE_4,
    Num5 = sys::SDL_Scancode_SDL_SCANCODE_5,
    Num6 = sys::SDL_Scancode_SDL_SCANCODE_6,
    Num7 = sys::SDL_Scancode_SDL_SCANCODE_7,
    Num8 = sys::SDL_Scancode_SDL_SCANCODE_8,
    Num9 = sys::SDL_Scancode_SDL_SCANCODE_9,
    Num0 = sys::SDL_Scancode_SDL_SCANCODE_0,
    Return = sys::SDL_Scancode_SDL_SCANCODE_RETURN,
    Escape = sys::SDL_Scancode_SDL_SCANCODE_ESCAPE,
    Backspace = sys::SDL_Scancode_SDL_SCANCODE_BACKSPACE,
    Tab = sys::SDL_Scancode_SDL_SCANCODE_TAB,
    Space = sys::SDL_Scancode_SDL_SCANCODE_SPACE,
    Minus = sys::SDL_Scancode_SDL_SCANCODE_MINUS,
    Equals = sys::SDL_Scancode_SDL_SCANCODE_EQUALS,
    LeftBracket = sys::SDL_Scancode_SDL_SCANCODE_LEFTBRACKET,
    RightBracket = sys::SDL_Scancode_SDL_SCANCODE_RIGHTBRACKET,
    /// Located at the lower left of the return key on ISO keyboards and at the right end
    /// of the QWERTY row on ANSI keyboards.
    /// Produces REVERSE SOLIDUS (backslash) and VERTICAL LINE in a US layout, REVERSE
    /// SOLIDUS and VERTICAL LINE in a UK Mac layout, NUMBER SIGN and TILDE in a UK
    /// Windows layout, DOLLAR SIGN and POUND SIGN in a Swiss German layout, NUMBER SIGN and
    /// APOSTROPHE in a German layout, GRAVE ACCENT and POUND SIGN in a French Mac
    /// layout, and ASTERISK and MICRO SIGN in a French Windows layout.
    Backslash = sys::SDL_Scancode_SDL_SCANCODE_BACKSLASH,
    /// ISO USB keyboards actually use this code instead of 49 for the same key, but all
    /// OSes I've seen treat the two codes identically. So, as an implementor, unless
    /// your keyboard generates both of those codes and your OS treats them differently,
    /// you should generate SDL_SCANCODE_BACKSLASH instead of this code. As a user, you
    /// should not rely on this code because SDL will never generate it with most (all?)
    /// keyboards.
    NonUSHash = sys::SDL_Scancode_SDL_SCANCODE_NONUSHASH,
    Semicolon = sys::SDL_Scancode_SDL_SCANCODE_SEMICOLON,
    Apostrophe = sys::SDL_Scancode_SDL_SCANCODE_APOSTROPHE,
    /// Located in the top left corner (on both ANSI and ISO keyboards). Produces GRAVE ACCENT and
    /// TILDE in a US Windows layout and in US and UK Mac layouts on ANSI keyboards, GRAVE ACCENT
    /// and NOT SIGN in a UK Windows layout, SECTION SIGN and PLUS-MINUS SIGN in US and UK Mac
    /// layouts on ISO keyboards, SECTION SIGN and DEGREE SIGN in a Swiss German layout (Mac:
    /// only on ISO keyboards), CIRCUMFLEX ACCENT and DEGREE SIGN in a German layout (Mac: only on
    /// ISO keyboards), SUPERSCRIPT TWO and TILDE in a French Windows layout, COMMERCIAL AT and
    /// NUMBER SIGN in a French Mac layout on ISO keyboards, and LESS-THAN SIGN and GREATER-THAN
    /// SIGN in a Swiss German, German, or French Mac layout on ANSI keyboards.
    Grave = sys::SDL_Scancode_SDL_SCANCODE_GRAVE,
    Comma = sys::SDL_Scancode_SDL_SCANCODE_COMMA,
    Period = sys::SDL_Scancode_SDL_SCANCODE_PERIOD,
    Slash = sys::SDL_Scancode_SDL_SCANCODE_SLASH,

    CapsLock = sys::SDL_Scancode_SDL_SCANCODE_CAPSLOCK,

    F1 = sys::SDL_Scancode_SDL_SCANCODE_F1,
    F2 = sys::SDL_Scancode_SDL_SCANCODE_F2,
    F3 = sys::SDL_Scancode_SDL_SCANCODE_F3,
    F4 = sys::SDL_Scancode_SDL_SCANCODE_F4,
    F5 = sys::SDL_Scancode_SDL_SCANCODE_F5,
    F6 = sys::SDL_Scancode_SDL_SCANCODE_F6,
    F7 = sys::SDL_Scancode_SDL_SCANCODE_F7,
    F8 = sys::SDL_Scancode_SDL_SCANCODE_F8,
    F9 = sys::SDL_Scancode_SDL_SCANCODE_F9,
    F10 = sys::SDL_Scancode_SDL_SCANCODE_F10,
    F11 = sys::SDL_Scancode_SDL_SCANCODE_F11,
    F12 = sys::SDL_Scancode_SDL_SCANCODE_F12,

    PrintScreen = sys::SDL_Scancode_SDL_SCANCODE_PRINTSCREEN,
    ScrollLock = sys::SDL_Scancode_SDL_SCANCODE_SCROLLLOCK,
    Pause = sys::SDL_Scancode_SDL_SCANCODE_PAUSE,
    /// Insert on PC, help on some Mac keyboards (but does send code 73, not 117)
    Insert = sys::SDL_Scancode_SDL_SCANCODE_INSERT,
    Home = sys::SDL_Scancode_SDL_SCANCODE_HOME,
    PageUp = sys::SDL_Scancode_SDL_SCANCODE_PAGEUP,
    Delete = sys::SDL_Scancode_SDL_SCANCODE_DELETE,
    End = sys::SDL_Scancode_SDL_SCANCODE_END,
    PageDown = sys::SDL_Scancode_SDL_SCANCODE_PAGEDOWN,
    Right = sys::SDL_Scancode_SDL_SCANCODE_RIGHT,
    Left = sys::SDL_Scancode_SDL_SCANCODE_LEFT,
    Down = sys::SDL_Scancode_SDL_SCANCODE_DOWN,
    Up = sys::SDL_Scancode_SDL_SCANCODE_UP,

    /// Num lock on PC, clear on Mac keyboards
    NumLockClear = sys::SDL_Scancode_SDL_SCANCODE_NUMLOCKCLEAR,
    KpDivide = sys::SDL_Scancode_SDL_SCANCODE_KP_DIVIDE,
    KpMultiply = sys::SDL_Scancode_SDL_SCANCODE_KP_MULTIPLY,
    KpMinus = sys::SDL_Scancode_SDL_SCANCODE_KP_MINUS,
    KpPlus = sys::SDL_Scancode_SDL_SCANCODE_KP_PLUS,
    KpEnter = sys::SDL_Scancode_SDL_SCANCODE_KP_ENTER,
    Kp1 = sys::SDL_Scancode_SDL_SCANCODE_KP_1,
    Kp2 = sys::SDL_Scancode_SDL_SCANCODE_KP_2,
    Kp3 = sys::SDL_Scancode_SDL_SCANCODE_KP_3,
    Kp4 = sys::SDL_Scancode_SDL_SCANCODE_KP_4,
    Kp5 = sys::SDL_Scancode_SDL_SCANCODE_KP_5,
    Kp6 = sys::SDL_Scancode_SDL_SCANCODE_KP_6,
    Kp7 = sys::SDL_Scancode_SDL_SCANCODE_KP_7,
    Kp8 = sys::SDL_Scancode_SDL_SCANCODE_KP_8,
    Kp9 = sys::SDL_Scancode_SDL_SCANCODE_KP_9,
    Kp0 = sys::SDL_Scancode_SDL_SCANCODE_KP_0,
    KpPeriod = sys::SDL_Scancode_SDL_SCANCODE_KP_PERIOD,
    /// This is the additional key that ISO keyboards have over ANSI ones,
    /// located between left shift and Y. Produces GRAVE ACCENT and TILDE in a
    /// US or UK Mac layout, REVERSE SOLIDUS (backslash) and VERTICAL LINE in a
    /// US or UK Windows layout, and LESS-THAN SIGN and GREATER-THAN SIGN
    /// in a Swiss German, German, or French layout.
    NonUSBackslash = sys::SDL_Scancode_SDL_SCANCODE_NONUSBACKSLASH,
    /// Windows contextual menu, compose
    Application = sys::SDL_Scancode_SDL_SCANCODE_APPLICATION,
    /// The USB document says this is a status flag, not a physical key - but some Mac keyboards
    /// do have a power key.
    Power = sys::SDL_Scancode_SDL_SCANCODE_POWER,
    KpEquals = sys::SDL_Scancode_SDL_SCANCODE_KP_EQUALS,
    F13 = sys::SDL_Scancode_SDL_SCANCODE_F13,
    F14 = sys::SDL_Scancode_SDL_SCANCODE_F14,
    F15 = sys::SDL_Scancode_SDL_SCANCODE_F15,
    F16 = sys::SDL_Scancode_SDL_SCANCODE_F16,
    F17 = sys::SDL_Scancode_SDL_SCANCODE_F17,
    F18 = sys::SDL_Scancode_SDL_SCANCODE_F18,
    F19 = sys::SDL_Scancode_SDL_SCANCODE_F19,
    F20 = sys::SDL_Scancode_SDL_SCANCODE_F20,
    F21 = sys::SDL_Scancode_SDL_SCANCODE_F21,
    F22 = sys::SDL_Scancode_SDL_SCANCODE_F22,
    F23 = sys::SDL_Scancode_SDL_SCANCODE_F23,
    F24 = sys::SDL_Scancode_SDL_SCANCODE_F24,
    Execute = sys::SDL_Scancode_SDL_SCANCODE_EXECUTE,
    /// AL Integrated Help Center
    Help = sys::SDL_Scancode_SDL_SCANCODE_HELP,
    /// Menu (show menu)
    Menu = sys::SDL_Scancode_SDL_SCANCODE_MENU,
    Select = sys::SDL_Scancode_SDL_SCANCODE_SELECT,
    /// AC Stop
    Stop = sys::SDL_Scancode_SDL_SCANCODE_STOP,
    /// AC Redo/Repeat
    Again = sys::SDL_Scancode_SDL_SCANCODE_AGAIN,
    /// AC Undo
    Undo = sys::SDL_Scancode_SDL_SCANCODE_UNDO,
    /// AC Cut
    Cut = sys::SDL_Scancode_SDL_SCANCODE_CUT,
    /// AC Copy
    Copy = sys::SDL_Scancode_SDL_SCANCODE_COPY,
    /// AC Paste
    Paste = sys::SDL_Scancode_SDL_SCANCODE_PASTE,
    /// AC Find
    Find = sys::SDL_Scancode_SDL_SCANCODE_FIND,
    Mute = sys::SDL_Scancode_SDL_SCANCODE_MUTE,
    VolumeUp = sys::SDL_Scancode_SDL_SCANCODE_VOLUMEUP,
    VolumeDown = sys::SDL_Scancode_SDL_SCANCODE_VOLUMEDOWN,

    KpComma = sys::SDL_Scancode_SDL_SCANCODE_KP_COMMA,
    KpEqualsAs400 = sys::SDL_Scancode_SDL_SCANCODE_KP_EQUALSAS400,

    /// Used on Asian keyboards, see footnotes in USB doc
    International1 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL1,
    International2 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL2,
    /// Yen
    International3 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL3,
    International4 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL4,
    International5 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL5,
    International6 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL6,
    International7 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL7,
    International8 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL8,
    International9 = sys::SDL_Scancode_SDL_SCANCODE_INTERNATIONAL9,
    /// Hangul/English toggle
    Lang1 = sys::SDL_Scancode_SDL_SCANCODE_LANG1,
    /// Hanja conversion
    Lang2 = sys::SDL_Scancode_SDL_SCANCODE_LANG2,
    /// Katakana
    Lang3 = sys::SDL_Scancode_SDL_SCANCODE_LANG3,
    /// Hiragana
    Lang4 = sys::SDL_Scancode_SDL_SCANCODE_LANG4,
    /// Zenkaku/Hankaku
    Lang5 = sys::SDL_Scancode_SDL_SCANCODE_LANG5,
    /// Reserved
    Lang6 = sys::SDL_Scancode_SDL_SCANCODE_LANG6,
    /// Reserved
    Lang7 = sys::SDL_Scancode_SDL_SCANCODE_LANG7,
    /// Reserved
    Lang8 = sys::SDL_Scancode_SDL_SCANCODE_LANG8,
    /// Reserved
    Lang9 = sys::SDL_Scancode_SDL_SCANCODE_LANG9,
    /// Erase-Eaze
    AltErase = sys::SDL_Scancode_SDL_SCANCODE_ALTERASE,
    SysReq = sys::SDL_Scancode_SDL_SCANCODE_SYSREQ,
    /// AC Cancel
    Cancel = sys::SDL_Scancode_SDL_SCANCODE_CANCEL,
    Clear = sys::SDL_Scancode_SDL_SCANCODE_CLEAR,
    Prior = sys::SDL_Scancode_SDL_SCANCODE_PRIOR,
    Return2 = sys::SDL_Scancode_SDL_SCANCODE_RETURN2,
    Separator = sys::SDL_Scancode_SDL_SCANCODE_SEPARATOR,
    Out = sys::SDL_Scancode_SDL_SCANCODE_OUT,
    Oper = sys::SDL_Scancode_SDL_SCANCODE_OPER,
    ClearAgain = sys::SDL_Scancode_SDL_SCANCODE_CLEARAGAIN,
    CrSel = sys::SDL_Scancode_SDL_SCANCODE_CRSEL,
    ExSel = sys::SDL_Scancode_SDL_SCANCODE_EXSEL,

    Kp00 = sys::SDL_Scancode_SDL_SCANCODE_KP_00,
    Kp000 = sys::SDL_Scancode_SDL_SCANCODE_KP_000,
    ThousandsSeparator = sys::SDL_Scancode_SDL_SCANCODE_THOUSANDSSEPARATOR,
    DecimalSeparator = sys::SDL_Scancode_SDL_SCANCODE_DECIMALSEPARATOR,
    CurrencyUnit = sys::SDL_Scancode_SDL_SCANCODE_CURRENCYUNIT,
    CurrencySubUnit = sys::SDL_Scancode_SDL_SCANCODE_CURRENCYSUBUNIT,
    KpLeftParen = sys::SDL_Scancode_SDL_SCANCODE_KP_LEFTPAREN,
    KpRightParen = sys::SDL_Scancode_SDL_SCANCODE_KP_RIGHTPAREN,
    KpLeftBrace = sys::SDL_Scancode_SDL_SCANCODE_KP_LEFTBRACE,
    KpRightBrace = sys::SDL_Scancode_SDL_SCANCODE_KP_RIGHTBRACE,
    KpTab = sys::SDL_Scancode_SDL_SCANCODE_KP_TAB,
    KpBackspace = sys::SDL_Scancode_SDL_SCANCODE_KP_BACKSPACE,
    KpA = sys::SDL_Scancode_SDL_SCANCODE_KP_A,
    KpB = sys::SDL_Scancode_SDL_SCANCODE_KP_B,
    KpC = sys::SDL_Scancode_SDL_SCANCODE_KP_C,
    KpD = sys::SDL_Scancode_SDL_SCANCODE_KP_D,
    KpE = sys::SDL_Scancode_SDL_SCANCODE_KP_E,
    KpF = sys::SDL_Scancode_SDL_SCANCODE_KP_F,
    KpXor = sys::SDL_Scancode_SDL_SCANCODE_KP_XOR,
    KpPower = sys::SDL_Scancode_SDL_SCANCODE_KP_POWER,
    KpPercent = sys::SDL_Scancode_SDL_SCANCODE_KP_PERCENT,
    KpLess = sys::SDL_Scancode_SDL_SCANCODE_KP_LESS,
    KpGreater = sys::SDL_Scancode_SDL_SCANCODE_KP_GREATER,
    KpAmpersand = sys::SDL_Scancode_SDL_SCANCODE_KP_AMPERSAND,
    KpDblAmpersand = sys::SDL_Scancode_SDL_SCANCODE_KP_DBLAMPERSAND,
    KpVerticalBar = sys::SDL_Scancode_SDL_SCANCODE_KP_VERTICALBAR,
    KpDblVerticalBar = sys::SDL_Scancode_SDL_SCANCODE_KP_DBLVERTICALBAR,
    KpColon = sys::SDL_Scancode_SDL_SCANCODE_KP_COLON,
    KpHash = sys::SDL_Scancode_SDL_SCANCODE_KP_HASH,
    KpSpace = sys::SDL_Scancode_SDL_SCANCODE_KP_SPACE,
    KpAt = sys::SDL_Scancode_SDL_SCANCODE_KP_AT,
    KpExclam = sys::SDL_Scancode_SDL_SCANCODE_KP_EXCLAM,
    KpMemStore = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMSTORE,
    KpMemRecall = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMRECALL,
    KpMemClear = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMCLEAR,
    KpMemAdd = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMADD,
    KpMemSubtract = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMSUBTRACT,
    KpMemMultiply = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMMULTIPLY,
    KpMemDivide = sys::SDL_Scancode_SDL_SCANCODE_KP_MEMDIVIDE,
    KpPlusMinus = sys::SDL_Scancode_SDL_SCANCODE_KP_PLUSMINUS,
    KpClear = sys::SDL_Scancode_SDL_SCANCODE_KP_CLEAR,
    KpClearEntry = sys::SDL_Scancode_SDL_SCANCODE_KP_CLEARENTRY,
    KpBinary = sys::SDL_Scancode_SDL_SCANCODE_KP_BINARY,
    KpOctal = sys::SDL_Scancode_SDL_SCANCODE_KP_OCTAL,
    KpDecimal = sys::SDL_Scancode_SDL_SCANCODE_KP_DECIMAL,
    KpHexadecimal = sys::SDL_Scancode_SDL_SCANCODE_KP_HEXADECIMAL,

    LCtrl = sys::SDL_Scancode_SDL_SCANCODE_LCTRL,
    LShift = sys::SDL_Scancode_SDL_SCANCODE_LSHIFT,
    /// Alt, option
    LAlt = sys::SDL_Scancode_SDL_SCANCODE_LALT,
    /// Windows, command (apple), meta
    LGui = sys::SDL_Scancode_SDL_SCANCODE_LGUI,
    RCtrl = sys::SDL_Scancode_SDL_SCANCODE_RCTRL,
    RShift = sys::SDL_Scancode_SDL_SCANCODE_RSHIFT,
    /// Alt gr, option
    RAlt = sys::SDL_Scancode_SDL_SCANCODE_RALT,
    /// Windows, command (apple), meta
    RGui = sys::SDL_Scancode_SDL_SCANCODE_RGUI,
    /// I'm not sure if this is really not covered by any of the above, but since there's a
    /// special SDL_KMOD_MODE for it I'm adding it here
    Mode = sys::SDL_Scancode_SDL_SCANCODE_MODE,
    /// Sleep
    Sleep = sys::SDL_Scancode_SDL_SCANCODE_SLEEP,
    /// Wake
    Wake = sys::SDL_Scancode_SDL_SCANCODE_WAKE,
    /// Channel Increment
    ChannelIncrement = sys::SDL_Scancode_SDL_SCANCODE_CHANNEL_INCREMENT,
    /// Channel Decrement
    ChannelDecrement = sys::SDL_Scancode_SDL_SCANCODE_CHANNEL_DECREMENT,
    /// Play
    MediaPlay = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_PLAY,
    /// Pause
    MediaPause = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_PAUSE,
    /// Record
    MediaRecord = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_RECORD,
    /// Fast Forward
    MediaFastForward = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_FAST_FORWARD,
    /// Rewind
    MediaRewind = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_REWIND,
    /// Next Track
    MediaNextTrack = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_NEXT_TRACK,
    /// Previous Track
    MediaPreviousTrack = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_PREVIOUS_TRACK,
    /// Stop
    MediaStop = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_STOP,
    /// Eject
    MediaEject = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_EJECT,
    /// Play / Pause
    MediaPlayPause = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_PLAY_PAUSE,
    /// Media Select
    MediaSelect = sys::SDL_Scancode_SDL_SCANCODE_MEDIA_SELECT,
    /// AC New
    AcNew = sys::SDL_Scancode_SDL_SCANCODE_AC_NEW,
    /// AC Open
    AcOpen = sys::SDL_Scancode_SDL_SCANCODE_AC_OPEN,
    /// AC Close
    AcClose = sys::SDL_Scancode_SDL_SCANCODE_AC_CLOSE,
    /// AC Exit
    AcExit = sys::SDL_Scancode_SDL_SCANCODE_AC_EXIT,
    /// AC Save
    AcSave = sys::SDL_Scancode_SDL_SCANCODE_AC_SAVE,
    /// AC Print
    AcPrint = sys::SDL_Scancode_SDL_SCANCODE_AC_PRINT,
    /// AC Properties
    AcProperties = sys::SDL_Scancode_SDL_SCANCODE_AC_PROPERTIES,
    /// AC Search
    AcSearch = sys::SDL_Scancode_SDL_SCANCODE_AC_SEARCH,
    /// AC Home
    AcHome = sys::SDL_Scancode_SDL_SCANCODE_AC_HOME,
    /// AC Back
    AcBack = sys::SDL_Scancode_SDL_SCANCODE_AC_BACK,
    /// AC Forward
    AcForward = sys::SDL_Scancode_SDL_SCANCODE_AC_FORWARD,
    /// AC Stop
    AcStop = sys::SDL_Scancode_SDL_SCANCODE_AC_STOP,
    /// AC Refresh
    AcRefresh = sys::SDL_Scancode_SDL_SCANCODE_AC_REFRESH,
    /// AC Bookmarks
    AcBookmarks = sys::SDL_Scancode_SDL_SCANCODE_AC_BOOKMARKS,
    /// Usually situated below the display on phones and used as a multi-function feature key for selecting
    /// a software defined function shown on the bottom left of the display.
    SoftLeft = sys::SDL_Scancode_SDL_SCANCODE_SOFTLEFT,
    /// Usually situated below the display on phones and used as a multi-function feature key for selecting
    /// a software defined function shown on the bottom right of the display.
    SoftRight = sys::SDL_Scancode_SDL_SCANCODE_SOFTRIGHT,
    /// Used for accepting phone calls.
    Call = sys::SDL_Scancode_SDL_SCANCODE_CALL,
    /// Used for rejecting phone calls.
    EndCall = sys::SDL_Scancode_SDL_SCANCODE_ENDCALL,
    /// 400-500 reserved for dynamic keycodes
    Reserved = sys::SDL_Scancode_SDL_SCANCODE_RESERVED,
    /// Not a key, just marks the number of scancodes for array bounds
    Count = sys::SDL_Scancode_SDL_SCANCODE_COUNT,
}

impl Scancode {
    /// Converts the scancode into an index that can be used to access a key's current state.
    #[inline]
    pub fn as_index(&self) -> usize {
        *self as u32 as usize
    }
}
