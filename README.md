# Lupa

A minimalist launcher built with gtk4-layer-shell and rust.

> [!NOTE]
> On some Wayland compositors, the clipboard may not persist after the app closes.
> You may want to try [wl-clip-persist](https://github.com/Linus789/wl-clip-persist)
> if this is the case.

## Usage
TODO

## Core Features
- Search and launch apps
- Search and open files
- Evaluate mathematical expressions, making use of the [evalexpr crate](https://https://crates.io/crates/evalexpr)

## Credit
Some implementation details are heavily based in
[lucien](https://github.com/Wachamuli/lucien).

## TO-DO
- [ ] Stateful search icon, morphs depending on prefix
- [ ] More providers
    - [ ] System actions (Shutdown, reboot, etc.)
- [ ] Usage documentation
