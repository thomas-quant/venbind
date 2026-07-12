# Windows Key Input APIs — Reference for venbind

> Authoritative reference for the Win32 keyboard APIs used (or formerly misused) by venbind's Windows
> backend (`src/windows.rs`). Primary source: Microsoft Learn (learn.microsoft.com). Inline citations
> use bracketed shorthand resolved in the [Sources](#sources) section.

---

## How venbind Uses These APIs

venbind's Windows backend processes libuiohook low-level keyboard hook events. Each event carries two
key identifiers:

- **`rawcode`** — the Windows virtual-key code (VK_* constant).
- **`keycode`** — libuiohook's own scan-code-derived constant.

When a key event arrives the backend must convert one of those values into a stable token string that
can be matched against the user's keybind registration (e.g. `"pageup"`, `"ctrl"`, `"f5"`). Two
translation paths exist in the code:

1. **Static VK-to-token mapping** — used for named / non-printable keys. The event `rawcode` is a
   Windows virtual-key code, and `keycode` carries libuiohook's normalized scancode for ambiguous
   cases such as Numpad Enter and NumLock-off keypad navigation.
2. **`keycode_to_unicode(virtual_key, buf, len)`** — used only for printable keys. This libuiohook
   helper walks the active layout DLL's `VK_TO_WCHARS` tables and maintains its own dead-key state;
   it does not call `ToUnicode` / `ToUnicodeEx`.

Understanding why the former `GetKeyNameTextW` path was wrong for token matching requires a precise
understanding of the APIs and the underlying scan-code / virtual-key model.

---

## 1. Key Identification Concepts

### 1.1 Scan Codes

A *scan code* is a device-dependent value assigned to each physical key by the keyboard hardware
[ABOUT]. Modern USB keyboards report HID Usage IDs; the Windows keyboard driver converts those to
PS/2 Set-1 (make) scan codes before placing them in keystroke messages [ABOUT].

**The scan code identifies the physical position of the key, not its logical meaning.** The same
physical key position produces the same scan code regardless of the active keyboard layout or locale.
This makes scan codes useful for layout-invariant matching (e.g. WASD for games) but unsuitable for
character-valued matching [ABOUT].

Within a WM\_KEYDOWN `lParam`, the scan code occupies **bits 16–23** (the low byte of the high word).
Extended keys (see §1.3) prefix their scan code with `0xE0`; the presence of that prefix is
signalled by bit 24 rather than being stored in bits 16–23 directly [GKNT, ABOUT].

### 1.2 Virtual-Key Codes

A *virtual-key code* (VK\_\* constant) is a **device-independent** value defined by Windows that
identifies the purpose or label of a key [ABOUT]. The keyboard layout driver maps each scan code to a
virtual-key code. Virtual-key codes are stable across keyboard hardware but are **not** stable across
keyboard layouts for printable (character) keys: the same physical key can produce different character
output on a US-QWERTY layout vs. a French-AZERTY layout.

For non-printable keys (function keys, navigation cluster, modifiers, numpad arithmetic operators),
the virtual-key code is the canonical identifier and does not vary with layout. For character keys
(A–Z, 0–9, punctuation), the virtual-key code identifies the key slot but the resulting character
depends on the active layout and the modifier state (Shift, Caps Lock, AltGr).

Selected VK codes relevant to keybind matching [VKC]:

| Key | VK constant | Hex |
|---|---|---|
| Page Up | `VK_PRIOR` | 0x21 |
| Page Down | `VK_NEXT` | 0x22 |
| End | `VK_END` | 0x23 |
| Home | `VK_HOME` | 0x24 |
| Left Arrow | `VK_LEFT` | 0x25 |
| Up Arrow | `VK_UP` | 0x26 |
| Right Arrow | `VK_RIGHT` | 0x27 |
| Down Arrow | `VK_DOWN` | 0x28 |
| Insert | `VK_INSERT` | 0x2D |
| Delete | `VK_DELETE` | 0x2E |
| Numpad 0–9 | `VK_NUMPAD0`–`VK_NUMPAD9` | 0x60–0x69 |
| F1–F12 | `VK_F1`–`VK_F12` | 0x70–0x7B |
| F13–F24 | `VK_F13`–`VK_F24` | 0x7C–0x87 |
| Num Lock | `VK_NUMLOCK` | 0x90 |
| Left Ctrl | `VK_LCONTROL` | 0xA2 |
| Right Ctrl | `VK_RCONTROL` | 0xA3 |
| Left Alt | `VK_LMENU` | 0xA4 |
| Right Alt | `VK_RMENU` | 0xA5 |

### 1.3 Extended-Key Flag (bit 24 of lParam)

The *extended-key flag* in a keystroke message's `lParam` (bit 24, exposed as `KF_EXTENDED = 0x0100`
in the high word) signals that the key's scan code is prefixed by `0xE0` [ABOUT, GKNT].

