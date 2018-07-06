// will_options.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017 Frank Pagliughi <fpagliughi@mindspring.com>
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

//! Last Will and Testament (LWT) options for the Paho MQTT Rust client library.
//!

// TODO: We probably don't need the will options... at least not for the
// public API. This is simply a message. So the public API could be:
//
//   let lwt = Message::new(...);
//   let opts = ConnectOptionsBuilder::new().will_message(lwt).finalize();
//

use std::ptr;
use std::ffi::CString;
use std::os::raw::c_void;
use std::borrow::Cow;

use ffi;

use message::Message;

/// The options for the Last Will and Testament (LWT).
/// This defines a message that is registered with the the server at the time
/// of connection. Then if the connection is lost unexpectedly, the message
/// is published by the server.
///
/// The will options are somewhat redundant in that they simply represent a
/// message, albeit a special one, that is included in the connect options.
/// This structure may eventually be phased out, and therefore users are
/// encouraged to just create a `Message` object and use it when building
/// `ConnectOptions`:
/// ```
/// extern crate paho_mqtt as mqtt;
///
/// let lwt = mqtt::Message::new("lwt", "disconnected", 1);
/// let opts = mqtt::ConnectOptionsBuilder::new().will_message(lwt).finalize();
/// ```
///
#[derive(Debug)]
pub struct WillOptions {
    pub(crate) copts: ffi::MQTTAsync_willOptions,
    topic: CString,
    payload: Vec<u8>,
}

impl WillOptions {
    /// Creates a new WillOptions message.
    ///
    /// # Arguments
    ///
    /// * `topic` The topic on which the LWT message is to be published.
    /// * `payload` The binary payload of the LWT message
    /// * `qos` The quality of service for message delivery (0, 1, or 2)
    pub fn new<S,V>(topic: S, payload: V, qos: i32) -> WillOptions
        where S: Into<String>,
              V: Into<Vec<u8>>
    {
        let opts = WillOptions {
            copts: ffi::MQTTAsync_willOptions {
                qos,
                ..ffi::MQTTAsync_willOptions::default()
            },
            topic: CString::new(topic.into()).unwrap(),
            payload: payload.into(),
        };
        WillOptions::fixup(opts)
    }

    /// Creates a new WillOptions message with the 'retain' flag set.
    ///
    /// # Arguments
    ///
    /// * `topic` The topic on which the LWT message is to be published.
    /// * `payload` The binary payload of the LWT message
    /// * `qos` The quality of service for message delivery (0, 1, or 2)
    pub fn new_retained<S,V>(topic: S, payload: V, qos: i32) -> WillOptions
        where S: Into<String>,
              V: Into<Vec<u8>>
    {
        let opts = WillOptions {
            copts: ffi::MQTTAsync_willOptions {
                retained: 1,
                qos,
                ..ffi::MQTTAsync_willOptions::default()
            },
            topic: CString::new(topic.into()).unwrap(),
            payload: payload.into(),
        };
        WillOptions::fixup(opts)
    }

    // Updates the C struct from the cached topic and payload vars
    fn fixup(mut opts: WillOptions) -> WillOptions {
        opts.copts.topicName = if opts.topic.as_bytes().len() != 0 {
            opts.topic.as_ptr()
        }
        else {
            ptr::null()
        };
        opts.copts.payload.data = if opts.payload.len() != 0 {
            opts.payload.as_ptr() as *const c_void
        }
        else {
            ptr::null()
        };
        opts.copts.payload.len = opts.payload.len() as i32;
        opts
    }

    /// Gets the topic string for the LWT
    fn topic(&self) -> &str {
        self.topic.to_str().unwrap()
    }

    /// Gets the payload of the LWT
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Gets the payload of the message as a string.
    /// Note that this clones the payload.
    pub fn payload_str(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.payload)
    }

    /// Returns the Quality of Service (QOS) for the message.
    pub fn qos(&self) -> i32 {
        self.copts.qos
    }

    /// Gets the 'retained' flag for the message.
    pub fn retained(&self) -> bool {
        self.copts.retained != 0
    }
}

impl Default for WillOptions {
    /// Creates a WillOptions struct with default options
    fn default() -> WillOptions {
        let opts = WillOptions {
            copts: ffi::MQTTAsync_willOptions::default(),
            topic: CString::new("").unwrap(),
            payload: Vec::new(),
        };
        WillOptions::fixup(opts)
    }
}

impl Clone for WillOptions {
    /// Creates a clone of the WillOptions struct.
    /// This clones the cached values and updates the C struct to refer
    /// to them.
    fn clone(&self) -> WillOptions {
        let will = WillOptions {
            copts: self.copts.clone(),
            topic: self.topic.clone(),
            payload: self.payload.clone(),
        };
        WillOptions::fixup(will)
    }
}

impl From<Message> for WillOptions {
    /// Create `WillOptions` from a `Message`
    fn from(msg: Message) -> Self {
        let will = WillOptions {
            copts: ffi::MQTTAsync_willOptions {
                qos: msg.cmsg.qos,
                retained: msg.cmsg.retained,
                ..ffi::MQTTAsync_willOptions::default()
            },
            topic: msg.topic,
            payload: msg.payload,
        };
        WillOptions::fixup(will)
    }
}

