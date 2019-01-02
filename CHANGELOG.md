# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Support for Rust _Futures_ and _Streams_. 
    - `Token` struct now implements the Future trait.
    - Client can create a futures `Stream` for incoming messages.
- `ResponseOptions` struct added to manage the internals of managing the C library's MQTTAsync_responseOptions 

### Changed
- `Token` has pushed its data members down a level into a `TokenInner` structure, and now `Token` just has an `Arc<TokenInner>` member.
- `AsyncClient` asynchronous functions now return `Token` instead of `Arc<Token>`, like:
```
pub fn connect<T>(&self, opt_opts: T) -> Token { ... }
pub fn reconnect(&self) -> Token { ... }
pub fn disconnect<T>(&self, opt_opts: T) -> Token { ... }
pub fn publish(&self, msg: Message) -> DeliveryToken { ... }
```
- Cloning a `Token` just creates a new `Arc` pointer to the same `TokenInner` struct.
- `Token` callbacks now implement `Fn` instead of `FnMut`.
- `Token::wait()` and `Token::wait_for()` now consume the Token (i.e. they take `self` instead of `&self`).
- Internals of handling persistence updated. The `UserPersistence` struct replaces the `ClientPersistenceBridge`.

## [v0.5](https://github.com/eclipse/paho.mqtt.rust/compare/v0.4..v0.5) - 2018-12-15

### Added

- WebSocket support (free with Paho C 1.3.0 update).
- Example apps can take server URI's from the command line.

### Changed

- Updated the library to bundle and use Paho C v1.3.0


