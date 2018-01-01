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

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Temporary
#![allow(dead_code)]

#[macro_use]
extern crate log;

extern crate paho_mqtt3as_sys as ffi;

pub use async_client::*;		//{AsyncClient, AsyncClientBuilder};
pub use client::*;				//{Client, ClientBuilder};
pub use create_options::*;		//{CreateOptions, CreateOptionsBuilder};
pub use connect_options::*;		//{ConnectOptions, ConnectOptionsBuilder, MQTT_VERSION_3_1_1, ...};
pub use will_options::*;		//{WillOptions, WillOptionsBuilder};
pub use ssl_options::*;			//{SslOptions, SslOptionsBuilder};
pub use disconnect_options::*;	//{DisconnectOptions, DisconnectOptionsBuilder};
pub use message::*;				//{Message, MessageBuilder};
pub use topic::*;				//{Topic}
pub use client_persistence::*;
pub use errors::*;				//{MqttResult, MqttError, ErrorKind};

//pub mod mqtt;
mod macros;

pub mod async_client;
pub mod client;
pub mod create_options;
pub mod connect_options;
pub mod will_options;
pub mod ssl_options;
pub mod disconnect_options;
pub mod message;
pub mod topic;
pub mod client_persistence;
pub mod errors;
pub mod string_collection;

#[cfg(test)]
mod tests {
}

