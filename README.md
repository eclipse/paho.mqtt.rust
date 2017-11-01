# Eclipse Paho MQTT Rust Client Library

This repository will soon contain the source code for the [Eclipse Paho](http://eclipse.org/paho) MQTT Rust client library on memory-managed operating systems such as Linux/Posix and Windows.

## Coming Soon!

The initial version of the library will be a wrapper for the Paho C library, similar to the implementation for the current Paho C++ library. It will target MQTT v3.1 and 3.1.1, and should include all of the features available in the C library, including:

* Standard TCP support
* SSL / TLS
* Last Will and Testament (LWT)
* Message Persistence
* Automatic Reconnect
* Offline Buffering
* High Availability
* Blocking and Non-blocking API's

Initial, highly-unstable code will start to appear this month (Nov 2017) in an _incubator_ Git branch. A full 1.0 release should be ready by early 2018.