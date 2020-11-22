// lib.rs
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017-2020 Frank Pagliughi <fpagliughi@mindspring.com>
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

#[macro_use] extern crate log;
extern crate paho_mqtt_sys as ffi;

pub use crate::async_client::*;        //{AsyncClient, AsyncClientBuilder};
pub use crate::client::*;              //{Client, ClientBuilder};
pub use crate::create_options::*;      //{CreateOptions, CreateOptionsBuilder};
pub use crate::connect_options::*;     //{ConnectOptions, ConnectOptionsBuilder, MQTT_VERSION_3_1_1, ...};
pub use crate::will_options::*;        //{WillOptions, WillOptionsBuilder};
pub use crate::ssl_options::*;         //{SslOptions, SslOptionsBuilder};
pub use crate::disconnect_options::*;  //{DisconnectOptions, DisconnectOptionsBuilder};
pub use crate::subscribe_options::*;   //{SubscribeOptions};
pub use crate::response_options::*;    //{ResponseOptions};
pub use crate::server_response::*;     //{ServerResponse, CommandResponse};
pub use crate::properties::*;          //{Property, Properties};
pub use crate::message::*;             //{Message, MessageBuilder};
pub use crate::name_value::*;          //{NameValueCollection};
pub use crate::token::*;               //{Token}
pub use crate::topic::*;               //{Topic}
pub use crate::reason_code::*;         //{ReasonCode}
pub use crate::types::*;               //...
pub use crate::client_persistence::*;
pub use crate::errors::*;              //{Result, Error, ErrorKind};

use std::{
    os::raw::c_int,
    any::Any,
};

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

/// Options for subscribing to topics
pub mod subscribe_options;

/// Options for responses coming back from the C lib.
pub mod response_options;

/// Responses coming back from the server.
pub mod server_response;

/// MQTT 5v properties.
pub mod properties;

/// The message object
pub mod message;

/// Tokens to monitor asynchronous operations
pub mod token;

/// Options for creating topic objects that are associated with a
/// particular server.
pub mod topic;

/// MQTT v5 Reason Codes
pub mod reason_code;

/// Miscelaneous types
pub mod types;

/// Definitions for creating user-defined persistence.
pub mod client_persistence;

/// The MQTT errors
pub mod errors;

/// Utility for creating string collections (to pass to the C library).
pub mod string_collection;

/// Utility for creating name/value string pair collections
/// (to pass to the C library).
pub mod name_value;

// --------------------------------------------------------------------------

/// Generic type for arbitrary user-supplied data.
///
/// The application can use a type compatible with this to store in the
/// client as "user data" to be accessed from callbacks, etc.
pub type UserData = Box<dyn Any + 'static + Send + Sync>;


// --------------------------------------------------------------------------

/// Convert a Rust bool to a Paho C boolean
pub fn to_c_bool(on: bool) -> c_int {
    if on { 1 } else { 0 }
}

/// Converts a C integer boolean to a Rust bool
pub fn from_c_bool(on: c_int) -> bool {
    on != 0
}
