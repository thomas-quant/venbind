# Windows Virtual-Key Codes — venbind Reference

This document is the authoritative mapping table for the Windows virtual-key (VK) codes that
`venbind` uses on Windows. The primary source is the official Microsoft Win32 documentation
("Virtual-Key Codes", `Winuser.h`). Every constant name, hex value, and description in the
table below has been verified against that page. The `windows`-crate column confirms compile-time
availability in `windows = "0.61.1"` (the version declared in `Cargo.toml`) with the feature
flag `Win32_UI_Input_KeyboardAndMouse`.

**Module path** (used in `src/windows.rs`):

```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_PRIOR, /* … */};
```

`VIRTUAL_KEY` is a `#[repr(transparent)] pub struct VIRTUAL_KEY(pub u16)` in the `windows`
crate — a newtype over `u16`, not a type alias. (In the lighter `windows-sys` crate it is
`pub type VIRTUAL_KEY = u16`, but venbind uses `windows`, not `windows-sys`.)

---

## Verified Mapping Table

All constants are of type `VIRTUAL_KEY` and live in
`windows::Win32::UI::Input::KeyboardAndMouse`.

### Navigation

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| Page Up | `VK_PRIOR` | `0x21` | 33 | `pageup` | |
| Page Down | `VK_NEXT` | `0x22` | 34 | `pagedown` | |
| Home | `VK_HOME` | `0x24` | 36 | `home` | |
| End | `VK_END` | `0x23` | 35 | `end` | |
| Insert | `VK_INSERT` | `0x2D` | 45 | `insert` | |
| Delete | `VK_DELETE` | `0x2E` | 46 | `delete` | |

### Editing / Whitespace

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| Escape | `VK_ESCAPE` | `0x1B` | 27 | `escape` | |
| Enter / Return | `VK_RETURN` | `0x0D` | 13 | `enter` | Shared with numpad Enter; see §Extended-key caveat |
| Backspace | `VK_BACK` | `0x08` | 8 | `backspace` | |
| Tab | `VK_TAB` | `0x09` | 9 | `tab` | |
| Space | `VK_SPACE` | `0x20` | 32 | `space` | |

### Arrow Keys

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| Left Arrow | `VK_LEFT` | `0x25` | 37 | `left` | |
| Right Arrow | `VK_RIGHT` | `0x27` | 39 | `right` | |
| Up Arrow | `VK_UP` | `0x26` | 38 | `up` | |
| Down Arrow | `VK_DOWN` | `0x28` | 40 | `down` | |

### Function Keys

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| F1 | `VK_F1` | `0x70` | 112 | `f1` | |
| F2 | `VK_F2` | `0x71` | 113 | `f2` | |
| F3 | `VK_F3` | `0x72` | 114 | `f3` | |
| F4 | `VK_F4` | `0x73` | 115 | `f4` | |
| F5 | `VK_F5` | `0x74` | 116 | `f5` | |
| F6 | `VK_F6` | `0x75` | 117 | `f6` | |
| F7 | `VK_F7` | `0x76` | 118 | `f7` | |
| F8 | `VK_F8` | `0x77` | 119 | `f8` | |
| F9 | `VK_F9` | `0x78` | 120 | `f9` | |
| F10 | `VK_F10` | `0x79` | 121 | `f10` | |
| F11 | `VK_F11` | `0x7A` | 122 | `f11` | |
| F12 | `VK_F12` | `0x7B` | 123 | `f12` | |
| F13 | `VK_F13` | `0x7C` | 124 | `f13` | Non-standard; no physical key on most keyboards |
| F14 | `VK_F14` | `0x7D` | 125 | `f14` | Non-standard |
| F15 | `VK_F15` | `0x7E` | 126 | `f15` | Non-standard |
| F16 | `VK_F16` | `0x7F` | 127 | `f16` | Non-standard |
| F17 | `VK_F17` | `0x80` | 128 | `f17` | Non-standard; Windows uses VK_LWIN+VK_F17 internally at shutdown |
| F18 | `VK_F18` | `0x81` | 129 | `f18` | Non-standard |
| F19 | `VK_F19` | `0x82` | 130 | `f19` | Non-standard |
| F20 | `VK_F20` | `0x83` | 131 | `f20` | Non-standard |
| F21 | `VK_F21` | `0x84` | 132 | `f21` | Non-standard |
| F22 | `VK_F22` | `0x85` | 133 | `f22` | Non-standard |
| F23 | `VK_F23` | `0x86` | 134 | `f23` | Non-standard |
| F24 | `VK_F24` | `0x87` | 135 | `f24` | Non-standard |

