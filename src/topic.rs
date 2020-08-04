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

use crate::{
    async_client::AsyncClient,
    token::{
        Token,
        DeliveryToken,
    },
    subscribe_options::SubscribeOptions,
    message::{Message, MessageBuilder},
    properties::PropertyCode,
};

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
    /// The topic alias.
    /// If this is non-zero is it sent in place of the string topic name.
    alias: u16,
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
            alias: 0,
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
            alias: 0,
        }
    }

    /// Subscribe to the topic.
    pub fn subscribe(&self) -> Token {
        self.cli.subscribe(self.topic.clone(), self.qos)
    }

    /// Subscribe to the topic with subscription options.
    pub fn subscribe_with_options<T>(&self, opts: T) -> Token
        where T: Into<SubscribeOptions>
    {
        self.cli.subscribe_with_options(self.topic.clone(), self.qos, opts)
    }

    /// Publish a message on the topic.
    ///
    /// If a topic alias was previously sent, this will use the integer alias
    /// property instead of sending the topic string.
    /// Topis aliases are only applicable for MQTT v5 connections.
    ///
    /// # Arguments
    ///
    /// `payload` The payload of the message
    ///
    pub fn publish<V>(&self, payload: V) -> DeliveryToken
        where V: Into<Vec<u8>>
    {
        // OPTIMIZE: This could be more efficient.
        let msg = if self.alias == 0 {
            Message::new(&self.topic, payload, self.qos)
        }
        else {
            let props = properties!{ PropertyCode::TopicAlias => self.alias };
            MessageBuilder::new()
                .topic("")
                .payload(payload)
                .qos(self.qos)
                .properties(props)
                .finalize()
        };
        self.cli.publish(msg)
    }

    /// Publish a message with a topic alias.
    ///
    /// This publishes the message with a topic alias property to set the
    /// alias at the broker. After calling this, the object keeps the
    /// alias and uses it for subsequent publishes instead of sending the
    /// full topic string.
    ///
    /// Note that using an alias is only valid with an MQTT v5 connection,
    /// and the value must be in the range of 1 - TopicAliasMaximum as the
    /// broker reported in the CONNACK packet. The alias is only valid
    /// for a single connection. It should be reset on a reconnect.
    ///
    /// This can be called a second time to change the alias setting.
    /// Using an alias of zero on a subsequent call instructs this object to
    /// stop using the alias and publish with the topic name.
    ///
    /// # Arguments
    ///
    /// `alias` The integer alias to use for subsequent message publishing.
    ///     This must be in the range 1 - `TopicAliasMaximum` as reported by
    ///     the server in the CONNACK package. Using a value of zero
    ///     instructs this object to stop using the alias and go back to
    ///     publishing with the string topic name.
    /// `payload` The payload of the message
    ///
    pub fn publish_with_alias<V>(&mut self, alias: u16, payload: V) -> DeliveryToken
        where V: Into<Vec<u8>>
    {
        self.alias = alias;
        let msg = if alias == 0 {
            Message::new(&self.topic, payload, self.qos)
        }
        else {
            let props = properties!{ PropertyCode::TopicAlias => alias };
            MessageBuilder::new()
                .topic(self.topic.clone())
                .payload(payload)
                .qos(self.qos)
                .properties(props)
                .finalize()
        };
        self.cli.publish(msg)
    }
}

