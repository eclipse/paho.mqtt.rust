# Eclipse Paho MQTT Rust Client Library

This repository contains the source code for the [Eclipse Paho](http://eclipse.org/paho) MQTT Rust client library on memory-managed operating systems such as Linux/Posix and Windows.

## Pre-release notes

This is a pre-release version of the library for the development and testing of an MQTT API for the Rust language.

_The API is guaranteed to change repeatedly and often while the code is being developed prior to a formal release. Use it with caution._

Initial development is being done on Linux. That is currently the only system known to work.

It is hoped that a full, stable, version 1.0 release should be ready by mid-2018.

## Features

The initial version of the library is a wrapper for the Paho C library, similar to the implementation for the current Paho C++ library. It will target MQTT v3.1 and 3.1.1, and will include all of the features available in the C library, including:

* Standard TCP support
* SSL / TLS
* Last Will and Testament (LWT)
* Message Persistence 
  * File or memory persistence
  * User-defined
* Automatic Reconnect
* Offline Buffering
* High Availability
* Asynchronous (Non-blocking) API
* Synchronous (Blocking)  API's

## Building the Crate

The library is a standard Rust "crate" using the _Cargo_ build tool. It uses the standard cargo commands for building:

`$ cargo build`
Builds the library complete with C bindings

`$ cargo build --examples`
Builds the sample applications in the _examples_ subdirectory.

`$ cargo test`
Builds and runs the unit tests.

`$ cargo doc`
Generates reference documentation.

### Paho C Library

The Paho Rust library is a wrapper around the Paho C library, and needs to locate both the pre-built library file and the C headers, particularly _MQTTAsync.h_.

Currently the Rust library is only linking to the SSL version of the library, _libpaho-mqtt3as_.

If the C library is not installed in a default system location, then the path to the headers and library must be specified as:

`PAHO_MQTT_C_INC_PATH= ...path to headers...`

`PAHO_MQTT_C_LIB_PATH= ...path to library...` 

### Bindgen linker issue

The crate currently uses the Rust _bindgen_ library to create the bindings to the Paho C library.
https://rust-lang-nursery.github.io/rust-bindgen/

Bindgen requires a recent version of the Clang library installed on the system - recommended v3.9 or 4.0. The bindgen dependencies seem, however, to seek out the oldest Clang version if multiple ones are installed on the system. On Ubuntu 14.04 or 16.04, the Clang v3.6 default might give some problems, although as the Paho builder is currently configured, it should work.

But the safest thing would be to set the `LIBCLANG_PATH` environment variable to point to a supported version, like:
```
export LIBCLANG_PATH=/usr/lib/llvm-3.9/lib
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