**Keys that set the extended-key flag** [ABOUT]:

- Right-hand Alt (`VK_RMENU`) and Ctrl (`VK_RCONTROL`).
- The gray navigation cluster: Insert, Delete, Home, End, Page Up, Page Down, and all four Arrow keys.
- Num Lock.
- The Break (Ctrl+Pause) key.
- Print Screen.
- Numpad Divide (`/`) and Numpad Enter.

**Keys that do NOT set the extended-key flag** (even though they may look similar):

- Right Shift — it uses a separate scan code (`0x0036`) with no `0xE0` prefix [ABOUT].
- Numpad 0–9 and Numpad Decimal — they use unextended scan codes. When NumLock is *off* these keys
  generate the same virtual-key codes as their gray-cluster counterparts (`VK_INSERT`, `VK_HOME`,
  `VK_PRIOR`, etc.) but *without* the extended-key flag; this is how an application can distinguish
  "numpad Insert (NumLock off)" from "gray Insert" [ABOUT].

**Practical consequence for venbind:** when building an `lParam` for `GetKeyNameTextW` from a bare
scan code (e.g. `lParam = (scancode as i32) << 16`), bit 24 is zero, meaning the function receives
no extended-key signal. For gray-cluster keys (whose scan code already carries the `0xE0` prefix in
the hardware stream but is stripped to bits 16–23 in the message), this can produce incorrect or
ambiguous name lookups from the keyboard layout's name table.

### 1.4 Bit 25 of lParam — "Do Not Care"

Bit 25 (`KF_REPEAT = 0x4000` in the high word is actually the previous-key-state flag; bit 25 in the
raw `lParam` is a separate flag) instructs `GetKeyNameTextW` not to distinguish left vs. right
instances of modifier keys such as Ctrl and Shift. When set, the function returns the same name for
`VK_LSHIFT` and `VK_RSHIFT`, for example [GKNT].

---

## 2. `GetKeyNameTextW`

```c
int GetKeyNameTextW(
    LONG   lParam,    // keystroke lParam (or synthetic equivalent)
    LPWSTR lpString,  // output buffer
    int    cchSize    // buffer size in chars, including null terminator
);
// Returns: character count written (excl. null), or 0 on failure.
```
[GKNT]

### What it does

`GetKeyNameTextW` consults the **currently active keyboard layout's name table** and returns a
human-readable Unicode string for the key described by `lParam`. The function interprets:

- **Bits 16–23**: OEM scan code.
- **Bit 24**: Extended-key flag (1 = extended, i.e. scan code has `0xE0` prefix).
- **Bit 25**: "Do not care" flag — suppresses left/right modifier distinction.

### Locale and layout dependence

> "The format of the key-name string depends on the current keyboard layout."
> "The key name is translated according to the **currently active keyboard layout**, therefore the
> function might return different results for different keyboard layouts." [GKNT]

Concrete implications:

- On a French keyboard layout, the key at VK\_PRIOR position may be named "Page préc." or a similar
  locale-specific string rather than "Page Up".
