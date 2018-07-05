// topic.rs
// 
// A set of message parameters to repeatedly publish to the same topic.
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

use std::sync::{Arc};

use async_client::{AsyncClient,DeliveryToken};
use message::Message;

/////////////////////////////////////////////////////////////////////////////
//                              Topic
/////////////////////////////////////////////////////////////////////////////

/// A topic destination for messages.
/// This keeps message parameters for repeatedly publishing to the same
/// topic on a server.
pub struct Topic<'a> {
    /// Reference to the broker that will receive the messages.
    cli: &'a AsyncClient,
    /// The topic on which to publish the messages.
    topic: String,
    /// The QoS level to publish the messages.
    qos: i32,
    /// Whether the last message should be retained by the broker.
    retained: bool,
}

impl<'a> Topic<'a> 
{
    /// Creates a new topic object for publishing messages.
    ///
    /// # Arguments
    ///
    /// `cli` The client used to publish the messages.
    /// `topic` The topic on which to publish the messages
    /// `qos` The quality of service for messages
    ///
    pub fn new<T>(cli: &'a AsyncClient, topic: T, qos: i32) -> Topic<'a>
        where T: Into<String>
    {
        Topic {
            cli,
            topic: topic.into(),
            qos,
            retained: false,
        }
    }

    /// Creates a new topic object for publishing messages.
    ///
    /// # Arguments
    ///
    /// `cli` The client used to publish the messages.
    /// `topic` The topic on which to publish the messages
    /// `qos` The quality of service for messages
    ///
    pub fn new_retained<T>(cli: &'a AsyncClient, topic: T, qos: i32) -> Topic<'a>
        where T: Into<String>
    {
        Topic {
            cli,
            topic: topic.into(),
            qos,
            retained: true,
        }
    }

    /// Publish a message on the topic.
    ///
    /// # Arguments
    ///
    /// `payload` The payload of the message
    ///
    pub fn publish<V>(&self, payload: V) -> Arc<DeliveryToken>
        where V: Into<Vec<u8>>
    {
        // OPTIMIZE: This could be more efficient.
        let msg = Message::new(self.topic.clone(), payload, self.qos);
        self.cli.publish(msg)
    }
}