### Locks and System Keys

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| Caps Lock | `VK_CAPITAL` | `0x14` | 20 | `capslock` | |
| Num Lock | `VK_NUMLOCK` | `0x90` | 144 | `numlock` | |
| Scroll Lock | `VK_SCROLL` | `0x91` | 145 | `scrolllock` | |
| Print Screen | `VK_SNAPSHOT` | `0x2C` | 44 | `printscreen` | May not generate a WM_KEYDOWN in some Windows configurations |
| Pause | `VK_PAUSE` | `0x13` | 19 | `pause` | |
| Application / Menu | `VK_APPS` | `0x5D` | 93 | `menu` | Context-menu key, right of right Meta on full keyboards |
| Cancel | `VK_CANCEL` | `0x03` | 3 | `cancel` | Control-Break processing |
| Clear | `VK_CLEAR` | `0x0C` | 12 | See note | Vendored libuiohook cannot distinguish it from Numpad 5 with NumLock off; production events use `numpad5` |
| Select | `VK_SELECT` | `0x29` | 41 | `select` | |
| Print | `VK_PRINT` | `0x2A` | 42 | `print` | Distinct from Print Screen |
| Execute | `VK_EXECUTE` | `0x2B` | 43 | `execute` | |
| Help | `VK_HELP` | `0x2F` | 47 | `help` | |
| Sleep | `VK_SLEEP` | `0x5F` | 95 | `sleep` | |

### Volume, Media, Browser, and Launch Keys

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token |
|-----|--------------------------|-----------|---------------|-----------------|
| Volume Mute | `VK_VOLUME_MUTE` | `0xAD` | 173 | `volumemute` |
| Volume Down | `VK_VOLUME_DOWN` | `0xAE` | 174 | `volumedown` |
| Volume Up | `VK_VOLUME_UP` | `0xAF` | 175 | `volumeup` |
| Next Track | `VK_MEDIA_NEXT_TRACK` | `0xB0` | 176 | `medianexttrack` |
| Previous Track | `VK_MEDIA_PREV_TRACK` | `0xB1` | 177 | `mediaprevtrack` |
| Stop Media | `VK_MEDIA_STOP` | `0xB2` | 178 | `mediastop` |
| Play/Pause Media | `VK_MEDIA_PLAY_PAUSE` | `0xB3` | 179 | `mediaplaypause` |
| Browser Back | `VK_BROWSER_BACK` | `0xA6` | 166 | `browserback` |
| Browser Forward | `VK_BROWSER_FORWARD` | `0xA7` | 167 | `browserforward` |
| Browser Refresh | `VK_BROWSER_REFRESH` | `0xA8` | 168 | `browserrefresh` |
| Browser Stop | `VK_BROWSER_STOP` | `0xA9` | 169 | `browserstop` |
| Browser Search | `VK_BROWSER_SEARCH` | `0xAA` | 170 | `browsersearch` |
| Browser Favorites | `VK_BROWSER_FAVORITES` | `0xAB` | 171 | `browserfavorites` |
| Browser Home | `VK_BROWSER_HOME` | `0xAC` | 172 | `browserhome` |
| Launch Mail | `VK_LAUNCH_MAIL` | `0xB4` | 180 | `launchmail` |
| Launch Media | `VK_LAUNCH_MEDIA_SELECT` | `0xB5` | 181 | `launchmedia` |
| Launch Application 1 | `VK_LAUNCH_APP1` | `0xB6` | 182 | `launchapp1` |
| Launch Application 2 | `VK_LAUNCH_APP2` | `0xB7` | 183 | `launchapp2` |

