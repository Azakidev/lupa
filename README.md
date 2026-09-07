# Lupa

A minimalist launcher built with gtk4-layer-shell and rust.

> [!NOTE]
> On some Wayland compositors, the clipboard may not persist after the app closes.
> You may want to try [wl-clip-persist](https://github.com/Linus789/wl-clip-persist)
> if this is the case.

## Usage
### Core Features
The application comes with several core search providers.
- Applications
- Files, making use and requiring the `localsearch` indexer
- Evaluate mathematical expressions, making use of the [evalexpr crate](https://https://crates.io/crates/evalexpr)
- System actions (shutdown, reboot, etc)

### Prefixes
Each provider has a prefix to narrow down the search, the prefixes for the
official providers are:
- Apps: #
- Calculator: =
- Files: /
- System actions: *
### Configuration
The configuration file for lupa is located in `$XDG_CONFIG_HOME/lupa/conf.toml`,
usually resolving to `~/.config/lupa/conf.toml`, automatically generated on first launch
if it's missing.

You may always print the default configuration by running lupa with the
`--default-config` flag (or just `-d`).

Documentation for the configuration file are included in the default configuration.

## Credit
Some implementation details for app discovery and execution are heavily based in
[lucien](https://github.com/Wachamuli/lucien).

## TO-DO
- [ ] Stateful search icon, morphs depending on prefix
- [ ] More providers
    - [x] System actions (Shutdown, reboot, etc.)
    - [ ] Emoji browser
- [ ] Usage documentation
