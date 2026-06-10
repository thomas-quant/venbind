use std::collections::HashSet;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::{mpsc::Sender, Mutex};
use std::sync::{LazyLock, OnceLock};

use uiohook_sys::{
    _event_type_EVENT_KEY_PRESSED, _event_type_EVENT_KEY_RELEASED, _uiohook_event, hook_run,
    hook_set_dispatch_proc, UIOHOOK_SUCCESS,
};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_ADD, VK_APPS, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DECIMAL,
    VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14,
    VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24, VK_F3,
    VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HOME, VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LMENU,
    VK_LSHIFT, VK_LWIN, VK_MENU, VK_MULTIPLY, VK_NEXT, VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1,
    VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
    VK_PAUSE, VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SCROLL,
    VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_TAB, VK_UP,
};

use crate::errors::{Result, VenbindError};
use crate::structs::{KeybindId, KeybindInfo, KeybindTrigger, Keybinds, Shortcut};

static KEYBINDS: LazyLock<Mutex<Keybinds>> = LazyLock::new(|| Mutex::new(Keybinds::default()));
static CURR_DOWN: LazyLock<Mutex<Shortcut>> = LazyLock::new(|| {
    Mutex::new(Shortcut {
        shift: false,
        alt: false,
        ctrl: false,
        meta: false,
        keys: HashSet::new(),
    })
});
static CURR_ACTIVE_KEYBINDS: LazyLock<Mutex<HashSet<KeybindId>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TX: OnceLock<Sender<KeybindTrigger>> = OnceLock::new();

pub(crate) fn start_keybinds_internal(tx: Sender<KeybindTrigger>, _: Option<String>) -> Result<()> {
    TX.set(tx).unwrap();

    unsafe {
        hook_set_dispatch_proc(Some(dispatch_proc));
        if hook_run() != UIOHOOK_SUCCESS as i32 {
            return Err(VenbindError::LibUIOHookError);
        }
    };
    Ok(())
}

