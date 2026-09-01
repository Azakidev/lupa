/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::{env, fs, path::Path, process::Command};

static GETTEXT_PACKAGE: &str = "lupa";

fn main() {
    // Re-run build if translation files change
    println!("cargo:rerun-if-changed=src/ui");

    glib_build_tools::compile_resources(
        &["src/ui"],
        "src/ui/resources.gresource.xml",
        "lupa.gresource",
    );

    println!("cargo:rerun-if-changed=po/POTFILES.in");
    println!("cargo:rerun-if-changed=po/");

    let po_dir = Path::new("po");
    let potfiles_path = po_dir.join("POTFILES");
    let out_dir = env::var("OUT_DIR").unwrap();
    let locale_dir = Path::new(&out_dir).join("locale");

    let mut files_to_translate = Vec::new();

    if potfiles_path.exists() {
        let file = fs::read_to_string(&potfiles_path).unwrap();

        for line in file.lines() {
            let line = line.trim();
            // Ignore empty lines and comments
            if !line.is_empty() && !line.starts_with('#') {
                files_to_translate.push(line.to_string());
                // Tell Cargo to watch this specific file for changes!
                println!("cargo:rerun-if-changed={}", line);
            }
        }
    } else {
        println!("cargo:warning=po/POTFILES not found. Skipping POT generation.");
    }

    // Generate lupa.pot
    if !files_to_translate.is_empty() {
        let pot_path = po_dir.join(format!("{}.pot", GETTEXT_PACKAGE));

        let mut cmd = Command::new("xgettext");
        cmd.args([
            "--from-code=UTF-8",
            "--keyword=_",
            "--language=C", // Treats .blp / Rust string syntax correctly
            "-o",
            pot_path.to_str().unwrap(),
        ]);

        for file in &files_to_translate {
            cmd.arg(file);
        }

        let status = cmd
            .status()
            .expect("Failed to run xgettext. Is gettext installed?");
        if !status.success() {
            eprintln!("Warning: xgettext failed to extract strings.");
        }
    }

    // Compile po files
    if po_dir.exists() {
        for entry in fs::read_dir(po_dir).expect("Could not read po directory") {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("po") {
                let lang = path.file_stem().unwrap().to_str().unwrap();
                let lang_locale_dir = locale_dir.join(lang).join("LC_MESSAGES");
                fs::create_dir_all(&lang_locale_dir).unwrap();

                let mo_path = lang_locale_dir.join(format!("{}.mo", GETTEXT_PACKAGE));

                let status = Command::new("msgfmt")
                    .args(["-o", mo_path.to_str().unwrap(), path.to_str().unwrap()])
                    .status()
                    .expect("Failed to execute msgfmt.");

                if !status.success() {
                    panic!("msgfmt failed for file {:?}", path);
                }
            }
        }
    }
}
