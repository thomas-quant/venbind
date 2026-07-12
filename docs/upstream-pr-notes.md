# Future upstream PR: Windows keyboard event reliability

> Working note for a future task. Do **not** submit the current fork branch wholesale. Prepare a
> clean branch from the then-current upstream default branch, reconstruct the relevant changes, and
> write an upstream-focused PR description.

## Current validated state

- Fork branch: `fix/windows-non-unicode-actions`
- Validated head: `bc163a0` (`fix(keybinds): preserve printable character bindings`)
- Green Windows/Linux run: <https://github.com/thomas-quant/venbind/actions/runs/29184362906>
- Windows physical test: 96 bindings passed with matched press/release events.
- Physically verified printable coverage: A-Z, 0-9, and
  `[]#;',./?><:@~}{!"£$%^&*()_+`.
- Physically verified named coverage on the available keyboard: modifiers, navigation, arrows,
  editing/whitespace keys, and F1-F12.
- Both Windows and Linux release `index.node` files loaded successfully and exposed
  `defineErrorHandle`, `getCurrentShortcut`, `setKeybinds`, and `startKeybinds`.

The first physical-test conclusion was invalid because the initial manual harness registered only
named tokens. The corrected harness registered printable characters and was rerun with the
`bc163a0` artifact. Do not cite the first run as printable-key evidence.

## Problem in the old implementation

Upstream `6afcf28` handled only a small set of named keys explicitly. Everything else tried
`keycode_to_unicode`, then fell back to:

```rust
GetKeyNameTextW((scancode as i32) << 16, &mut buf)
```

That design had several independent failure modes.

### `GetKeyNameTextW` received a malformed identifier

`GetKeyNameTextW` expects a Windows keystroke `lParam`: scan code in bits 16-23 and the extended-key
flag in bit 24. libuiohook's `event.data.keyboard.keycode` is a normalized libuiohook constant, not a
ready-to-shift `lParam` scan-code byte. It may already contain `0x0E`, `0xE0`, or `0xEE` high-byte
markers. Shifting the whole value placed those markers in the wrong fields and discarded the actual
extended-key meaning. An unmapped special key could consequently acquire an unrelated printable
name; Volume Up was observed resolving as `"b"` and could fire a binding for that letter.

### Key names were unsuitable matching tokens

`GetKeyNameTextW` returns display names from the active keyboard layout. They are localized,
capitalized, and may contain spaces (for example, `"Page Up"` or a localized equivalent). Registered
shortcuts are lowercased canonical strings such as `pageup`. Display names therefore could not be a
stable, locale-independent matching namespace.

### Too many keys entered the unsafe fallback

Only Escape, Backspace, Tab, Delete, Enter, and Space were treated as named keys. Navigation,
arrows, locks, F keys, keypad, volume/media/browser, and launch keys passed through Unicode lookup
and then the malformed name fallback. Unsupported VK groups were also guessed rather than ignored.

### Press and release were resolved independently

The old callback called the resolver on both press and release. libuiohook's Windows
`keycode_to_unicode` walks the keyboard-layout DLL tables and mutates its own static dead-key state.
Resolving again on repeat/release could mutate that state unnecessarily and could produce a
different token from the one inserted on press, leaving a key stuck in current state.

### Shared VK values were not interpreted correctly

- Main Enter and Numpad Enter both use `VK_RETURN`; the libuiohook keycode distinguishes
  `VC_ENTER` from `VC_KP_ENTER`.
- With NumLock off, keypad digits use navigation VKs. The extended flag distinguishes those keypad
  events from the dedicated navigation cluster.
- The vendored `keycode_to_scancode` names are counterintuitive in this path: non-extended
  `VK_HOME` produces `VC_HOME` (NumLock-off Numpad 7), while extended `VK_HOME` becomes
  `VC_KP_HOME` (dedicated Home). The implementation must follow actual emitted values, not infer
  semantics from the constant names.