#[no_mangle]
pub extern "C" fn dispatch_proc(event_ref: *mut _uiohook_event) {
    let event = unsafe { *event_ref };
    if event.type_ == _event_type_EVENT_KEY_PRESSED || event.type_ == _event_type_EVENT_KEY_RELEASED
    {
        let keycode = unsafe { event.data.keyboard.rawcode };
        let scancode = unsafe { event.data.keyboard.keycode };
        let vk = VIRTUAL_KEY(keycode);
        let key: Option<String> = match vk {
            // Modifier keys are tracked via `event.mask`, never as a key token.
            VK_SHIFT | VK_MENU | VK_CONTROL | VK_LWIN | VK_RWIN | VK_LSHIFT | VK_RSHIFT
            | VK_RCONTROL | VK_LCONTROL | VK_LMENU | VK_RMENU => None,
            // Numpad Enter shares `VK_RETURN` with the main Enter key; the only
            // distinguisher is libuiohook's scancode field, which carries the
            // extended bit as `VC_KP_ENTER` (0x0E1C) vs `VC_ENTER` (0x001C). Check
            // it first so "numpadenter" registrations match, keeping lock-step with
            // the Linux backend (which emits NUMPAD_ENTER for `KP_Enter`).
            VK_RETURN if u32::from(scancode) == uiohook_sys::VC_KP_ENTER => {
                Some(crate::structs::tokens::NUMPAD_ENTER.to_owned())
            }
            _ => {
                // Named / non-printable keys -> a canonical, locale-independent
                // token (see `crate::structs::tokens`), checked *before* the
                // unicode path so keys that also have an ascii form (space, enter,
                // tab, backspace, escape, delete, numpad) still resolve to their
                // stable token rather than a raw control char.
                if let Some(token) = vk_to_token(vk) {
                    Some(token.to_owned())
                } else {
                    // Printable keys -> their lowercased unicode character.
                    const BUF_SIZE: usize = 8;
                    let mut key_buffer: Vec<uiohook_sys::platform::wchar_t> = vec![0; BUF_SIZE];
                    let str_count = unsafe {
                        uiohook_sys::platform::keycode_to_unicode(
                            keycode as u32,
                            key_buffer.as_mut_ptr(),
                            BUF_SIZE.try_into().unwrap(),
                        )
                    };
                    key_buffer.truncate(str_count.try_into().unwrap());
                    let key = OsString::from_wide(&key_buffer);
                    if !key.is_empty() {
                        Some(key.to_string_lossy().to_lowercase())
                    } else {
                        // Unmapped, non-printable key (media / browser / IME keys,
                        // etc.): ignore it rather than guess a name. The old
                        // `GetKeyNameTextW((scancode as i32) << 16)` fallback fed a
                        // malformed lParam -- libuiohook's scancode carries an
                        // `0xE0` high-byte prefix that lands outside the scancode
                        // (bits 16-23) and extended-flag (bit 24) fields -- so e.g.
                        // Volume Up resolved to "b" and mis-fired any keybind bound
                        // to that letter. `None` is strictly safer: these keys
                        // could not be reliably bound anyway.
                        None
                    }
                }
            }
        };

        let shift = event.mask & uiohook_sys::MASK_SHIFT as u16 != 0;
        let alt = event.mask & uiohook_sys::MASK_ALT as u16 != 0;
        let ctrl = event.mask & uiohook_sys::MASK_CTRL as u16 != 0;
        let meta = event.mask & uiohook_sys::MASK_META as u16 != 0;

        let mut curr_down = CURR_DOWN.lock().unwrap();
        curr_down.alt = alt;
        curr_down.shift = shift;
        curr_down.ctrl = ctrl;
        curr_down.meta = meta;
        if let Some(key) = key {
            if event.type_ == _event_type_EVENT_KEY_PRESSED {
                curr_down.keys.insert(key);
            } else {
                curr_down.keys.remove(&key);
            }
        }
        let keybinds = KEYBINDS.lock().unwrap();
        let active: HashSet<String> = keybinds
            .get_active_keybinds(&curr_down)
            .into_iter()
            .collect();
        let mut curr_active_keybinds = CURR_ACTIVE_KEYBINDS.lock().unwrap();
        let pressed_keybinds = active.difference(&curr_active_keybinds);
        let released_keybinds = curr_active_keybinds.difference(&active);
        // `dispatch_proc` is `extern "C"` and runs on libuiohook's hook thread, so
        // a panic here would unwind across the FFI boundary (undefined behaviour).
        // If the consumer has dropped the receiver the send simply fails -- ignore
        // it rather than `.unwrap()`-ing and aborting the host process.
        if let Some(tx) = TX.get() {
            for pressed in pressed_keybinds {
                let _ = tx.send(KeybindTrigger::Pressed(pressed.clone()));
            }
            for released in released_keybinds {
                let _ = tx.send(KeybindTrigger::Released(released.clone()));
            }
        }
        curr_active_keybinds.clear();
        curr_active_keybinds.extend(active);
    }
}

pub(crate) fn set_keybinds_internal(keybinds: Vec<KeybindInfo>) -> Result<()> {
    let mut keybinds_mutex = KEYBINDS.lock().unwrap();
    keybinds_mutex.clear();
    keybinds.iter().for_each(|x| {
        if x.shortcut.is_some() {
            keybinds_mutex.register_keybind(
                Shortcut::from_string(x.shortcut.clone().unwrap()),
                x.id.clone(),
            )
        }
    });
    Ok(())
}

pub(crate) fn get_current_shortcut_internal() -> Result<String> {
    let down = CURR_DOWN.lock().unwrap();
    Ok(down.to_string())
}

