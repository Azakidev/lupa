{
    description = "Lupa's development environment for Nix";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs, flake-utils }:
        flake-utils.lib.eachDefaultSystem (system:
        let
            pkgs = import nixpkgs { inherit system; };
        in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "lupa";
            version = "0.4.2";
            src = ./.;

            cargoLock = {
                lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
                blueprint-compiler
                gettext
                pkg-config
                wrapGAppsHook4
            ];

            buildInputs = with pkgs; [
                glib
                gtk4
                gtk4-layer-shell
                libadwaita
                localsearch
                luajit
                xdg-utils
            ];

            # Build and install translations
            postInstall = ''
                if [ -d po ]; then
                    for po in po/*.po; do
                        if [ -f "$po" ]; then
                            lang="$(basename "$po" .po)"
                            mkdir -p "$out/share/locale/$lang/LC_MESSAGES"
                            msgfmt -o "$out/share/locale/$lang/LC_MESSAGES/lupa.mo" "$po"
                        fi
                    done
                fi
            '';

            meta = {
                    description = "A minimalist launcher built on gtk4-layer-shell.";
                    homepage = "https://github/Azakidev/lupa";
                    license = pkgs.lib.licenses.mit;
                };

            };

             devShells.default = pkgs.mkShell {
                 inputsFrom = [ self.packages.${system}.default ];

                 packages = with pkgs; [
                     cargo
                     cargo-audit
                     clippy
                     rust-analyzer
                     rustc
                     rustfmt
                 ];

                 RUST_BACKTRACE = "1";
             };
         }
     );
}
