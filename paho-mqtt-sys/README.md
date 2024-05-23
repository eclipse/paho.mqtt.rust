# Low-level Eclipse Paho MQTT C Client Library Wrapper

[![docs.rs](https://docs.rs/paho-mqtt-sys/badge.svg)](https://docs.rs/paho-mqtt-sys)
[![crates.io](https://img.shields.io/crates/d/paho-mqtt-sys?label=crates.io%20downloads)](https://crates.io/crates/paho-mqtt-sys)

An un-safe, low-level, wrapper around the [Eclipse Paho](http://eclipse.org/paho) C Library, which can be used to write Rust MQTT client library on memory-managed operating systems such as Linux/Posix, Mac, and Windows. This is primarily used by the [Eclipse Paho Rust Library](https://crates.io/crates/paho-mqtt), which provides a safe Rust interface over this one.

This package can build the recommended version of Paho C automatically. This is the default behavior, which comes in withthe _"bundled"_ feature. It uses the [cmake crate](https://crates.io/crates/cmake) which can also cross-compile the C library for most targets.

When not using the _bundled_ build, it will attept to link to a pre-installed version of the library. It is fairly sensitive to the C version.

The current recommended Paho C version is: v1.3.13

## Configurable Features

The default features are: `["bundled", "ssl"]`

The full set of features include the following:

- _"bundled"_ - Whether to build the Paho C library contained in the Git submodule. This is similar to the "vendored" feature in other Rust projects. If not selected, it will attempt to find and link to a pre-installed version of the Paho C library for the target.
- _"build_bindgen"_ - Whether to generate the C language bindings for the target using _bindgen_. If not set, the build will attempt to find and use pre-built bindings for the target.
- _"ssl"_ - Whether to enable the use of secure sockets and secure websocket connections.
- _"vendored-ssl"_ - Whether to build OpenSSL. This passes the "vendored" option to the _openssl-sys_ crate. This also selects the _"ssl"_ option, if not already set.

The _bundled_ feature requires `CMake` and a C compiler for the target.

The _vendored-ssl_ feature requires the target C compiler as well, but also requires `Perl` and `make`.

### Using SSL/TLS

Starting with Version 0.5.0 we are using the [openssl-sys](https://crates.io/crates/openssl-sys) crate which allows for further modification of the behavior through environment variables, such as specifying the location of the OpenSSL library or linking it statically.

For more information read the [Rust OpenSSL Docs](https://docs.rs/openssl/latest/openssl), _carefully_.

In particular:

- If you use _vendored-ssl_, you need a C compiler for the target, `Perl`, and `make`.

- If you don't use _vendored-ssl_, it will attempt to use a package manager on the build host to find the library: `pkg-config` on Unix-like systems, `Homebrew` on macOS, and `vcpkg` on Windows.

- If all else fails, you may need to set the specific location of the library with an environment variable. For example, on Windows, you may need to do something like this:

    set OPENSSL_DIR=C:\OpenSSL-Win64
