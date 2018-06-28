// lib.rs
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017-2018 Frank Pagliughi <fpagliughi@mindspring.com>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v10.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

//! This is the Eclipse Paho MQTT client library for the Rust language.
//!

// TODO: Uncomment this and build to check the documentation coverage
//#![deny(missing_docs)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Temporary
#![allow(dead_code)]

#[macro_use]
extern crate log;

extern crate paho_mqtt3as_sys as ffi;

pub use async_client::*;        //{AsyncClient, AsyncClientBuilder};
pub use client::*;              //{Client, ClientBuilder};
pub use create_options::*;      //{CreateOptions, CreateOptionsBuilder};
pub use connect_options::*;     //{ConnectOptions, ConnectOptionsBuilder, MQTT_VERSION_3_1_1, ...};
pub use will_options::*;        //{WillOptions, WillOptionsBuilder};
pub use ssl_options::*;         //{SslOptions, SslOptionsBuilder};
pub use disconnect_options::*;  //{DisconnectOptions, DisconnectOptionsBuilder};
pub use message::*;             //{Message, MessageBuilder};
pub use topic::*;               //{Topic}
pub use client_persistence::*;
pub use errors::*;              //{MqttResult, MqttError, ErrorKind};

//pub mod mqtt;
mod macros;

/// The asynchronous API
pub mod async_client;

/// The synchronous API
pub mod client;

/// Client creation options
pub mod create_options;

/// Options for connecting to the server.
pub mod connect_options;

/// Connect options for the Last Will and Testament (LWT) message.
pub mod will_options;

/// Connect options for creating secure connections to the server.
pub mod ssl_options;

/// Options for disconnecting from the server.
pub mod disconnect_options;

/// The message object
pub mod message;

/// Options for creating topic objects that are associated with a
/// particular server.
pub mod topic;

/// Definitions for creating user-defined persistence.
pub mod client_persistence;

/// The MQTT errors
pub mod errors;

/// Utility for creating string collections (to pass to the C library).
pub mod string_collection;

#[cfg(test)]
mod tests {
}

