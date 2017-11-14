# Eclipse Paho MQTT Rust Client Library

This repository contains the source code for the [Eclipse Paho](http://eclipse.org/paho) MQTT Rust client library on memory-managed operating systems such as Linux/Posix and Windows.

## Incubator

This is the incubator branch for development and testing out an API for the Rust language.

_The API is likely to change repeatedly and often while the code is being developed in this branch. Use it with caution._

Initial development is being done on Linux. That is currently the only system known to work, although even that requires a few work-arounds as listed below.

It is hoped that a full, stable, version 1.0 release should be ready by early 2018.

## Features

The initial version of the library is a wrapper for the Paho C library, similar to the implementation for the current Paho C++ library. It will target MQTT v3.1 and 3.1.1, and should include all of the features available in the C library, including:

* Standard TCP support
* SSL / TLS
* Last Will and Testament (LWT)
* Message Persistence 
  * File or memory persistence
  * User-defined (coming soon)
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

There appears to be a bug in the _bindgen_ library in which it is outputting code that mangles the names of the C functions. This issue has been raised for the _bindgen_ project:
https://github.com/rust-lang-nursery/rust-bindgen/issues/1046

As a temporary solution, the _fixbindings.sh_ script can be run to fix the _bindings.rs_ file which is created by _bindgen. This works on Linux. On other systems, simply remove all of the lines in _bindings.rs_ which contain the text:
`#[link_name = ...]`

## Example

Several small sample applications can be found in the _examples_ directory. Here is an example of a small MQTT publisher:

```
extern crate paho_mqtt as mqtt;

fn main() {
    // Create a client & define connect options
    let mut cli = mqtt::AsyncClient::new("tcp://localhost:1883", "");

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        println!("Unable to connect:\n\t{:?}", e);
        ::std::process::exit(1);
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
