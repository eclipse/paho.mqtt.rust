# Eclipse Paho MQTT Rust Client Library

This repository contains the source code for the [Eclipse Paho](http://eclipse.org/paho) MQTT Rust client library on memory-managed operating systems such as Linux/Posix and Windows.

## Release notes

The Rust crate is a safe wrapper around the Paho C Library. This version is **specifically matched to Paho C v 1.3.x**, and is currently being tested with version 1.3.0. It will not build against newer versions of the C library, as the C lib expands functionality by extending structures, thus breaking the Rust build.

This is a pre-release version of the library, but has good feature coverage for an MQTT 3.1.1 library, and doesn't have a lot of outstanding issues.

The API is still under development, and there will likely be some minor breaking changes in the next few releases, but major redevelopment of the API is slowing down and approaching stability/

Most development and deployment has being done on Linux. Please let us know about any success or failure on other systems.

## Latest News

To keep up with the latest announcements for this project, follow:

**Twitter:** [@eclipsepaho](https://twitter.com/eclipsepaho) and [@fmpagliughi](https://twitter.com/fmpagliughi)

**EMail:** [Eclipse Paho Mailing List](https://accounts.eclipse.org/mailing-list/paho-dev)

**Mattermost:** [Eclipse Mattermost Paho Channel](https://mattermost.eclipse.org/eclipse/channels/paho)

### Unreleased Features (in this branch)

Development is proceeding to add support for Futures and clean up the internal implementation of the library. The following is already checked into this branch:

- **Futures support:**
    - Compatible with the [Rust Futures](https://docs.rs/futures/0.1.25/futures/) library v0.1
    - The `Token` objects, which are returned by asynchronous calls, now implements the `Futures` trait, which is _mostly_ compatible with the previous implementation.
    - Incoming messages can be obtained through a `Stream` from the client, implemented with a futures channel.
    - New examples of a publisher and subscriber implemented with futures.

- **Server Responses**
    - There are now several different types of tokens corresponding to different requests for which the server can return a response: _ConnectToken_, _DeliveryToken_, _SubscribeToken_, etc. 
    - Tokens now track the type of request and get the server response upon completion. This is the Futures _Item_ type for the token.
    - In particular this is useful for connecting subscribers. The app can now determine if a persistent session is already present, and only needs to subscribe if not.
    
- **Send and Sync Traits**
    - The clients are now marked as _Send_ and _Sync_
    - The _Token_ types are _Send_
    - Most of the option types are _Send_ and _Sync_
    
- **Internal Cleanup**
    - Moved `Tokens` into their own source file.
    - Consolidated persistence internals into `UserPersistence` struct.
    - Created a new `ResponseOptions` struct to manage the details of the C `MQTTAsync_responseOptions` objects.
    - Cleanup of the `AsyncClient` implementation.
    - A bad reconnect bug is fixed (Issue #33)

### Features

The initial version of the library is a wrapper for the Paho C library, similar to the implementation for the current Paho C++ library. It targets MQTT v3.1 and 3.1.1, and includes all of the features available in the C library for those versions, including:

- Standard TCP support
- SSL / TLS
- WebSockets
- QoS 0, 1, and 2
- Last Will and Testament (LWT)
- Message Persistence 
    - File or memory persistence
    - User-defined persistence
- Automatic Reconnect
- Offline Buffering
- High Availability
- Rust Futures and Streams for asynchronous operations.
- Traditional asynchronous API
- Synchronous/blocking  API

Supports Paho C v1.3.0

### Upcoming Release(s)

As soon as the current version is stabilized and released, work will immediately begin to bring in support for MQTT v5.

Prior to that, the plan is to allow the Futures support to settle, improve error handling and reporting, and clean up the internal implementation.

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

The Paho Rust crate is a wrapper around the Paho C library. The project includes a Rust _-sys_ crate, called _paho-mqtt-sys_, which provides unsafe bindings to the C library.  The repository contains a Git submodule pointing to the specific version of the C library that the Rust crate requires, and by default, it will automatically build and link to that library, using pre-generated C bindings that are also included in the repo.

When building, the user has several options:

 - Build the bundled library using the pre-generated bindings and SSL (default).
 - Build the bundled library, but regenerate the bindings at build time.
 - Use an external library, with the location specified by environment variables, generating the bindings at build time.
 - Use the pre-installed library with the pre-generated bindings.

These are chosen with cargo features, explained below.

Currently the Rust library is only linking to the SSL version of the library, _libpaho-mqtt3as_.

#### Building the bundled Paho C library

This is the default:

    $ cargo build
    
This will initialize and update the C library sources from Git, then use the _cmake_ crate to build the static version of the C library, and link it in. By default, the build will use the pre-generated bindings in _bindings/bindings_paho_mqtt_X_Y_Z.rs_, where _X_Y_Z_ is the currently supported library version.

When building the bundled libraries, the bindings can also be regenerated at build-time. This is especially useful when building on uncommon/untested platforms to ensure proper bindings for that system. This is done using the "buildtime_bindgen" feature:

    $ cargo build --features "build_bindgen"
    
In this case it will generate bindings based on the header files in the bundled C repository,

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

In this case, 
#### Linking to an installed Paho C library

If the correct version of the Paho C library is expected to be installed on the target system, the simplest solution is to use the pre-generated bindings and specify a link to the shared paho C library. 

    $ cargo build --no-default-features --features "ssl"

This is especially useful in a production environment where the system is well controlled, such as  when working with full-system build tools like _yocto_ or _buildroot_. It could be easier to build or cross-compile the packages separately.

Again, the "ssl" feature can be omitted if it is not desired.

This option should be used with caution when building an application that will ship independetly of the target system, since it assumes a _very specific_ version of the C library and will fail if that is not the one on the target.

#### Bindgen linker issue

The crate can optionally use the Rust _bindgen_ library to create the bindings to the Paho C library.

https://rust-lang-nursery.github.io/rust-bindgen/

Bindgen requires a relatively recent version of the Clang library installed on the system - recommended v3.9 or 4.0. The bindgen dependencies seem, however, to seek out the oldest Clang version if multiple ones are installed on the system. On Ubuntu 14.04 or 16.04, the Clang v3.6 default might give some problems, although as the Paho builder is currently configured, it should work.

But the safest thing would be to set the `LIBCLANG_PATH` environment variable to point to a supported version, like:
```
export LIBCLANG_PATH=/usr/lib/llvm-3.9/lib
```

### Cross-Compiling

I was pleasently surprised to discover that the *cmake* crate seems to automatically handle cross-compiling libraries. You'll need a C cross-compiler installed on your system. See here for more info about cross-compiling Rust, in general: 

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

Several small sample applications can be found in the _examples_ directory. Here is an example of a small MQTT publisher:

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