/////////////////////////////////////////////////////////////////////////////
//                                  Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::{c_char};
    use message::MessageBuilder;

    // The C struct identifier for will options and the supported struct version.
    const STRUCT_ID: [c_char; 4] = [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'W' as c_char];
    const STRUCT_VERSION: i32 = 1;

    // These should differ from defaults.
    const TOPIC: &'static str = "test";
    const PAYLOAD: &'static [u8] = b"Hello world";
    const QOS: i32 = 2;
    const RETAINED: bool = true;

    // By convention our defaults should match the defaults of the C library
    #[test]
    fn test_default() {
        let opts = WillOptions::default();
        // Get default C options for comparison
        let copts = ffi::MQTTAsync_willOptions::default();

        // First, make sure C options valid
        assert_eq!(STRUCT_ID, copts.struct_id);
        assert_eq!(STRUCT_VERSION, copts.struct_version);
        assert_eq!(ptr::null(), copts.message);

        assert_eq!(copts.struct_id, opts.copts.struct_id);
        assert_eq!(copts.struct_version, opts.copts.struct_version);
        assert_eq!(copts.topicName, opts.copts.topicName);
        assert_eq!(copts.retained, opts.copts.retained);
        assert_eq!(copts.qos, opts.copts.qos);
        assert_eq!(copts.payload.len, opts.copts.payload.len);
        assert_eq!(copts.payload.data, opts.copts.payload.data);

        assert_eq!(&[] as &[u8], opts.topic.as_bytes());
        assert_eq!(&[] as &[u8], opts.payload.as_slice());
    }

    #[test]
    fn test_new() {
        let msg = WillOptions::new(TOPIC, PAYLOAD, QOS);

        assert_eq!(STRUCT_ID, msg.copts.struct_id);
        assert_eq!(STRUCT_VERSION, msg.copts.struct_version);

        assert_eq!(TOPIC, msg.topic.to_str().unwrap());
        assert_eq!(PAYLOAD, msg.payload.as_slice());

        assert_eq!(msg.payload.len() as i32, msg.copts.payload.len);
        assert_eq!(msg.payload.as_ptr() as *const c_void, msg.copts.payload.data);

        assert_eq!(QOS, msg.copts.qos);
        assert!(msg.copts.retained == 0);
    }

    #[test]
    fn test_new_retained() {
        let msg = WillOptions::new_retained(TOPIC, PAYLOAD, QOS);

        assert_eq!(STRUCT_ID, msg.copts.struct_id);
        assert_eq!(STRUCT_VERSION, msg.copts.struct_version);

        assert_eq!(TOPIC, msg.topic.to_str().unwrap());
        assert_eq!(PAYLOAD, msg.payload.as_slice());

        assert_eq!(msg.payload.len() as i32, msg.copts.payload.len);
        assert_eq!(msg.payload.as_ptr() as *const c_void, msg.copts.payload.data);

        assert_eq!(QOS, msg.copts.qos);
        assert!(msg.copts.retained != 0);
    }

    // Test creating will options from a message
    #[test]
    fn test_from_message() {
        let msg = MessageBuilder::new()
                        .topic(TOPIC)
                        .payload(PAYLOAD)
                        .qos(QOS)
                        .retained(RETAINED)
                        .finalize();

        let opts = WillOptions::from(msg);

        assert_eq!(TOPIC, opts.topic.to_str().unwrap());
        assert_eq!(PAYLOAD, opts.payload.as_slice());

        assert_eq!(opts.payload.len() as i32, opts.copts.payload.len);
        assert_eq!(opts.payload.as_ptr() as *const c_void, opts.copts.payload.data);

        assert_eq!(QOS, opts.copts.qos);
        assert!(opts.copts.retained != 0);
    }

    // Make sure assignment works properly
    // This primarily ensures that C pointers stay fixed to cached values,
    // and/or that the cached buffers don't move due to assignment.
    #[test]
    fn test_assign() {
        let msg = MessageBuilder::new()
                        .topic(TOPIC)
                        .payload(PAYLOAD)
                        .qos(QOS)
                        .retained(RETAINED)
                        .finalize();

        let org_opts = WillOptions::from(msg);
        let opts = org_opts;

        assert_eq!(TOPIC, opts.topic.to_str().unwrap());
        assert_eq!(PAYLOAD, opts.payload.as_slice());

        assert_eq!(opts.payload.len() as i32, opts.copts.payload.len);
        assert_eq!(opts.payload.as_ptr() as *const c_void, opts.copts.payload.data);

        assert_eq!(QOS, opts.copts.qos);
        assert!(opts.copts.retained != 0);
    }

    // Test that a clone works properly.
    // This ensures that the cached values are cloned and that the C pointers
    // in the new object point to those clones.
    #[test]
    fn test_clone() {
        let opts = {
            let msg = MessageBuilder::new()
                            .topic(TOPIC)
                            .payload(PAYLOAD)
                            .qos(QOS)
                            .retained(RETAINED)
                            .finalize();

            let org_opts = WillOptions::from(msg);
            org_opts.clone()
        };

        assert_eq!(TOPIC, opts.topic.to_str().unwrap());
        assert_eq!(PAYLOAD, opts.payload.as_slice());

        assert_eq!(opts.payload.len() as i32, opts.copts.payload.len);
        assert_eq!(opts.payload.as_ptr() as *const c_void, opts.copts.payload.data);

        assert_eq!(QOS, opts.copts.qos);
        assert!(opts.copts.retained != 0);
    }
}