### Numpad Keys

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| Numpad 0 | `VK_NUMPAD0` | `0x60` | 96 | `numpad0` | NumLock ON; see §NumLock-off caveat |
| Numpad 1 | `VK_NUMPAD1` | `0x61` | 97 | `numpad1` | NumLock ON |
| Numpad 2 | `VK_NUMPAD2` | `0x62` | 98 | `numpad2` | NumLock ON |
| Numpad 3 | `VK_NUMPAD3` | `0x63` | 99 | `numpad3` | NumLock ON |
| Numpad 4 | `VK_NUMPAD4` | `0x64` | 100 | `numpad4` | NumLock ON |
| Numpad 5 | `VK_NUMPAD5` | `0x65` | 101 | `numpad5` | NumLock ON |
| Numpad 6 | `VK_NUMPAD6` | `0x66` | 102 | `numpad6` | NumLock ON |
| Numpad 7 | `VK_NUMPAD7` | `0x67` | 103 | `numpad7` | NumLock ON |
| Numpad 8 | `VK_NUMPAD8` | `0x68` | 104 | `numpad8` | NumLock ON |
| Numpad 9 | `VK_NUMPAD9` | `0x69` | 105 | `numpad9` | NumLock ON |
| Numpad + | `VK_ADD` | `0x6B` | 107 | `numpadadd` | |
| Numpad - | `VK_SUBTRACT` | `0x6D` | 109 | `numpadsubtract` | |
| Numpad \* | `VK_MULTIPLY` | `0x6A` | 106 | `numpadmultiply` | |
| Numpad / | `VK_DIVIDE` | `0x6F` | 111 | `numpaddivide` | |
| Numpad . | `VK_DECIMAL` | `0x6E` | 110 | `numpaddecimal` | |
| Numpad Separator | `VK_SEPARATOR` | `0x6C` | 108 | `numpadseparator` | Layout/hardware dependent |

### Modifier Keys (no canonical token; handled via event mask)

These keys are intercepted at the `event.mask` level in venbind and are never emitted as a
`key` token. They are listed here for completeness and to document the L/R-variant DOM mismatch
(see §DOM Alignment below).

| Key | `windows`-crate constant | Hex value | DOM `keyCode` | Canonical token | Notes |
|-----|--------------------------|-----------|---------------|-----------------|-------|
| Shift (generic) | `VK_SHIFT` | `0x10` | 16 | — | |
| Ctrl (generic) | `VK_CONTROL` | `0x11` | 17 | — | |
| Alt / Menu (generic) | `VK_MENU` | `0x12` | 18 | — | |
| Left Shift | `VK_LSHIFT` | `0xA0` | 16 ⚠ | — | DOM reports 16 (VK_SHIFT), not 160; see §DOM Alignment |
| Right Shift | `VK_RSHIFT` | `0xA1` | 16 ⚠ | — | DOM reports 16 (VK_SHIFT), not 161 |
| Left Ctrl | `VK_LCONTROL` | `0xA2` | 17 ⚠ | — | DOM reports 17 (VK_CONTROL), not 162 |
| Right Ctrl | `VK_RCONTROL` | `0xA3` | 17 ⚠ | — | DOM reports 17 (VK_CONTROL), not 163 |
| Left Alt | `VK_LMENU` | `0xA4` | 18 ⚠ | — | DOM reports 18 (VK_MENU), not 164 |
| Right Alt | `VK_RMENU` | `0xA5` | 18 ⚠ | — | DOM reports 18 (VK_MENU), not 165 |
| Left Win | `VK_LWIN` | `0x5B` | 91 | — | DOM and VK are identical for Win keys |
| Right Win | `VK_RWIN` | `0x5C` | 92 | — | DOM and VK are identical for Win keys |

---

## DOM `keyCode` Alignment

The legacy `KeyboardEvent.keyCode` values in browsers were historically derived from Windows
virtual-key codes. For every key in this table **except the L/R Shift, Ctrl, and Alt variants**,
`keyCode` equals the decimal interpretation of the VK hex value, confirming a 1:1 alignment.

**Examples of exact alignment:**

- `VK_PRIOR` = `0x21` = 33 decimal → `KeyboardEvent.keyCode` = 33 ✓
- `VK_F1` = `0x70` = 112 decimal → `KeyboardEvent.keyCode` = 112 ✓
- `VK_NUMPAD0` = `0x60` = 96 decimal → `KeyboardEvent.keyCode` = 96 ✓
- `VK_APPS` = `0x5D` = 93 decimal → `KeyboardEvent.keyCode` = 93 ✓

**Keys where DOM `keyCode` does NOT equal the VK hex value (⚠ flagged above):**