- On a German layout, the Esc key may be returned as "Esc" or "Entf", depending on the layout.
- For alphabetic keys (VK\_A–VK\_Z), the docs note they are always returned as uppercase Latin
  A–Z (`U+0041`–`U+005A`) regardless of the current layout — but multi-character key names (e.g.
  "Page Up", "Num Lock", "Scroll Lock") are translated. [GKNT]
- Dead keys are "spelled out in full" (e.g. the key producing the acute accent diacritic is returned
  as the spelled-out word "Acute" or its locale equivalent) [GKNT].

### The capitalization issue

The returned string uses the **keyboard layout's capitalization convention**. Windows ships names
like `"Page Up"`, `"Num Lock"`, `"Scroll Lock"`, `"Escape"`, `"Backspace"` — mixed case, often with
spaces — not lowercase tokens. A comparison against lowercase keybind strings like `"pageup"`,
`"numlock"`, `"escape"` will always fail without explicit case-folding, and even with case-folding
will fail on locale variants.

---

## 3. `ToUnicode` and `ToUnicodeEx`

```c
// Layout-implicit version (uses the calling thread's active layout)
int ToUnicode(
    UINT       wVirtKey,     // virtual-key code
    UINT       wScanCode,    // hardware scan code; high-order bit set if key is up
    const BYTE *lpKeyState,  // 256-byte keyboard state array
    LPWSTR     pwszBuff,     // output: UTF-16 code units (NOT guaranteed null-terminated)
    int        cchBuff,      // buffer size in chars
    UINT       wFlags        // bit 0: menu active; bit 2: don't mutate kernel state (Win10 1607+)
);

// Layout-explicit version
int ToUnicodeEx(
    UINT       wVirtKey,
    UINT       wScanCode,
    const BYTE *lpKeyState,
    LPWSTR     pwszBuff,
    int        cchBuff,
    UINT       wFlags,
    HKL        dwhkl          // keyboard layout handle from LoadKeyboardLayout/GetKeyboardLayout
);
```
[TU, TUX]

### Return value semantics

| Return value | Meaning |
|---|---|
| < 0 | Dead key — a spacing version of the dead-key character was written to the buffer (e.g. `U+00B4 ACUTE ACCENT` rather than the combining form). |
| 0 | No translation for the current keyboard state; buffer unchanged. |
| > 0 | That many UTF-16 code units were written. The buffer may contain more bytes than the return value; ignore the extras. |

[TU, TUX]

### Keyboard-layout dependence

Both functions perform translation based on the active (or specified) keyboard layout [TUX]:
> "The input locale identifier is a broader concept than a keyboard layout, since it can also
> encompass a speech-to-text converter, an Input Method Editor (IME), or any other form of input."

The character produced for a given virtual-key code varies with the active layout, the Shift and
Caps Lock states in `lpKeyState`, and any pending dead-key state.

### Dead-key state mutation

> "As `ToUnicodeEx` translates the virtual-key code, it also changes the state of the kernel-mode
> keyboard buffer. This state-change affects dead keys, ligatures, Alt+Numeric keypad key entry,
> and so on. It might also cause undesired side-effects if used in conjunction with
> `TranslateMessage`." [TUX]

Setting `wFlags` bit 2 (available from Windows 10 version 1607) prevents this state mutation,
making the call safe to use from a low-level keyboard hook without corrupting the user's IME/dead-key
state.

> **Note — does not apply to libuiohook's `keycode_to_unicode`.** libuiohook does **not** call
> `ToUnicode`/`ToUnicodeEx`; its `keycode_to_unicode` (`vendor/src/windows/input_helper.c`) walks the
> active layout DLL's `VK_TO_WCHARS` tables directly and keeps its **own** `static WCHAR deadChar`.
> The `wFlags` bit-2 mitigation is a parameter of the Win32 `ToUnicode` API and has no effect on that
> path. It would only be relevant if venbind called `ToUnicode`/`ToUnicodeEx` itself. The dead-key
> state libuiohook keeps is still mutated on every call (including the extra calls venbind makes from
> `dispatch_proc`) — that is a separate concern, not something bit 2 can fix.