/// Maps a Windows virtual-key code for a named / non-printable key to venbind's
/// canonical, lowercase, locale-independent token (see `crate::structs::tokens`).
/// Returns `None` for printable keys (letters, digits, OEM punctuation), which
/// are matched by their unicode character instead. This is kept in lock-step with
/// `linux.rs::keysym_to_token`, so a consumer can register one string (e.g.
/// "ctrl+pageup", "f5") that matches identically on Windows and Linux/X11.
fn vk_to_token(vk: VIRTUAL_KEY) -> Option<&'static str> {
    use crate::structs::tokens::*;
    Some(match vk {
        VK_PRIOR => PAGE_UP,
        VK_NEXT => PAGE_DOWN,
        VK_HOME => HOME,
        VK_END => END,
        VK_INSERT => INSERT,
        VK_DELETE => DELETE,
        VK_ESCAPE => ESCAPE,
        VK_RETURN => ENTER,
        VK_BACK => BACKSPACE,
        VK_TAB => TAB,
        VK_SPACE => SPACE,
        VK_UP => UP,
        VK_DOWN => DOWN,
        VK_LEFT => LEFT,
        VK_RIGHT => RIGHT,
        VK_CAPITAL => CAPS_LOCK,
        VK_NUMLOCK => NUM_LOCK,
        VK_SCROLL => SCROLL_LOCK,
        VK_SNAPSHOT => PRINT_SCREEN,
        VK_PAUSE => PAUSE,
        VK_APPS => MENU,
        VK_F1 => F1,
        VK_F2 => F2,
        VK_F3 => F3,
        VK_F4 => F4,
        VK_F5 => F5,
        VK_F6 => F6,
        VK_F7 => F7,
        VK_F8 => F8,
        VK_F9 => F9,
        VK_F10 => F10,
        VK_F11 => F11,
        VK_F12 => F12,
        VK_F13 => F13,
        VK_F14 => F14,
        VK_F15 => F15,
        VK_F16 => F16,
        VK_F17 => F17,
        VK_F18 => F18,
        VK_F19 => F19,
        VK_F20 => F20,
        VK_F21 => F21,
        VK_F22 => F22,
        VK_F23 => F23,
        VK_F24 => F24,
        VK_NUMPAD0 => NUMPAD0,
        VK_NUMPAD1 => NUMPAD1,
        VK_NUMPAD2 => NUMPAD2,
        VK_NUMPAD3 => NUMPAD3,
        VK_NUMPAD4 => NUMPAD4,
        VK_NUMPAD5 => NUMPAD5,
        VK_NUMPAD6 => NUMPAD6,
        VK_NUMPAD7 => NUMPAD7,
        VK_NUMPAD8 => NUMPAD8,
        VK_NUMPAD9 => NUMPAD9,
        VK_ADD => NUMPAD_ADD,
        VK_SUBTRACT => NUMPAD_SUBTRACT,
        VK_MULTIPLY => NUMPAD_MULTIPLY,
        VK_DIVIDE => NUMPAD_DIVIDE,
        VK_DECIMAL => NUMPAD_DECIMAL,
        // Numpad Enter also reports VK_RETURN here; the caller splits it from the
        // main Enter via the scancode (VC_KP_ENTER) before reaching this map, so
        // VK_RETURN resolves to ENTER.
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::tokens;

    #[test]
    fn named_keys_map_to_canonical_tokens() {
        assert_eq!(vk_to_token(VK_PRIOR), Some(tokens::PAGE_UP));
        assert_eq!(vk_to_token(VK_NEXT), Some(tokens::PAGE_DOWN));
        assert_eq!(vk_to_token(VK_SPACE), Some(tokens::SPACE));
        assert_eq!(vk_to_token(VK_RETURN), Some(tokens::ENTER));
        assert_eq!(vk_to_token(VK_F5), Some(tokens::F5));
        assert_eq!(vk_to_token(VK_NUMPAD7), Some(tokens::NUMPAD7));
    }
}