| VK constant | VK hex | VK decimal | DOM `keyCode` | Discrepancy |
|-------------|--------|-----------|---------------|-------------|
| `VK_LSHIFT` | `0xA0` | 160 | 16 | DOM collapses L/R to the generic `VK_SHIFT` value |
| `VK_RSHIFT` | `0xA1` | 161 | 16 | Same |
| `VK_LCONTROL` | `0xA2` | 162 | 17 | DOM collapses L/R to the generic `VK_CONTROL` value |
| `VK_RCONTROL` | `0xA3` | 163 | 17 | Same |
| `VK_LMENU` | `0xA4` | 164 | 18 | DOM collapses L/R to the generic `VK_MENU` value |
| `VK_RMENU` | `0xA5` | 165 | 18 | Same |

Web browsers receive WM_KEYDOWN with the generic VK (e.g. `VK_SHIFT` = 0x10) and use `location`
(`KeyboardEvent.location`: 1 = left, 2 = right) to convey handedness, so `keyCode` never
carries the sided value. venbind bypasses the browser layer entirely and reads raw VK codes
from the uiohook hook, so `VK_LSHIFT` (160) and `VK_RSHIFT` (161) are fully distinguishable at
the Win32 level even though DOM `keyCode` does not expose them.

---

## Caveats

### Numpad Enter — no distinct VK code

There is **no separate virtual-key for the numpad Enter key**. Both main-keyboard Enter and
numpad Enter produce `VK_RETURN` (`0x0D`). The two can be distinguished only by the
extended-key bit (bit 24) of the `KBDLLHOOKSTRUCT.flags` field, or equivalently by the
`KF_EXTENDED` flag in the `lParam` of `WM_KEYDOWN`. venbind uses
`uiohook_event.data.keyboard.keycode` alongside the rawcode: `VC_ENTER` maps to `enter` and
`VC_KP_ENTER` maps to `numpadenter`.

### NumLock OFF — numpad keys emit navigation VKs

When NumLock is **off**, the numpad digit keys do not generate `VK_NUMPAD*`. Instead they emit
the same VK codes as the corresponding navigation and arrow keys:

| Numpad key | NumLock ON | NumLock OFF |
|------------|------------|-------------|
| Numpad 0 | `VK_NUMPAD0` (0x60) | `VK_INSERT` (0x2D) |
| Numpad 1 | `VK_NUMPAD1` (0x61) | `VK_END` (0x23) |
| Numpad 2 | `VK_NUMPAD2` (0x62) | `VK_DOWN` (0x28) |
| Numpad 3 | `VK_NUMPAD3` (0x63) | `VK_NEXT` (0x22) |
| Numpad 4 | `VK_NUMPAD4` (0x64) | `VK_LEFT` (0x25) |
| Numpad 5 | `VK_NUMPAD5` (0x65) | `VK_CLEAR` (0x0C) |
| Numpad 6 | `VK_NUMPAD6` (0x66) | `VK_RIGHT` (0x27) |
| Numpad 7 | `VK_NUMPAD7` (0x67) | `VK_HOME` (0x24) |
| Numpad 8 | `VK_NUMPAD8` (0x68) | `VK_UP` (0x26) |
| Numpad 9 | `VK_NUMPAD9` (0x69) | `VK_PRIOR` (0x21) |

venbind uses the normalized libuiohook scancode to retain physical keypad identity. With NumLock
off, pressing numpad-9 still produces `numpad9`, while the dedicated navigation key produces
`pageup`. This behavior is verified through synthetic `_uiohook_event` values created with the
vendored `keycode_to_scancode` implementation and passed through the same state/event processor as
the production hook callback.
The operator keys (`VK_ADD`, `VK_SUBTRACT`, `VK_MULTIPLY`, `VK_DIVIDE`, `VK_DECIMAL`) are
**not** affected by NumLock state — they always produce their own VK codes.

### Extended-key flag

Certain keys that share a VK code with a numpad counterpart are distinguished by the
extended-key flag (`LLKHF_EXTENDED`, bit 0 of `KBDLLHOOKSTRUCT.flags`, equivalent to
`KF_EXTENDED >> 8`). The following keys set the extended-key flag:

- Right-hand Ctrl (`VK_RCONTROL`) and Alt (`VK_RMENU`)
- Numpad `/` (`VK_DIVIDE`) — as opposed to the alphabetic `?` key
- Numpad Enter (`VK_RETURN` with extended flag set)
- Arrow keys, Home, End, Page Up, Page Down, Insert, Delete on the **dedicated** navigation
  cluster (not the numpad equivalents when NumLock is off)

