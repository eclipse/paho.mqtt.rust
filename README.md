# Eclipse Paho MQTT Rust Client Library

![Crates.io](https://img.shields.io/crates/d/paho-mqtt)

This repository contains the source code for the [Eclipse Paho](http://eclipse.org/paho) MQTT Rust client library on memory-managed operating systems such as Linux/Posix, Mac, and Windows.

The Rust crate is a safe wrapper around the Paho C Library. 

## Features

The initial version of this crate is a wrapper for the Paho C library, and includes all of the features available in that library, including:

- Supports MQTT v5, 3.1.1, and 3.1
- Network Transports:
    - Standard TCP support
    - SSL / TLS
    - WebSockets
- QoS 0, 1, and 2
- Last Will and Testament (LWT)
- Message Persistence 
    - File or memory persistence
    - User-defined key/value persistence (including example for Redis)
- Automatic Reconnect
- Offline Buffering
- High Availability
- Several API's:
    - Rust Futures and Streams for asynchronous operations.
    - Traditional asynchronous (token/wait) API
    - Synchronous/blocking  API

Supports Paho C v1.3.2

## Latest News

Version 0.7 brings full support for MQTT v5.

Work has started to move the library to modern Rust, including upgrading to the 2018 Edition and implementing the asynchronous client using async/await. That should hopefully be complete by June 2020.

To keep up with the latest announcements for this project, follow:

**Twitter:** [@eclipsepaho](https://twitter.com/eclipsepaho) and [@fmpagliughi](https://twitter.com/fmpagliughi)

**EMail:** [Eclipse Paho Mailing List](https://accounts.eclipse.org/mailing-list/paho-dev)

**Mattermost:** [Eclipse Mattermost Paho Channel](https://mattermost.eclipse.org/eclipse/channels/paho)

### What's new in v0.7

Version 0.7 brings full support for MQTT v5, including:

- Ability to create an MQTT v5 client and request a v5 connection to the server.
- MQTT v5 `Properties` (for connect, publish, incoming messages, etc)
- `ReasonCode` and better error notifications.
- [Breaking] Restored the single `Token` type, getting rid of separate implementations of `ConnectToken`, `SubscribeToken`, etc.
- Subscribe options, such as "No Local," etc.
- `Topic` objects can now be used to subscribe.
- New callback `on_disconnect()` for when the client receives a disconnect packet from the server, complete with a reason code and properties.
- Example for a simple chat application _(mqttrs_chat)_ using the v5 "No Local" subscription option. The publisher does not get their own messages echoed back to them.
 - Examples for RPC using v5 _Properties_ for _ResponseTopic_ and _CorrelationData:_
     - A math RPC service/server _(rpc_math_srvr)_ that performs basic operations on a list of numbers. 
     - A math RPC client  _(rpc_math_cli)_ that can send requests.

Also:

- Fix for #48: Sends a _None_ (and exits consumer) on manual disconnect.
- Fix for #49: Supporting `on_connect()` callback.
- Fix for #51: Segfault on `subscribe_many()` with a single topic.
- The build now uses the environment variable `OPENSSL_ROOT_DIR` to help find the SSL libraries in a non-standard install directory.

Note that v0.7 still targets Futures v0.1 and Rust Edition 2015. Support for async/await, and std Future (0.3) will be coming shortly in v0.8.

## Building the Crate

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

The Paho Rust crate is a wrapper around the Paho C library. This version is **specifically matched to Paho C v 1.3.x**, and is currently using version 1.3.2. It will generally not build against newer versions of the C library, as the C lib expands functionality by extending structures, thus breaking the Rust build.

The project includes a Rust _-sys_ crate, called _paho-mqtt-sys_, which provides unsafe bindings to the C library.  The repository contains a Git submodule pointing to the specific version of the C library that the Rust crate requires, and by default, it will automatically build and link to that library, using pre-generated C bindings that are also included in the repo.

When building, the user has several options:

 - Build the bundled library using the pre-generated bindings and SSL (default).
 - Build the bundled library, but regenerate the bindings at build time.
 - Use an external library, with the location specified by environment variables, generating the bindings at build time.
 - Use the pre-installed library with the pre-generated bindings.

These are chosen with cargo features, explained below.

#### Building the bundled Paho C library

This is the default:

    $ cargo build
    
This will initialize and update the C library sources from Git, then use the _cmake_ crate to build the static version of the C library, and link it in. By default, the build will use the pre-generated bindings in _bindings/bindings_paho_mqtt_X_Y_Z.rs_, where _X_Y_Z_ is the currently supported library version.

The defalut features for the build are: ["bundled", "ssl"]

When building the bundled libraries, the bindings can also be regenerated at build-time. This is especially useful when building on uncommon/untested platforms to ensure proper bindings for that system. This is done adding the "build_bindgen" feature:

    $ cargo build --features "build_bindgen"
    
In this case it will generate bindings based on the header files in the bundled C repository.

The cached versions of the bindings are target-specific. If the pre-generated version doesn't exist for the target, it will need to be generated.


#### Building the Paho C library with or without SSL/TLS

Building with SSL happens automatically as "ssl" is a default feature. It requires the OpenSSL libraries be installed for the target. If they are in a non-standard place, then the `OPENSSL_ROOT_DIR` environment variable should be set, pointing at the top-level install path, with the .lib, .a and other library files in a `lib/` directory just under the root. Use like:

    $ export OPENSSL_ROOT_DIR=/home/myacct/openssl

or wherever the library was installed.

The crate can also be build without SSL by using `--no-default-features`. For example, to build the bundled Paho C library without secure sockets:

    $ cargo build --no-default-features --features "bundled"


#### Linking to an exteral Paho C library

The crate can generate bindings to a copy of the Paho C library in a different location in the local file system, and link to that library. 

    $ cargo build --no-default-features --features "build_bindgen,ssl"

The "ssl" feature can be omitted if it is not desired. 

The location of the C library is specified through an environment variable:

    PAHO_MQTT_C_DIR= ...path to install directory...
    
It's assumed that the headers are in an _include/_ directory below the one specified, and the library is in _lib/_ under it. This would be the case with a normal install.

Alternately, this can be expressed with individual environment variables for each of the header and library directories:

    PAHO_MQTT_C_INCLUDE_DIR= ...path to headers...
    PAHO_MQTT_C_LIB_DIR= ...path to library...

In this case, the headers and library can be found independently. This was necessary when building against a development tree for Paho C that used GNU Make build. This doesn't seem as necessary now that CMake is used everywhere.

#### Linking to an installed Paho C library

If the correct version of the Paho C library is expected to be installed on the target system, the simplest solution is to use the pre-generated bindings and specify a link to the shared paho C library. 

    $ cargo build --no-default-features --features "ssl"

This is especially useful in a production environment where the system is well controlled, such as  when working with full-system build tools like _yocto_ or _buildroot_. It could be easier to build or cross-compile the packages separately.

Again, the "ssl" feature can be omitted if it is not desired.

This option should be used with caution when building an application that will ship independetly of the target system, since it assumes a _very specific_ version of the C library and will fail if that is not the one on the target.

#### Rust-C Bindings

As described above, the crate can optionally use _bindgen_ to create the bindings to the Paho C library.

https://rust-lang-nursery.github.io/rust-bindgen/

Generating bindings each time you build the Rust crate is time consuming and uses a lot of resources. This is especially noticeable when building natively on a small target like an ARM board, or similar.

But each release of the the Rust crate is build against a specific version of the Paho C library, which means that for a specific target, the bindings never change from build to build. Therefore, we can create the bindings once for a target and then use them for a speedy build after that.

The crate comes with a number of pre-built bindings for several popular targets in: `paho-mqtt-sys/bindings`. These are files with names in the form:

```
bindings_paho_mqtt_c_<version>-<target>.rs
```

Some of these include:

```
bindings_paho_mqtt_c_1.3.2-x86_64-unknown-linux-gnu.rs
bindings_paho_mqtt_c_1.3.2-x86_64-pc-windows-msvc.rs
bindings_paho_mqtt_c_1.3.2-i686-pc-windows-msvc.rs
bindings_paho_mqtt_c_1.3.2-aarch64-unknown-linux-gnu.rs
bindings_paho_mqtt_c_1.3.2-armv7-unknown-linux-gnueabihf.rs
bindings_paho_mqtt_c_1.3.2-default-32.rs
bindings_paho_mqtt_c_1.3.2-default-64.rs
```

Bidings can be created for new versions of the Paho C library or for different target platforms using the command-line _bindgen_ tool. For example on an x86 version of Windows using MSVC, you can re-generate the bindings like this:

```
$ cd paho-mqtt-sys
$ bindgen wrapper.h -o bindings/bindings_paho_mqtt_c_1.3.2-x86_64-pc-windows-msvc.rs -- -Ipaho.mqtt.c/src
```

To create bindings for a different target, use the _TARGET_ environment variable. For example, to build the 32-bit MSVC bindings for Windows on a 64-bit host, use the _i686-pc-windows-msvc_ target:

```
$ TARGET=i686-pc-windows-msvc bindgen wrapper.h -o bindings/bindings_paho_mqtt_c_1.3.2-i686-pc-windows-msvc.rs -- -Ipaho.mqtt.c/src
```

##### Bindgen linker issue

Bindgen requires a relatively recent version of the Clang library installed on the system - recommended v3.9 or later. The bindgen dependencies seem, however, to seek out the oldest Clang version if multiple ones are installed on the system. On Ubuntu 14.04 or 16.04, the Clang v3.6 default might give some problems, although as the Paho builder is currently configured, it should work.

But the safest thing would be to set the `LIBCLANG_PATH` environment variable to point to a supported version, like:
```
export LIBCLANG_PATH=/usr/lib/llvm-3.9/lib
```

### Cross-Compiling

I was pleasently surprised to discover that the *cmake* crate automatically handles cross-compiling libraries. You'll need a C cross-compiler installed on your system. See here for more info about cross-compiling Rust, in general: 

https://github.com/japaric/rust-cross

For example, to do a full build for `ARMv7`, which includes Raspberry Pi's, BeagleBones, UDOO Neo's, and lots of other ARM maker boards:

```
$ cargo build --target=armv7-unknown-linux-gnueabihf --examples
```

This builds the main crate, the *-sys* crate, and it cross-compiles the Paho C library. It uses SSL, so it requires you to have a version of the SSL development library installed with the cross-compiler.

If you don't have SSL for the cross-compiler
```
$ cargo build --target=armv7-unknown-linux-gnueabihf --no-default-features \
    --features="bundled" --examples
```

## Logging

The Rust library uses the `log` crate to output debug and trace information. Applications can choose to use one of the available logger implementations or define one of their own. More information is available at:

https://docs.rs/log/0.4.0/log/

The sample applications use the enviroment log crate, `env_logger` to configure output via the `RUST_LOG` environment variable. To use this, the following call is specified in the samples before using any of the Rust MQTT API:

```
env_logger::init().unwrap();
```

And then the library will output information as defined by the environment. Use like:

```
$ RUST_LOG=debug ./async_publish
DEBUG:paho_mqtt::async_client: Creating client with persistence: 0, 0x0
DEBUG:paho_mqtt::async_client: AsyncClient handle: 0x7f9ae2eab004
DEBUG:paho_mqtt::async_client: Connecting handle: 0x7f9ae2eab004
...
```

In addition, the underlying Paho C library has its own logging capabilities which can be used to trace network and protocol transactions. It is configured by the environment variables `MQTT_C_CLIENT_TRACE` and `MQTT_C_CLIENT_TRACE_LEVEL`. The former names the log file, with the special value "ON" to log to stdout. The latter specifies one of the levels: ERROR, PROTOCOL, MINIMUM, MEDIUM and MAXIMUM.

```
export MQTT_C_CLIENT_TRACE=ON
export MQTT_C_CLIENT_TRACE_LEVEL=PROTOCOL
```

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

