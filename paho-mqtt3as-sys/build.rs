extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
	let target = env::var("TARGET").expect("TARGET was not set");
	let mut hdr_path = "-I.";

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
	// The bindgen::Builder is the main entry point
	// to bindgen, and lets you build up options for
	// the resulting bindings.
	let bindings = bindgen::Builder::default()
		// The input header we would like to generate
		// bindings for.
		.header("wrapper.h").clang_arg(hdr_path)
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

