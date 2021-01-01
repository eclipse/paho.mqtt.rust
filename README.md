# Eclipse Paho MQTT Rust Client Library

![Crates.io](https://img.shields.io/crates/d/paho-mqtt)

The [Eclipse Paho](http://eclipse.org/paho) MQTT Rust client library on memory-managed operating systems such as Linux/Posix, Mac, and Windows.

The Rust crate is a safe wrapper around the Paho C Library.

## Features

The initial version of this crate is a wrapper for the Paho C library, and includes all of the features available in that library, including:

- Supports MQTT v5, 3.1.1, and 3.1
- Network Transports:
    - Standard TCP support
    - SSL / TLS (with optional ALPN protocols)
    - WebSockets (secure and insecure), and optional Proxies
- QoS 0, 1, and 2
- Last Will and Testament (LWT)
- Message Persistence
    - File or memory persistence
    - User-defined key/value persistence (including example for Redis)
- Automatic Reconnect
- Offline Buffering
- High Availability
- Several API's:
    - Async/Await with Rust Futures and Streams for asynchronous operations.
    - Traditional asynchronous (token/wait) API
    - Synchronous/blocking  API

Requires Paho C v1.3.8, or possibly later.

## Latest News

To keep up with the latest announcements for this project, follow:

**Twitter:** [@eclipsepaho](https://twitter.com/eclipsepaho) and [@fmpagliughi](https://twitter.com/fmpagliughi)

**EMail:** [Eclipse Paho Mailing List](https://accounts.eclipse.org/mailing-list/paho-dev)

**Mattermost:** [Eclipse Mattermost Paho Channel](https://mattermost.eclipse.org/eclipse/channels/paho)

### What's new in v0.9.1

- [#101](https://github.com/eclipse/paho.mqtt.rust/issues/101) `Token::try_wait()` to check for the result of a `Token` without blocking.
- [#101](https://github.com/eclipse/paho.mqtt.rust/issues/101) A `try_publish()` function for the `AsyncClient` and `Topic` which return a synchronous result that the message was created and queued for transmission successfully.
- [#28](https://github.com/eclipse/paho.mqtt.rust/issues/28) Some instructions for using the "cross" tool for cross-compiling.

### What's new in v0.9.0

- Websocket HTTP/HTTPS proxy support
- Added missing MQTT v5 support:
    - Subscribe and Unsubscribe can now have v5 properties, thus enabling Subscription Identifiers.
- [Breaking] Persistence defaults to `None` if no Client ID specified in creation. 
- Ability to specify a path when using File persistence
- Updated bindings to Paho C v1.3.8
- Ability to start publishing (queuing) messages before first successful connection.
- New offline buffering options:
    - Ability to start publishing (queuing) messages before first successful connection.
    - Option to delete the oldest messages first from the queue when it fills up.
- New persistence options:
    - The option to not restore messages from persistence on startup (fresh restart).
    - The option to not persist QoS 0 messages.
- [#110](https://github.com/eclipse/paho.mqtt.rust/issues/110) Update to `futures-timer` v3.0
- [#95](https://github.com/eclipse/paho.mqtt.rust/issues/95) Added Send bounds to `ClientPersistence`
- [#92](https://github.com/eclipse/paho.mqtt.rust/issues/92) Vendored SSL with _openssl-sys_ crate (optional)
- New example apps:
    - _sync_consume_v5.rs_ - An MQTT v5 consumer that uses Subscription ID's to handle incoming messages.
    - _ws_publish.rs_ - Simeple websocket example with optional proxy.

## Using the Crate

To use the library, simply add this to your application's `Cargo.toml` dependencies list:

    paho-mqtt = "0.9"

By default it enables the features "bundled" and "ssl" meaning it will attempt to compile the Paho C library for the target, using the pre-built bindings, and will enable secure sockets capabilities.

Note that this default behavior requires a C compiler for the target and _CMake_ to be installed.

Also note that the build will use pre-generated bindings by default to speed up compile times. _If you experience segfaults or other hard crashes, the first thing to do is try using the "build_bindgen" feature in your crate to regenerate the bindings for your target._ If that doesn't fix it, then please submit an issue on [GitHub](https://github.com/eclipse/paho.mqtt.rust/issues).

### Configurable Features

The default behaviour can be altered by enabling or disabling the features:

- _"bundled"_ - Whether to build the Paho C library contained in the Git submodule under the contained _paho-mqtt-sys_ crate. This is similar to the "vendored" feature in other Rust projects.
- _"build_bindgen"_ - Whether to build the bindings for the target using _bindgen_. If not set, the build will attempt to find and use pre-built bindings for the target.
- _"ssl"_ - Whether to enable the use of secure sockets and secure websocket connections. 
- _"vendored-ssl"_ - Whether to build OpenSSL. This passes the "vendored" option to the _openssl-sys_ crate.

Version 0.9 has started using the [openssl-sys](https://crates.io/crates/openssl-sys) crate which allows for further modification of the behavior through environment variables, such as specifying the location of the OpenSSL library or linking it statically. See below for details, or get more information from that crate.

In particular, if you are using a pre-built OpenSSL library, you may now need to set the specific location of the library with an environment variable. For example, on Windows, you may need to do something like this:

    set OPENSSL_DIR=C:\OpenSSL-Win64

## Developing the Crate

The library is a standard Rust "crate" using the _Cargo_ build tool. It uses the standard cargo commands for building:

`$ cargo build`

Builds the library, and also builds the *-sys* subcrate and the bundled Paho C library. It includes SSL, as it is defined as a default feature.

`$ cargo build --examples`

Builds the library and sample applications in the _examples_ subdirectory.

`$ cargo test`

Builds and runs the unit tests.

`$ cargo doc`

Generates reference documentation.

###  The Paho C Library and _paho-mqtt-sys_

The Paho Rust crate is a wrapper around the Paho C library. This version is **specifically matched to Paho C v 1.3.x**, and is currently using version 1.3.8. It will generally not build against newer versions of the C library, as the C lib expands functionality by extending structures, thus breaking the Rust build.

The project includes a Rust _-sys_ crate, called _paho-mqtt-sys_, which provides unsafe bindings to the C library.  The repository contains a Git submodule pointing to the specific version of the C library that the Rust crate requires, and by default, it will automatically build and link to that library, using pre-generated C bindings that are also included in the repo.

When building, the user has several options:

 - Build the bundled library using the pre-generated bindings and SSL (default).
 - Build the bundled library and compile a copy of OpenSSL to statically link to.
 - Build the bundled library, but regenerate the bindings at build time.
 - Use an external library, with the location specified by environment variables, generating the bindings at build time.
 - Use the pre-installed library with the pre-generated bindings.

These are chosen with cargo features, explained below.

#### Building the bundled Paho C library

This is the default:

    $ cargo build

This will initialize and update the C library sources from Git, then use the _cmake_ crate to build the static version of the C library, and link it in. By default, the build will use the pre-generated bindings in _bindings/bindings_paho_mqtt_X_Y_Z.rs_, where _X_Y_Z_ is the currently supported library version.

The default features for the build are: ["bundled", "ssl"]

When building the bundled libraries, the bindings can also be regenerated at build-time. This is especially useful when building on uncommon/untested platforms to ensure proper bindings for that system. This is done adding the "build_bindgen" feature:

    $ cargo build --features "build_bindgen"

In this case it will generate bindings based on the header files in the bundled C repository.

The cached versions of the bindings are target-specific. If the pre-generated version doesn't exist for the target, it will need to be generated.


#### Building the Paho C library with or without SSL/TLS

To build the Paho C library with SSL/TLS we depend on the `openssl-sys` crate. The `openssl-sys` crate supports automatically detecting OpenSSL installations, manually pointing towards an OpenSSL installation using environment variables or building and statically linking to a vendored copy of OpenSSL (see the `openssl-sys` documentation for all available [options](https://docs.rs/openssl/latest/openssl/#building)). To use the vendored option, please use the `vendored-ssl` feature which also enables the `bundled` and `ssl` features.

Building with SSL happens automatically as `ssl` is a default feature. It requires the OpenSSL libraries be installed for the target. If they are in a non-standard place, then the `OPENSSL_DIR` environment variable should be set, pointing at the top-level install path, with the .lib, .a and other library files in a `lib/` directory just under the root. Use like:

    $ export OPENSSL_DIR=/home/myacct/openssl

or wherever the library was installed.

The crate can also be build without SSL by using `--no-default-features`. For example, to build the bundled Paho C library without secure sockets:

    $ cargo build --no-default-features --features "bundled"

##### Linking OpenSSL Statically

Enable the `--vendored-ssl` feature to build the crate with a compiled and statically linked copy of OpenSSL. The `--vendored-ssl` feature also enables the `bundled` and `ssl` features, so either of these command will work:

    $ cargo build --features "vendored-ssl"
    $ cargo build --no-default-features --features "vendored-ssl"


#### Linking to an external Paho C library

The crate can generate bindings to a copy of the Paho C library in a different location in the local file system, and link to that library.

    $ cargo build --no-default-features --features "build_bindgen,ssl"

The `ssl` feature can be omitted if it is not desired.

The location of the C library is specified through an environment variable:

    PAHO_MQTT_C_DIR= ...path to install directory...

It's assumed that the headers are in an _include/_ directory below the one specified, and the library is in _lib/_ under it. This would be the case with a normal install.

Alternately, this can be expressed with individual environment variables for each of the header and library directories:

    PAHO_MQTT_C_INCLUDE_DIR= ...path to headers...
    PAHO_MQTT_C_LIB_DIR= ...path to library...

In this case, the headers and library can be found independently. This was necessary when building against a development tree for Paho C that used GNU Make build. This doesn't seem as necessary now that CMake is used everywhere.

#### Linking to an installed Paho C library

If the correct version of the Paho C library is expected to be installed on the target system, the simplest solution is to use the pre-generated bindings and specify a link to the shared Paho C library.

    $ cargo build --no-default-features --features "ssl"

This is especially useful in a production environment where the system is well controlled, such as  when working with full-system build tools like _yocto_ or _buildroot_. It could be easier to build or cross-compile the packages separately.

Again, the `ssl` feature can be omitted if it is not desired.

This option should be used with caution when building an application that will ship independently of the target system, since it assumes a _very specific_ version of the C library and will fail if that is not the one on the target.

#### Rust-C Bindings

As described above, the crate can optionally use _bindgen_ to create the bindings to the Paho C library.

https://rust-lang-nursery.github.io/rust-bindgen/

Generating bindings each time you build the Rust crate is time consuming and uses a lot of resources. This is especially noticeable when building natively on a small target like an ARM board, or similar.

But each release of the Rust crate is build against a specific version of the Paho C library, which means that for a specific target, the bindings never change from build to build. Therefore, we can create the bindings once for a target and then use them for a speedy build after that.

The crate comes with a number of pre-built bindings for several popular targets in: `paho-mqtt-sys/bindings`. These are files with names in the form:

    bindings_paho_mqtt_c_<version>-<target>.rs

Some of these include:

    bindings_paho_mqtt_c_1.3.8-x86_64-unknown-linux-gnu.rs
    bindings_paho_mqtt_c_1.3.8-x86_64-pc-windows-msvc.rs
    bindings_paho_mqtt_c_1.3.8-aarch64-unknown-linux-gnu.rs
    bindings_paho_mqtt_c_1.3.8-armv7-unknown-linux-gnueabihf.rs
    bindings_paho_mqtt_c_1.3.8-x86_64-apple-darwin.rs
    bindings_paho_mqtt_c_1.3.8-default-32.rs
    bindings_paho_mqtt_c_1.3.8-default-64.rs

Bindings can be created for new versions of the Paho C library or for different target platforms using the command-line _bindgen_ tool. For example on an x86 version of Windows using MSVC, you can re-generate the bindings like this:

```
$ cd paho-mqtt-sys
$ bindgen wrapper.h -o bindings/bindings_paho_mqtt_c_1.3.8-x86_64-pc-windows-msvc.rs -- -Ipaho.mqtt.c/src
```

To create bindings for a different target, use the _TARGET_ environment variable. For example, to build the 32-bit MSVC bindings for Windows on a 64-bit host, use the _i686-pc-windows-msvc_ target:

```
$ TARGET=i686-pc-windows-msvc bindgen wrapper.h -o bindings/bindings_paho_mqtt_c_1.3.8-i686-pc-windows-msvc.rs -- -Ipaho.mqtt.c/src
```

##### Bindgen linker issue

Bindgen requires a relatively recent version of the Clang library installed on the system - recommended v3.9 or later. The bindgen dependencies seem, however, to seek out the oldest Clang version if multiple ones are installed on the system. On Ubuntu 14.04 or 16.04, the Clang v3.6 default might give some problems, although as the Paho builder is currently configured, it should work.

But the safest thing would be to set the `LIBCLANG_PATH` environment variable to point to a supported version, like:
```
export LIBCLANG_PATH=/usr/lib/llvm-3.9/lib
```

### Cross-Compiling

The *cmake* crate automatically handles cross-compiling libraries. You'll need a C cross-compiler installed on your system. See here for more info about cross-compiling Rust, in general:

https://github.com/japaric/rust-cross

[The Rust Book](https://rust-lang.github.io/rustup/cross-compilation.html)


For example, to do a full build for `ARMv7`, which includes Raspberry Pi's, BeagleBones, UDOO Neo's, and lots of other ARM maker boards:

    $ cargo build --target=armv7-unknown-linux-gnueabihf --examples

This builds the main crate, the *-sys* crate, and it cross-compiles the Paho C library. It uses SSL, so it requires you to have a version of the SSL development library installed with the cross-compiler. If the SSL libraries are not available you can compile and link them as part of the Rust build using the `--vendored-ssl` feature:
```
$ cargo build --target=armv7-unknown-linux-gnueabihf --features="vendored-ssl" --examples
```

If you don't want to use SSL with the cross-compiler:
```
$ cargo build --target=armv7-unknown-linux-gnueabihf --no-default-features --features="bundled" --examples
```

If the triplet of the installed cross-compiler doesn't exactly match that of the Rust target, you might also need to correct the `CC` environment variable:
```
$ CC_armv7-unknown-linux-gnueabihf=armv7-unknown-linux-gnueabihf-gcc cargo build --target=armv7-unknown-linux-gnueabihf --features="vendored-ssl" --examples
```

#### Cross-Compiling with the "cross" project.

The [cross](https://github.com/rust-embedded/cross) project is a cross-compilation build tool that utilizes docker containers pre-loaded with the build tools for a number of targets. It requires [Docker](https://docs.docker.com/get-docker/) to be installed and running on your system.

Then build/install the `cross` tool:

    $ cargo install cross

After that, you should be able to build the project for any of the supported targets. Just use the `cross` command instead of cargo.

```
$ cross build --target=armv7-unknown-linux-gnueabihf \
    --features=vendored-ssl --examples
```

## Fully Static Builds with _musl_

With the v0.9 release and beyond, it should be fairly easy to create fully static builds of applications that use the Paho crate using the _musl_ library and tools.

On a recent Ubuntu/Mint Linux host it should work as follows, but should be similar on any development host once the tools are installed.

First install the Rust compiler for _musl_ and the tools:

    $ rustup target add x86_64-unknown-linux-musl
    $ sudo apt install musl-tools

Check the _musl_ compiler:

    $ musl-gcc --version
    cc (Ubuntu 7.5.0-3ubuntu1~18.04) 7.5.0
    ...

Building without SSL is like this:

```
$  cargo build --no-default-features --features="bundled" \
    --target=x86_64-unknown-linux-musl --examples 
```

## Logging

The Rust library uses the `log` crate to output debug and trace information. Applications can choose to use one of the available logger implementations or define one of their own. More information is available at:

https://docs.rs/log/0.4.0/log/

The sample applications use the environment log crate, `env_logger` to configure output via the `RUST_LOG` environment variable. To use this, the following call is specified in the samples before using any of the Rust MQTT API:

    env_logger::init().unwrap();

And then the library will output information as defined by the environment. Use like:

    $ RUST_LOG=debug ./async_publish
    DEBUG:paho_mqtt::async_client: Creating client with persistence: 0, 0x0
    DEBUG:paho_mqtt::async_client: AsyncClient handle: 0x7f9ae2eab004
    DEBUG:paho_mqtt::async_client: Connecting handle: 0x7f9ae2eab004
    ...

In addition, the underlying Paho C library has its own logging capabilities which can be used to trace network and protocol transactions. It is configured by the environment variables `MQTT_C_CLIENT_TRACE` and `MQTT_C_CLIENT_TRACE_LEVEL`. The former names the log file, with the special value "ON" to log to stdout. The latter specifies one of the levels: ERROR, PROTOCOL, MINIMUM, MEDIUM and MAXIMUM.

    export MQTT_C_CLIENT_TRACE=ON
    export MQTT_C_CLIENT_TRACE_LEVEL=PROTOCOL

## Example

Several small sample applications can be found in the _examples_ directory. Here is what a small MQTT publisher might look like:

```
use std::process;

extern crate paho_mqtt as mqtt;

fn main() {
    // Create a client & define connect options
    let cli = mqtt::Client::new("tcp://localhost:1883").unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    // Create a message and publish it
    let msg = mqtt::Message::new("test", "Hello world!");
    let tok = cli.publish(msg);

    if let Err(e) = tok.wait() {
        println!("Error sending message: {:?}", e);
    }

    // Disconnect from the broker
    let tok = cli.disconnect();
    tok.wait().unwrap();
}
```

## External Libraries and Utilities

Several external projects are under development which use or enhance the Paho MQTT Rust library. These can be used in a system with the Rust library or serve as further examples of it's use.

### Redis Persistence

The `mqtt-redis` create allows the use of Redis as a persistence store. It also provides a good example of creating a user-defined persistence which implements the `ClientPersistence` trait. It can be found at:

https://github.com/fpagliughi/mqtt.rust.redis