### Surrogate pairs and ligatures

Some layouts may produce multiple UTF-16 code units or surrogate pairs from a single key press [TU].
The buffer must be sized to accommodate at least two `WCHAR`s, and callers must inspect the return
count rather than assuming one code unit per key.

---

## 4. `MapVirtualKeyW`

```c
UINT MapVirtualKeyW(
    UINT uCode,     // input: VK code or scan code depending on uMapType
    UINT uMapType   // translation direction
);
// Returns: translated value, or 0 if no translation exists.
```
[MVK]

### Translation modes

| `uMapType` constant | Direction | Notes |
|---|---|---|
| `MAPVK_VK_TO_VSC` (0) | VK → scan code | Returns left-hand scan code for ambiguous VKs; 0 if none. |
| `MAPVK_VSC_TO_VK` (1) | scan code → VK | Does not distinguish left/right modifiers. On Vista+, high byte of `uCode` may be `0xE0`/`0xE1`. |
| `MAPVK_VK_TO_CHAR` (2) | VK → unshifted character | Returns character in low word; top bit set for dead keys. A–Z always returned uppercase regardless of layout. Use `ToUnicode` for accurate char lookup. |
| `MAPVK_VSC_TO_VK_EX` (3) | scan code → VK (left/right distinguished) | Returns `VK_LCONTROL`/`VK_RCONTROL` etc. |
| `MAPVK_VK_TO_VSC_EX` (4) | VK → extended scan code | High byte contains `0xE0`/`0xE1` for extended keys. Vista+. |

[MVK]

### A–Z capitalization caveat

> "In `MAPVK_VK_TO_CHAR` mode, the 'A'..'Z' keys are translated to upper-case 'A'..'Z' characters
> regardless of current keyboard layout. If you want to translate a virtual-key code to the
> corresponding character, use the `ToUnicode` function." [MVK]

This mirrors `GetKeyNameTextW`'s behavior: both systematically capitalize alphabetic results,
confirming that neither is suitable as a raw character-to-token converter.

---

## 5. NumLock Interaction with Numpad Keys

When **NumLock is on**, numpad digit keys generate `VK_NUMPAD0`–`VK_NUMPAD9` (0x60–0x69). When
**NumLock is off**, the same physical keys generate the *gray-cluster* virtual-key codes
(`VK_INSERT`, `VK_END`, `VK_DOWN`, `VK_PRIOR`, etc.) — identical VK codes to the dedicated gray
navigation keys — but without the extended-key flag [ABOUT].

The extended-key flag is the only reliable way to distinguish "gray Home" from "numpad 7 with NumLock
off" when both produce `VK_HOME` (0x24). The `ToUnicode` docs confirm that the toggle state of
Num Lock is *ignored* when translating with `ToUnicode`/`ToUnicodeEx`; only Caps Lock toggle state
is considered [TU].

`GetKeyNameTextW` uses the scan code (bits 16–23) and the extended-key flag (bit 24). If the
caller constructs `lParam = (scancode << 16)` without setting bit 24 for gray-cluster keys, the
function may look up the numpad key name ("Num 7") instead of the gray key name ("Home"), or
vice versa, depending on the OEM scan-code value and layout table.

---

## 6. Why `GetKeyNameTextW` Is Wrong for Key Matching

Combining the evidence above, `GetKeyNameTextW` fails as a stable key-matching identifier for three
independent reasons:

**Reason 1 — Locale/layout dependence.**
The returned string is looked up from the currently active keyboard layout's name table. A user
running a French, German, Russian, or Japanese keyboard layout will see different strings for
non-alphabetic keys. A keybind registered as `"pageup"` will never match `"Page préc."` on a French
layout, even after case-folding. The docs are explicit: "the function might return different results
for different keyboard layouts." [GKNT]

