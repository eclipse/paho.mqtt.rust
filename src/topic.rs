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
    errors::{Error, Result},
    message::{Message, MessageBuilder},
    properties::{Properties, PropertyCode},
    subscribe_options::SubscribeOptions,
    token::{DeliveryToken, Token},
};
use std::fmt;

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

impl<'a> Topic<'a> {
    /// Creates a new topic object for publishing messages.
    ///
    /// # Arguments
    ///
    /// `cli` The client used to publish the messages.
    /// `topic` The topic on which to publish the messages
    /// `qos` The quality of service for messages
    ///
    pub fn new<T>(cli: &'a AsyncClient, topic: T, qos: i32) -> Topic<'a>
    where
        T: Into<String>,
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
    where
        T: Into<String>,
    {
        Topic {
            cli,
            topic: topic.into(),
            qos,
            retained: true,
            alias: 0,
        }
    }

    /// Create a message for the topic using the supplied payload
    fn create_message<V>(&self, payload: V) -> Message
    where
        V: Into<Vec<u8>>,
    {
        // OPTIMIZE: This could be more efficient.
        if self.alias == 0 {
            Message::new(&self.topic, payload, self.qos)
        }
        else {
            let props = properties! { PropertyCode::TopicAlias => self.alias };
            MessageBuilder::new()
                .topic("")
                .payload(payload)
                .qos(self.qos)
                .retained(self.retained)
                .properties(props)
                .finalize()
        }
    }

    /// Subscribe to the topic.
    pub fn subscribe(&self) -> Token {
        self.cli.subscribe(self.topic.clone(), self.qos)
    }

    /// Subscribe to the topic with subscription options.
    pub fn subscribe_with_options<T, P>(&self, opts: T, props: P) -> Token
    where
        T: Into<SubscribeOptions>,
        P: Into<Option<Properties>>,
    {
        self.cli
            .subscribe_with_options(self.topic.clone(), self.qos, opts, props)
    }

    /// Publish a message on the topic.
    ///
    /// If a topic alias was previously sent, this will use the integer alias
    /// property instead of sending the topic string.
    /// Topic aliases are only applicable for MQTT v5 connections.
    ///
    /// # Arguments
    ///
    /// `payload` The payload of the message
    ///
    pub fn publish<V>(&self, payload: V) -> DeliveryToken
    where
        V: Into<Vec<u8>>,
    {
        let msg = self.create_message(payload);
        self.cli.publish(msg)
    }

    /// Attempts to publish a message on the topic, but returns an error
    /// immediately if there's a problem creating or queuing the message for
    /// transmission.
    ///
    /// If a topic alias was previously sent, this will use the integer alias
    /// property instead of sending the topic string.
    /// Topic aliases are only applicable for MQTT v5 connections.
    ///
    /// # Arguments
    ///
    /// `payload` The payload of the message
    ///
    /// Returns a Publish Error containing the complete message on failure.
    pub fn try_publish<V>(&self, payload: V) -> Result<DeliveryToken>
    where
        V: Into<Vec<u8>>,
    {
        let msg = self.create_message(payload);
        self.cli.try_publish(msg)
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
    where
        V: Into<Vec<u8>>,
    {
        self.alias = alias;
        self.publish(payload)
    }

    /// Attempts to publish a message on the topic using and setting a new topic
    /// alias, but returns an error immediately if there's a problem creating or
    /// queuing the message for transmission.
    ///
    /// See `publish_with_alias()` for more information.
    ///
    /// Returns a Publish Error containing the complete message on failure.
    pub fn try_publish_with_alias<V>(&mut self, alias: u16, payload: V) -> Result<DeliveryToken>
    where
        V: Into<Vec<u8>>,
    {
        self.alias = alias;
        self.try_publish(payload)
    }
}

/////////////////////////////////////////////////////////////////////////////

/// A topic filter.
///
/// An MQTT topic filter is a multi-field string, delimited by forward
/// slashes, '/', in which fields can contain the wildcards:
///     '+' - Matches a single field
///     '#' - Matches all subsequent fields (must be last field in filter)
///
/// It can be used to match against topics.
#[derive(Debug)]
pub enum TopicFilter {
    // If there are no wildcards, the filter is a straight topic string
    Topic(String),
    // If there are wildcards, the filter is split by fields.
    Fields(Vec<String>),
}

impl TopicFilter {
    /// Creates a new topic filter from the string.
    /// This can fail if the filter is not correct, such as having a '#'
    /// wildcard in anyplace other than the last field, or if
    pub fn new<S>(filter: S) -> Result<Self>
    where
        S: Into<String>,
    {
        let filter = filter.into();
        let n = filter.len();

        if n == 0 {
            Err(Error::BadTopicFilter)?;
        }

        // If the topic contains any wildcards.
        let wild = match filter.find('#') {
            Some(i) if i < n - 1 => Err(Error::BadTopicFilter)?,
            Some(_) => true,
            None => filter.contains('+'),
        };

        let v = if wild {
            let fields = filter.split('/').map(|s| s.to_string()).collect();
            Self::Fields(fields)
        }
        else {
            Self::Topic(filter)
        };

        Ok(v)
    }

    /// Determines if the topic matches the filter.
    pub fn is_match(&self, topic: &str) -> bool {
        match self {
            Self::Topic(filter) => topic == filter,
            Self::Fields(fields) => {
                let n = fields.len();
                let top_fields: Vec<_> = topic.split('/').collect();

                if n > top_fields.len() {
                    false
                }
                else {
                    for i in 0..n {
                        if fields[i] == "#" {
                            break;
                        }
                        if fields[i] != "+" && fields[i] != top_fields[i] {
                            return false;
                        }
                    }
                    true
                }
            }
        }
    }
}

impl fmt::Display for TopicFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Topic(filter) => write!(f, "{}", filter),
            // OPTIIMIZE: Do the individual writes, not join
            Self::Fields(fields) => write!(f, "{}", fields.join("/")),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonwild_topic_filter() {
        const FILTER: &str = "some/topic";

        let filter = TopicFilter::new(FILTER).unwrap();
        assert!(filter.is_match(FILTER));

        let s = format!("{}", filter);
        assert_eq!(s, FILTER);
    }

    #[test]
    fn test_topic_filter() {
        const FILTER1: &str = "some/topic/#";

        let filter = TopicFilter::new(FILTER1).unwrap();
        assert!(filter.is_match("some/topic/thing"));

        let s = format!("{}", filter);
        assert_eq!(s, FILTER1);

        const FILTER2: &str = "some/+/thing";
        let filter = TopicFilter::new(FILTER2).unwrap();
        assert!(filter.is_match("some/topic/thing"));

        let s = format!("{}", filter);
        assert_eq!(s, FILTER2);
    }
}
