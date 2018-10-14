// build.rs
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017 Frank Pagliughi <fpagliughi@mindspring.com>
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

// TODO:
// 
// Currently this should be able to use the Paho C library if it was 
// already compiled and installed to the normal system directory.
// 
// Eventually it should be able to download the C library from GitHub, 
// then build and install it.
//
// To work with development branches of the C lib, the user can define 
// environment variables:
//		PAHO_MQTT_C_INC_PATH= ...path to headers...
//		PAHO_MQTT_C_LIB_PATH= ...path to library...`
//

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {

	// Run 'cargo build -vv' to see debug output to console.
	// This is helpful to figure out what the build is doing.
	let cver = bindgen::clang_version();
	println!("debug:clang version: {}", cver.full);

	let target = env::var("TARGET").expect("TARGET was not set");

	//let paho_c_path_env = env::var("PAHO_C_PATH").unwrap_or("<unknown>".to_string());

	// TODO: These should not hard-code paths on my workstation.

	let dflt_inc_path = String::from(if target.contains("linux") {
										 "/home/fmp/mqtt/paho.mqtt.c/src"
									 }
									 else if target.contains("windows") {
										 r"D:\mqtt\paho.mqtt.c\src"
									 }
									 else {
										 "."
									 });

	let dflt_lib_path = String::from(if target.contains("linux") {
										 "/home/fmp/mqtt/paho.mqtt.c/build/output"
									 }
									 else if target.contains("windows") {
										 r"D:\mqtt\paho.mqtt.c\build\src\Debug"
									 }
									 else {
										 "."
									 });

	let paho_c_inc_path = match env::var("PAHO_MQTT_C_INC_PATH") {
		Ok(path) => path,
		_ => match env::var("PAHO_MQTT_C_PATH") {
				Ok(path) => path + "/src",
				_ => dflt_inc_path,
		},
	};

	let paho_c_lib_path = match env::var("PAHO_MQTT_C_LIB_PATH") {
		Ok(path) => path,
		_ => match env::var("PAHO_MQTT_C_PATH") {
				Ok(path) => path + "/build/output",
				_ => dflt_lib_path,
		},
	};

	let inc_path = format!("-I{}", paho_c_inc_path);
	let lib_path = paho_c_lib_path;

	println!("debug:inc_path={}", inc_path);
	println!("debug:lib_path={}", lib_path);

	//let paho_c_lib_path_env = env::var("PAHO_C_LIB_PATH");

	// This is the path for a local GNU Make build of the Paho C lib

	// This would be the path for a CMake build w/ local install
	//let link_path = "/home/fmp/mqtt/paho.mqtt.c/build-cmake/_install/lib";

	//println!("cargo:include={}", hdr_path);

	// Tell cargo to tell rustc to link the Paho MQTT C library
	// shared library.
	// This would be the link specifier for the static library
	//println!("cargo:rustc-link-lib=static=paho-mqtt3as-static");

	println!("cargo:rustc-link-search=native={}", lib_path);

	if target.contains("windows") {
		println!("cargo:rustc-link-lib=static=paho-mqtt3a-static");
	}
	else {
		println!("cargo:rustc-link-lib=paho-mqtt3as");
	}

	/*
	if target.contains("linux") {
		hdr_path = "-I/home/fmp/mqtt/paho.mqtt.c/src";

		// This is the path for a local GNU Make build of the Paho C lib
		let link_path = "/home/fmp/mqtt/paho.mqtt.c/build/output";

		// This would be the path for a CMake build w/ local install
		//let link_path = "/home/fmp/mqtt/paho.mqtt.c/build-cmake/_install/lib";

		//println!("cargo:include={}", hdr_path);

		// Tell cargo to tell rustc to link the Paho MQTT C library
		// shared library.
		// This would be the link specifier for the static library
		//println!("cargo:rustc-link-lib=static=paho-mqtt3as-static");

		println!("cargo:rustc-link-lib=paho-mqtt3as");
		println!("cargo:rustc-link-search=native={}", link_path);
	}
	else if target.contains("windows") {
		hdr_path = r"-ID:\mqtt\paho.mqtt.c\src";
		let link_path = r"D:\mqtt\paho.mqtt.c\build\src\Debug";

		println!("cargo:rustc-link-lib=static=paho-mqtt3a-static");
		println!("cargo:rustc-link-search=native={}", link_path);

	}
	*/

	// The bindgen::Builder is the main entry point
	// to bindgen, and lets you build up options for
	// the resulting bindings.
	let bindings = bindgen::Builder::default()
		// Older clang versions (~v3.6) improperly mangle the functions.
		// We shouldn't require mangling for straight C library. I think.
		.trust_clang_mangling(false)
		// The input header we would like to generate
		// bindings for.
		.header("wrapper.h").clang_arg(inc_path)
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