### The callback was difficult and unsafe to test

State was spread across three global mutexes and all interpretation happened inside the C callback.
Tests could exercise mapping helpers but not the production state/transition path with real
`_uiohook_event` unions. The callback also dereferenced without a null guard, used `unwrap()` while
sending, and could panic across an `extern "C"` boundary.

### Printable `+` was not representable

`Shortcut::from_string` splits on `+`. A standalone `"+"` became empty components and could never
match a printable plus token. The corrected grammar uses `"+"` for standalone plus and a doubled
trailing separator with modifiers, such as `"ctrl++"`.

## Implemented design

### Canonical named-key tokens

- A bounded Windows `VK_*` to lowercase token table handles supported non-printable keys.
- Shared constants in `src/structs.rs` define the token contract.
- Linux/X11 mappings use the same tokens, including XF86 media/browser/launch parity.
- Printable alphanumeric and common layout-dependent OEM VKs continue through libuiohook's Unicode
  resolver and are lowercased.
- Unsupported IME/process/packet, reserved/OEM-specific, terminal, gamepad, and mouse ranges are
  ignored instead of guessed.

### Deterministic production event processor

`WindowsEventState` owns current keys/modifiers, active bindings, and press-time token cache.
`process_keyboard_event` receives a real `_uiohook_event`, updates that state, and returns sorted
press/release transitions. The actual libuiohook callback calls this same function.

The resolver is injectable solely so deterministic tests can cover layout-dependent printable
tokens. Named-key tests call the production resolver directly.

### Press-time token cache

The cache is keyed by `(raw VK, normalized libuiohook keycode)`. The first press resolves once,
auto-repeat reuses the cached value, and release removes and uses exactly that press-time token.

### FFI boundary hardening

- Null event pointers return immediately.
- Poisoned locks drop the event instead of panicking.
- Sender failure is ignored when the embedding receiver has gone away.
- `catch_unwind` prevents a Rust panic crossing the C callback boundary.

### Test coverage

Windows tests construct actual `_uiohook_event` keyboard unions and call the production event
processor. They also call the vendored C `keycode_to_scancode` helper to validate the exact values
used for Enter and ambiguous navigation/keypad events.

Coverage includes:

- Navigation, editing, whitespace, arrows, and F1-F24.
- Locks, keypad, common system keys, volume/media/browser/launch keys.
- Main Enter vs. Numpad Enter.
- Dedicated navigation vs. NumLock-off keypad.
- Both sides of modifier masks while retaining unsided public bindings.
- Auto-repeat and exact press-time-token release.
- Unknown and explicitly unsupported VK values.
- Printable AZERTY/Latin letters, digits, punctuation, accented characters, and literal plus through
  the production event processor with an injected deterministic resolver.

The workflow builds natively on Windows and Linux, then loads `index.node` with Node and verifies
the expected exports. It never starts a global hook on a hosted runner.

## Known limitations and intentional non-goals

- Modifier bindings remain unsided: left/right Shift, Ctrl, Alt, and Meta map to the same modifier
  booleans. Physical testing confirmed this behavior.
- Vendored libuiohook always returns `VC_CLEAR` for `VK_CLEAR`, regardless of the extended flag.
  Production therefore treats that event as NumLock-off Numpad 5 and cannot reliably expose a
  separate physical `clear` binding.
- The available physical keyboard lacked many keypad, media/browser/launch, and F13-F24 keys. Those
  paths are covered synthetically through the production processor but were not all physically
  exercised.
- IME/process/packet keys, OEM-specific/reserved ranges, legacy terminal keys, gamepad VKs, mouse
  input, and GoofCord conversion remain out of scope.
- libuiohook still owns mutable dead-key state internally. Resolving only on the first press limits
  venbind's interaction with it but does not redesign libuiohook.
- Do not mix unrelated lifecycle/API cleanup (for example, repeated `start_keybinds` handling) into
  this PR unless upstream explicitly requests it.