**Reason 2 — Capitalization and embedded spaces.**
The function returns human-readable names with mixed case and spaces (`"Page Up"`, `"Num Lock"`,
`"Scroll Lock"`, `"Caps Lock"`, `"Back Space"`). venbind's registrations are lowercase tokens
(`"pageup"`, `"numlock"`). Even on a US English layout, simple string equality always fails; a
case-fold + space-strip is required, and that heuristic is fragile across locales.

**Reason 3 — No cross-platform compatibility with Linux/X11 keysym names.**
X11 keysym names (`XK_Prior`, `XK_Next`, `XK_F1`, etc.) have a well-defined, locale-invariant
namespace. The strings returned by `GetKeyNameTextW` have no defined relationship to those names.
A keybind that works on Linux using the keysym-derived token `"prior"` would require a separate
hand-maintained translation table to work on Windows if `GetKeyNameTextW` strings are used as the
canonical identifier.

**Correct approach:** map `rawcode` (the Windows VK code) directly to a canonical lowercase token
using a static lookup table (`VK_PRIOR` → `"pageup"`, `VK_F1` → `"f1"`, etc.). Virtual-key codes
are defined constants in `winuser.h`; they are stable across locales, keyboard layouts, and Windows
versions since Windows 2000. This is the approach adopted in the fix.

---

## 7. Implications for venbind

- **Named-key path:** uses a VK-code-to-token table for supported non-printable keys. The `rawcode`
  field carries the VK code; the normalized libuiohook scancode distinguishes keypad and extended
  variants where Windows reuses a VK code.
- **Printable key path (libuiohook's `keycode_to_unicode`):** produces the correct Unicode
  character for the physical key under the current layout, which is the right behavior for
  character-key matching. Note this helper does **not** call `ToUnicode`/`ToUnicodeEx` — it reads the
  layout DLL's `VK_TO_WCHARS` tables directly and keeps its own `static deadChar`. The `ToUnicode`
  `wFlags` bit-2 (no-state-mutation) flag therefore does **not** apply here; it would only matter if
  venbind called `ToUnicode` itself. venbind resolves a printable token only on the first press,
  caches it through auto-repeat, and reuses that exact token on release instead of mutating the
  helper's dead-key state a second time.
- **Extended-key distinction:** venbind uses libuiohook's normalized scancode alongside the VK code
  to keep keypad tokens stable across NumLock state and distinguish Numpad Enter from Enter.
- **Right-hand modifiers:** `VK_CONTROL` (0x11) and `VK_MENU` (0x12) are the generic (non-sided)
  codes. To distinguish left vs. right Ctrl/Alt, use `MapVirtualKeyW(scanCode, MAPVK_VSC_TO_VK_EX)`
  which returns `VK_LCONTROL`/`VK_RCONTROL` and `VK_LMENU`/`VK_RMENU` [ABOUT, MVK].

### Supported boundary

The static mapping includes navigation, editing and whitespace keys, arrows, F1–F24, locks, keypad
keys, common system keys, volume and media transport, browser controls, and application launch keys.
It intentionally excludes IME/process/packet keys, OEM-specific and reserved VK ranges, legacy
terminal keys, gamepad VKs, mouse buttons, and sided modifier bindings. An excluded key is ignored
when the active keyboard layout does not give it printable Unicode output.

---

## Sources

| Shorthand | Title | URL |
|---|---|---|
| [GKNT] | `GetKeyNameTextW` function (winuser.h) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeynametextw |
| [TU] | `ToUnicode` function (winuser.h) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicode |
| [TUX] | `ToUnicodeEx` function (winuser.h) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex |
| [MVK] | `MapVirtualKeyW` function (winuser.h) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-mapvirtualkeyw |
| [VKC] | Virtual-Key Codes (Winuser.h) | https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes |
| [ABOUT] | Keyboard Input Overview (About Keyboard Input) | https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input |
