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

use properties::{Properties};
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
    /// A subscription request (possibly multiple items)
    Subscribe,
    SubscribeMany(usize),
    /// An un subscribe request (possibly multiple items)
    Unsubscribe,
    UnsubscribeMany(usize),
}

impl Default for ServerRequest {
    fn default() -> Self { ServerRequest::None }
}

/////////////////////////////////////////////////////////////////////////////
// ServerResponse

/// The possible responses that may come back from the server, depending on
/// the type of request.
#[derive(Clone, PartialEq, Debug)]
pub enum CommandResponse {
    /// No response from the server
    None,
    /// The server URI, MQTT version, and whether the session is present
    Connect(String, i32, bool),
    /// The granted QoS of the subscription
    Subscribe(i32),
    /// The granted QoS of all the subscriptions
    SubscribeMany(Vec<i32>),
}

impl Default for CommandResponse {
    fn default() -> Self { CommandResponse::None }
}

#[derive(Clone, Debug)]
pub struct ServerResponse {
    /// The command-specific responses
    rsp: CommandResponse,
    /// MQTT v5 Properties
    props: Properties,
}


impl ServerResponse {
    pub fn new() -> Self { ServerResponse::default() }

    /// Creates the response object from the v3 "success" data structure
    /// sent by the C lib on completion of the operation.
    pub unsafe fn from_success(req: ServerRequest, rsp: &ffi::MQTTAsync_successData) -> Self {
        let rsp = match req {
            ServerRequest::Connect => {
                CommandResponse::Connect(
                    CStr::from_ptr(rsp.alt.connect.serverURI).to_string_lossy().to_string(),
                    rsp.alt.connect.MQTTVersion,
                    rsp.alt.connect.sessionPresent != 0
                )
            },
            ServerRequest::Subscribe => CommandResponse::Subscribe(rsp.alt.qos),
            ServerRequest::SubscribeMany(n) => {
                let mut qosv = Vec::new();
                if n == 1 { // TODO: Make sure this is right
                    qosv.push(rsp.alt.qos);
                }
                else {
                    for i in 0..n {
                        qosv.push(*rsp.alt.qosList.offset(i as isize));
                    }
                }
                debug!("Subscribed to {} topics w/ Qos: {:?}", qosv.len(), qosv);
                CommandResponse::SubscribeMany(qosv)
            },
            _ => CommandResponse::None,
        };
        ServerResponse {
            rsp,
            props: Properties::new(),
        }
    }

    /// Creates the response object from the v5 "success" data structure
    /// sent by the C lib on completion of the operation.
    pub unsafe fn from_success5(req: ServerRequest, rsp: &ffi::MQTTAsync_successData5) -> Self {
        let props = Properties::from_c_struct(&rsp.properties);
        //println!("Properties: {:?}", props);
        let rsp = match req {
            ServerRequest::Connect => {
                CommandResponse::Connect(
                    CStr::from_ptr(rsp.alt.connect.serverURI).to_string_lossy().to_string(),
                    rsp.alt.connect.MQTTVersion,
                    rsp.alt.connect.sessionPresent != 0
                )
            },
            ServerRequest::Subscribe => CommandResponse::Subscribe(rsp.reasonCode as i32),
            ServerRequest::SubscribeMany(n) => {
                let mut qosv = Vec::new();
                if n == 1 {
                    qosv.push(rsp.reasonCode as i32);
                }
                else {
                    for i in 0..n {
                        qosv.push(rsp.alt.sub.reasonCodes.offset(i as isize) as i32);
                    }
                }
                debug!("Subscribed to {} topics w/ Qos: {:?}", qosv.len(), qosv);
                CommandResponse::SubscribeMany(qosv)
            },
            _ => CommandResponse::None,
        };
        ServerResponse {
            rsp,
            props,
        }
    }

    pub fn command_response(&self) -> &CommandResponse {
        &self.rsp
    }

    pub fn connect_response(&self) -> Option<(String, i32, bool)> {
        match &self.rsp {
            CommandResponse::Connect(uri, ver, session) =>
                Some((uri.clone(), *ver, *session)),
            _ => None,
        }
    }

    pub fn properties(&self) -> &Properties {
        &self.props
    }
}


impl Default for ServerResponse {
    fn default() -> Self {
        ServerResponse {
            rsp: CommandResponse::default(),
            props: Properties::default(),
        }
    }
}
