// paho-mqtt/src/server_response.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.

/*******************************************************************************
 * Copyright (c) 2018-2019 Frank Pagliughi <fpagliughi@mindspring.com>
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

//! The Token module for the Paho MQTT Rust client library.
//!
//! Asynchronous operations return a `Token` that is a type of future. It
//! can be used to determine if an operation has completed, block and wait
//! for the operation to complete, and obtain the final result.
//! For example, you can start a connection, do something else, and then
//! wait for the connection to complete.
//!
//! The Token object implements the Future trait, and thus can be used and
//! combined with any other Rust futures.
//!

use std::ffi::CStr;

use ffi;

//use properties::{Properties};
//use errors;
//use errors::{MqttResult, MqttError};

/////////////////////////////////////////////////////////////////////////////
// ServerRequest

/// The server requests that expect a response.
/// This is required because the `alt` union of the MQTTAsync_successData
/// struct from C library doesn't indicate which field is valid.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ServerRequest {
    /// No response expected from the server
    None,
    /// Connecting to the server
    Connect,
    /// A subscription request of a single topic
    Subscribe,
    /// A subscription request of many topics
    SubscribeMany(usize),
}

impl Default for ServerRequest {
    fn default() -> Self { ServerRequest::None }
}

/////////////////////////////////////////////////////////////////////////////
// ServerResponse

/// The possible responses that may come back from the server, depending on
/// the type of request.
#[derive(Clone, PartialEq, Debug)]
pub enum ServerResponse {
    /// No response from the server
    None,
    /// The server URI, MQTT version, and whether the session is present
    Connect(String, i32, bool),
    /// The granted QoS of the subscription
    Subscribe(i32),
    /// The granted QoS of all the subscriptions
    SubscribeMany(Vec<i32>),
}

impl ServerResponse {
    pub fn new() -> Self { ServerResponse::default() }

    /// Creates the response object from the v3 "success" data structure
    /// sent by the C lib on completion of the operation.
    pub unsafe fn from_success(req: ServerRequest, rsp: &ffi::MQTTAsync_successData) -> Self {
        match req {
            ServerRequest::Connect => {
                ServerResponse::Connect(
                    CStr::from_ptr((*rsp).alt.connect.serverURI).to_string_lossy().to_string(),
                    rsp.alt.connect.MQTTVersion,
                    rsp.alt.connect.sessionPresent != 0
                )
            },
            ServerRequest::Subscribe => ServerResponse::Subscribe((*rsp).alt.qos),
            ServerRequest::SubscribeMany(n) => {
                let mut qosv = Vec::new();
                for i in 0..n {
                    qosv.push(*rsp.alt.qosList.offset(i as isize));
                }
                debug!("Subscribed to {} topics w/ Qos: {:?}", qosv.len(), qosv);
                ServerResponse::SubscribeMany(qosv)
            },
            _ => ServerResponse::None,
        }
    }

    /// Creates the response object from the v5 "success" data structure
    /// sent by the C lib on completion of the operation.
    pub unsafe fn from_success5(req: ServerRequest, rsp: &ffi::MQTTAsync_successData5) -> Self {
        match req {
            ServerRequest::Connect => {
                ServerResponse::Connect(
                    CStr::from_ptr(rsp.alt.connect.serverURI).to_string_lossy().to_string(),
                    rsp.alt.connect.MQTTVersion,
                    rsp.alt.connect.sessionPresent != 0
                )
            },
            // TODO: Get the correct QoS
            ServerRequest::Subscribe => ServerResponse::Subscribe(0 /*rsp.alt.qos*/),
            ServerRequest::SubscribeMany(n) => {
                let mut qosv = Vec::new();
                for _i in 0..n {
                    // TODO: Get the correct QoS values
                    qosv.push(0);   // *rsp.alt.qosList.offset(i as isize));
                }
                debug!("Subscribed to {} topics w/ Qos: {:?}", qosv.len(), qosv);
                ServerResponse::SubscribeMany(qosv)
            },
            _ => ServerResponse::None,
        }
    }
}

impl Default for ServerResponse {
    fn default() -> Self { ServerResponse::None }
}

