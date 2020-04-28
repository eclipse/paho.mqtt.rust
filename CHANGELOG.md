# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.7.1](https://github.com/eclipse/paho.mqtt.rust/compare/v0.7..v0.7.1) - 2020-04-28

It turned out that the update to the -sys crate in v0.7 was a breaking change. This just bumps the version numbers to indicate that, so as not to break v0.6 builds via crates.io

## [v0.7](https://github.com/eclipse/paho.mqtt.rust/compare/v0.6..v0.7) - 2020-04-27

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


## [v0.6](https://github.com/eclipse/paho.mqtt.rust/compare/v0.5..v0.6) - 2019-10-12

The v0.6 release added support for Futures and cleaned up the internal implementation of the library. 

- **Futures support:**
    - Compatible with the [Rust Futures](https://docs.rs/futures/0.1.25/futures/) library v0.1
    - Now depends on the crates "futures" (v0.1) and "futures-timer" (v0.1).
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
    - _AsyncClient_ and _Token_ objects are now just _Arc_ wrappers around inner structs making it easy to clone and pass references around.
    
- **Internal Cleanup**
    - Updated to wrap Paho C v1.3.1 which has a number of important bug fixes.
    - Moved `Tokens` into their own source file.
    - Consolidated persistence internals into `UserPersistence` struct.
    - Created a new `ResponseOptions` struct to manage the details of the C `MQTTAsync_responseOptions` objects.
    - Cleanup of the `AsyncClient` implementation.
    - A bad reconnect bug is fixed (Issue #33)


## [v0.5](https://github.com/eclipse/paho.mqtt.rust/compare/v0.4..v0.5) - 2018-12-15

### Added

- WebSocket support (free with Paho C 1.3.0 update).
- Example apps can take server URI's from the command line.

### Changed

- Updated the library to bundle and use Paho C v1.3.0