libuiohook normalizes this distinction into `event.data.keyboard.keycode`; venbind consumes that
field for Numpad Enter and NumLock-off keypad navigation.

#### Vendored libuiohook encoding (important naming caveat)

The vendored `keycode_to_scancode(vk_code, flags)` does not copy the hardware scan code from
`KBDLLHOOKSTRUCT`. It starts with `keycode_scancode_table[vk_code][0]` and, for the ten ambiguous
navigation VKs, ORs `0xEE00` into that value when `LLKHF_EXTENDED` is set. Consequently its constant
names look reversed if they are read as physical-key labels:

| Raw VK | `LLKHF_EXTENDED` | libuiohook `keycode` | Physical source | venbind token |
|---|---:|---|---|---|
| `VK_HOME` | 0 | `VC_HOME` (`0x0E47`) | Numpad 7, NumLock off | `numpad7` |
| `VK_HOME` | 1 | `VC_KP_HOME` (`0xEE47`) | Dedicated Home | `home` |
| `VK_RETURN` | 0 | `VC_ENTER` (`0x001C`) | Main Enter | `enter` |
| `VK_RETURN` | 1 | `VC_KP_ENTER` (`0x0E1C`) | Numpad Enter | `numpadenter` |

The same non-extended/extended relationship applies to Insert, Delete, End, Page Up, Page Down,
and all four arrows. venbind intentionally matches the values the vendored function actually emits,
not the apparent `VC_*`/`VC_KP_*` naming.

`VK_CLEAR` is the exception: the vendored helper does not branch on `LLKHF_EXTENDED` and always
returns `VC_CLEAR`. venbind therefore interprets production `VK_CLEAR` events as Numpad 5 with
NumLock off (`numpad5`); the backend cannot reliably expose a separate physical `clear` binding from
the information libuiohook supplies.

### Deliberately unsupported VK groups

The mapping excludes IME/process/packet keys, OEM-specific and reserved ranges, legacy terminal
keys (`VK_ATTN`, `VK_CRSEL`, and related values), gamepad VKs, mouse buttons, and sided modifier
bindings. These values do not receive guessed or localized tokens. Unicode resolution is bounded to
alphanumeric VKs and the common layout-dependent OEM punctuation VKs, so unsupported VK groups are
ignored even if a layout table contains an incidental entry for them.

### `VK_SNAPSHOT` (Print Screen) and `WM_KEYDOWN`

On many Windows configurations, `WM_KEYDOWN` is **not** generated for Print Screen because the
OS intercepts it for screenshots. Low-level keyboard hooks (`WH_KEYBOARD_LL`) do still receive
`VK_SNAPSHOT` events, so uiohook-based interception should work, but this key is not reliable
in all hook contexts.

---

## Sources

- **Microsoft Learn — Virtual-Key Codes (Winuser.h)**
  <https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes>
  Primary source for all VK constant names and hex values (table verified 2026-06-10).

- **microsoft/windows-rs documentation — `windows::Win32::UI::Input::KeyboardAndMouse`**
  <https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Input/KeyboardAndMouse/index.html>
  Official rustdoc for the `windows` crate; confirms `pub const VK_PRIOR: VIRTUAL_KEY` and the
  `#[repr(transparent)] pub struct VIRTUAL_KEY(pub u16)` newtype definition.

- **docs.rs — `windows-sys::Win32::UI::Input::KeyboardAndMouse`**
  <https://docs.rs/windows-sys/latest/windows_sys/Win32/UI/Input/KeyboardAndMouse/>
  Confirms `pub const VK_PRIOR: VIRTUAL_KEY = 33u16;` (in `windows-sys`, `VIRTUAL_KEY` is
  `pub type VIRTUAL_KEY = u16`, i.e. a type alias rather than a newtype).

- **MDN Web Docs — `KeyboardEvent.keyCode`**
  <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode>
  Source for DOM `keyCode` values; used for the alignment check and the L/R-modifier mismatch
  documentation. Note: `keyCode` is deprecated; modern code should use `KeyboardEvent.code`.

- **Microsoft Learn — `KBDLLHOOKSTRUCT` structure**
  <https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct>
  Documents the `flags` field and `LLKHF_EXTENDED` bit used to distinguish numpad Enter from
  main-keyboard Enter.
