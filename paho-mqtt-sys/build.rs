// paho-mqtt-sys/build.rs
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017-2018 Frank Pagliughi <fpagliughi@mindspring.com>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v10.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

// Run 'cargo build -vv' to see debug output to console or check the 'output'
// file in the build directory.
//
// We write some strings "debug:..."
// This is helpful to figure out what the build is doing.

// To work with development branches of the C lib, the user can define
// environment variables:
//      PAHO_MQTT_C_INCLUDE_DIR= ...path to headers...
//      PAHO_MQTT_C_LIB_DIR= ...path to library...`
//
// or you could simply set:
//      PAHO_MQTT_C_DIR=...
//
// in which case it will assume:
//      PAHO_MQTT_C_INCLUDE_DIR=$PAHO_C_DIR/include
//      PAHO_MQTT_C_LIB_DIR=$PAHO_C_DIR/lib
//
// The basic decision tree is as follow:
//  + If "bundled" feature, compile the bundled C lib
//    - If "build_bindgen" feature, regenerate bindings
//    - else use bundled bindings
//  + else (!"bundled")
//    - If environment vars set use that lib,
//      ^ If "builtime_bindgen" generate bindings for user lib
//      ^ else assume proper version and use bundled bindings
//    - else (no env)
//      ^ If "builtime_bindgen" exit with an error
//      ^ else assume system install and use bundled bindings
//

// TODO: Assuming the proper installed version of the library is problematic.
//      We should check that the version is correct, if possible.

fn main() {
    build::main();
}

// Determine if we're usine SSL or not, by feature request.
// This determines which Paho C library we link to.
fn link_lib() -> &'static str {
    if cfg!(feature = "ssl") {
        println!("debug:link Using SSL library");
        "paho-mqtt3as-static"
    }
    else {
        println!("debug:link Using non-SSL library");
        "paho-mqtt3a-static"
    }
}

#[cfg(not(feature = "build_bindgen"))]
mod bindings {
    const PAHO_MQTT_C_VERSION: &'static str = "1.2.1";

    use std::{env, fs};
    use std::path::Path;

    pub fn place_bindings(_inc_dir: &str) {
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir).join("bindings.rs");

        let bindings = format!("bindings/bindings_paho_mqtt_c_{}.rs", PAHO_MQTT_C_VERSION);
        fs::copy(&bindings, out_path)
            .expect("Could not copy bindings to output directory");
    }
}

#[cfg(feature = "build_bindgen")]
mod bindings {
    extern crate bindgen;

    use std::env;
    use std::path::PathBuf;

    pub fn place_bindings(inc_dir: &str) {
        let cver = bindgen::clang_version();
        println!("debug:clang version: {}", cver.full);
        println!("debug:bindgen include path: {}", inc_dir);

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // Older clang versions (~v3.6) improperly mangle the functions.
            // We shouldn't require mangling for straight C library. I think.
            .trust_clang_mangling(false)
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h").clang_arg(inc_dir)
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}

#[cfg(feature = "bundled")]
mod build {
    extern crate cmake;

    use super::*;
    use std::{/*env, fs,*/ process};
    use std::path::Path;
    use std::process::Command;

    pub fn main() {
        // we rerun the build if the `build.rs` file is changed.
        println!("cargo:rerun-if-changed=build.rs");

        // Mske sure that the Git submodule is checked out

        if !Path::new("paho.mqtt.c/.git").exists() {
            let _ = Command::new("git")
                        .args(&["submodule", "update", "--init"])
                        .status();
        }

        // Use cmake to build the C lib
        let ssl = if cfg!(feature = "ssl") { "on" } else { "off" };

        let mut cmk = cmake::Config::new("paho.mqtt.c/")
            .define("PAHO_BUILD_STATIC", "on")
            .define("PAHO_WITH_SSL", ssl)
            .build();

        // We check if the target library was compiled.
        let cmk_out_dir = cmk.clone();
        let cmk_out_dir = Path::new(&cmk_out_dir).join("lib");

        let link_lib = link_lib();
        let link_file = format!("lib{}.a", link_lib);

        let lib = cmk_out_dir.join(Path::new(&link_file));
        if !lib.exists() {
            println!("Error building Paho C library: '{}'", lib.to_string_lossy());
            process::exit(103);
        }

        // Get bundled bindings or regenerate
        let mut cmk_inc = cmk.clone();
        cmk_inc.push("include");
        let inc_dir = format!("{}", cmk_inc.display());
        bindings::place_bindings(&inc_dir);


        // we add the folder where all the libraries are built to the path search
        cmk.push("lib");

        if cfg!(feature = "ssl") {
            println!("cargo:rustc-link-lib=ssl");
            println!("cargo:rustc-link-lib=crypto");
        }

        println!("cargo:rustc-link-search=native={}", cmk.display());
        println!("cargo:rustc-link-lib=static={}", link_lib);
    }
}


#[cfg(not(feature = "bundled"))]
mod build {
    use super::*;
    use std::env;

    // Set the library path, and return the location of the header,
    // if found.
    fn find_paho_c() -> Option<String> {
        let link_lib = link_lib();

        println!("cargo:rerun-if-env-changed=PAHO_MQTT_C_DIR");
        println!("cargo:rerun-if-env-changed=PAHO_MQTT_C_INCLUDE_DIR");
        println!("cargo:rerun-if-env-changed=PAHO_MQTT_C_LIB_DIR");

        if cfg!(target_os = "windows") {
            println!("cargo:rerun-if-env-changed=PATH");
        }

        println!("cargo:rustc-link-lib={}", link_lib);

        // Allow users to specify where to find the C lib.
        if let Ok(lib_dir) = env::var("PAHO_MQTT_C_LIB_DIR") {
            if let Ok(inc_dir) = env::var("PAHO_MQTT_C_INCLUDE_DIR") {
                println!("debug:inc_dir={}", inc_dir);
                println!("debug:lib_dir={}", lib_dir);

                println!("cargo:rustc-link-search={}", lib_dir);
                return Some(inc_dir);
            }
            else {
                panic!("If specifying lib dir, must also specify include dir");
            }
        }

        if let Ok(dir) = env::var("PAHO_MQTT_C_DIR") {
            //println!("cargo:rustc-link-lib={}", link_lib);
            println!("cargo:rustc-link-search={}", format!("{}/lib", dir));
            return Some(format!("{}/include", dir));
        }

        //println!("cargo:rustc-link-search=native={}", lib_path);
        None
    }

    pub fn main() {
        // We will use the directory `paho.mqtt.c`, if something change
        // there we should rerun the compile step.
        println!("cargo:rerun-if-changed=paho.mqtt.c/");

        let inc_dir = find_paho_c().unwrap_or_default();
        if cfg!(feature = "build_bindgen") && inc_dir.is_empty() {
            panic!("Can't generate bindings. Unknown library location");
        }

        bindings::place_bindings(&inc_dir);
    }
}