## Commit/hunk provenance

The current branch is based on upstream `6afcf28` but contains fork-only CI and binary history. Use
these commits as references, not as a cherry-pick list:

- `b8d9c82`: initial canonical named-token mappings and cross-platform parity.
- `07006cf`: Windows token edge-case corrections.
- `fab3d35`: correction explaining libuiohook's own Unicode/dead-key implementation.
- `b3505f5`: expanded bounded VK table, NumLock/scancode handling, caching, Linux XF86 parity, and
  additional tests/docs.
- `81920ef`: extracted production event processor, synthetic `_uiohook_event` coverage, FFI
  hardening, and native-addon smoke-load step.
- `3281d65`: Windows bindgen type correction; fold this into the preceding implementation commit.
- `bc163a0`: printable regression coverage and literal-plus parsing.

Do not carry these fork-only changes into upstream merely because they are ancestors:

- `f8d7805`: committed prebuilt binaries for fork/GoofCord consumption.
- `6e145dd` and `dc855f0`: fork-specific workflow repair/replacement.
- The untracked `repomix-output.xml`, downloaded Actions artifacts, manual test scripts, and physical
  result files.

`5d02a49` combines useful research documentation with a fork build workflow. Extract/rewrite only
the documentation and any CI concepts upstream actually wants.

## Clean upstream PR checklist

1. Fetch upstream and create a new branch from its then-current default branch. Re-audit upstream in
   case keyboard code, dependencies, or workflows have changed.
2. Reconstruct a minimal diff by intent. Do not open a PR directly from
   `fix/windows-non-unicode-actions` and do not blindly cherry-pick its mixed history.
3. Keep generated binaries, fork-only prebuild paths, `repomix-output.xml`, the manual tester, and
   physical result logs out of the PR.
4. Decide whether the cross-platform literal-plus grammar fix belongs in the main PR or a small
   prerequisite PR. Preserve its regression test either way.
5. Keep Linux token parity with the Windows public token contract; explain why Linux changes are
   relevant to a Windows bug fix.
6. Adapt CI to upstream's existing workflows. Required gates are formatting, tests, native release
   builds on Windows and Linux, and post-build `index.node` export smoke loads. Avoid any interactive
   hook test.
7. Reduce/reorganize the long research notes if upstream prefers concise maintainer documentation,
   but retain the libuiohook `keycode_to_scancode`, `GetKeyNameTextW`, NumLock, and dead-key facts
   needed to justify the design.
8. Re-run deterministic CI on the reconstructed branch and repeat representative physical Windows
   validation from its artifact, including printable characters and literal plus.
9. Produce a clean commit series (or a single squash commit, per upstream preference). Fold
   `3281d65` into the event-processor change rather than preserving a compile-fix commit.
10. Write the PR around the user-visible bug and root causes, not the fork history. Include:
    - Minimal reproduction examples (`pageup`, Volume Up misidentifying as a letter, Enter vs.
      Numpad Enter, NumLock-off keypad, printable `+`).
    - Why localized display names cannot be matching identifiers.
    - Why the static table is deliberately bounded.
    - Why state/token resolution was extracted from the callback.
    - Automated test matrix and physical-test evidence.
    - Explicit limitations/non-goals from the section above.
11. Before submission, inspect the full `upstream/main...HEAD` diff and PR file list for fork-only or
    unrelated content. Do not merge or submit until the user explicitly asks.

## Suggested PR title and structure

Possible title:

> Fix Windows named-key matching through the production uiohook event path

Suggested body sections:

1. Problem and user-visible failures.
2. Root cause (`GetKeyNameTextW` misuse, unstable display names, repeated resolution).
3. Design (canonical table, shared tokens, event state, press-time cache).
4. Numpad/extended-key details from vendored libuiohook.
5. Tests and physical validation.
6. Compatibility, limitations, and non-goals.
