# Lupa

<p align="center">
  <img src="https://raw.githubusercontent.com/Azakidev/lupa/refs/heads/main/dist/logo.svg" />

  A minimalist launcher built with gtk4-layer-shell and rust.
</p>


## Usage
> [!NOTE]
> On some Wayland compositors, the clipboard may not persist after the app closes.
> You may want to try [wl-clip-persist](https://github.com/Linus789/wl-clip-persist)
> if this is the case.

### Core Features
Lupa comes with several core search providers.
- Applications
- Files, making use and requiring the `localsearch` indexer
- Evaluate mathematical expressions, making use of the [evalexpr crate](https://crates.io/crates/evalexpr)
- System actions (shutdown, reboot, etc)
- Emoji

### Prefixes
Each provider has a prefix to narrow down the search, the prefixes for the
official providers are:
- Apps: #
- Calculator: =
- Files: /
- System actions: *
- Emoji: !

### Configuration
The configuration file for lupa is located in `$XDG_CONFIG_HOME/lupa/conf.toml`,
usually resolving to `~/.config/lupa/conf.toml`, automatically generated on first launch
if it's missing.

You may always print the default configuration by running lupa with the
`--default-config` flag (or just `-d`).

Documentation for the configuration file are included in the default configuration.

## Screenshots
<p align="center">
<img src="https://raw.githubusercontent.com/Azakidev/lupa/refs/heads/main/dist/ss/1.png" />
</p>

> Lupa showing multiple result types.

<p align="center">
<img src="https://raw.githubusercontent.com/Azakidev/lupa/refs/heads/main/dist/ss/2.png" />
</p>

> Lupa showing the sidebar results for an application.

<p align="center">
<img src="https://raw.githubusercontent.com/Azakidev/lupa/refs/heads/main/dist/ss/3.png" />
</p>

> Lupa showing a system action.

## Credit
Some implementation details for app discovery and execution are heavily based in
[lucien](https://github.com/Wachamuli/lucien).

## TO-DO
- [ ] Stateful search icon, morphs depending on prefix
