// paho-mqtt/src/client.rs
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017-2019 Frank Pagliughi <fpagliughi@mindspring.com>
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

//! This contains the synchronous `Client` interface for the Paho MQTT Rust
//! library.
//!
//! This is a simple convenience wrapper around the asynchronous API in which
//! each function calls the underlying async function, and then blocks waiting
//! for it to complete.
//!
//! The synchronous calls use a default timeout

use std::time::Duration;
use std::sync::mpsc;

use async_client::AsyncClient;
use create_options::CreateOptions;
use connect_options::ConnectOptions;
use disconnect_options::DisconnectOptions;
use server_response::ServerResponse;
use message::Message;
use errors::MqttResult;

/////////////////////////////////////////////////////////////////////////////
// Client

/// MQTT Client with a synchronous (blocking) API.
/// This is simply a convenience wrapper around the asynchronous API,
/// providing blocking calls with timeouts.
pub struct Client {
    /// The underlying asynchronous client.
    cli: AsyncClient,
    /// The default timeout for synchronous calls.
    timeout: Duration,
}

impl Client {
    /// Creates a new MQTT client which can connect to an MQTT broker.
    pub fn new<T>(opts: T) -> MqttResult<Client>
        where T: Into<CreateOptions>
    {
        let async_cli = AsyncClient::new(opts)?;

        let cli = Client {
            cli: async_cli,
            timeout: Duration::from_secs(5*60),
        };
        //cli.start_consuming();
        Ok(cli)
    }

    /// Gets the default timeout used for synchronous operations.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Sets the default timeout used for synchronous operations.
    ///
    /// ## Arguments
    ///
    ///  `timeout` The timeout to use for synchronous calls, like
    ///     connect(), disconnect(), publish(), etc.
    ///
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout
    }

    /// Connects to an MQTT broker using the specified connect options.
    pub fn connect<T>(&self, opt_opts:T) -> MqttResult<ServerResponse>
        where T: Into<Option<ConnectOptions>>
    {
        self.cli.connect(opt_opts).wait_for(self.timeout)
    }

    /// Disconnects from the MQTT broker.
    ///
    /// ## Arguments
    ///
    /// `opt_opts` Optional disconnect options. Specifying `None` will use
    ///            default of immediate (zero timeout) disconnect.
    ///
    pub fn disconnect<T>(&self, opt_opts:T) -> MqttResult<()>
        where T: Into<Option<DisconnectOptions>>
    {
        self.cli.disconnect(opt_opts).wait_for(self.timeout)?;
        Ok(())
    }

    /// Disconnect from the MQTT broker with a timeout.
    /// This will delay the disconnect for up to the specified timeout to
    /// allow in-flight messages to complete.
    /// This is the same as calling disconnect with options specifying a
    /// timeout.
    ///
    /// # Arguments
    ///
    /// `timeout` The amount of time to wait for the disconnect. This has
    ///           a resolution in milliseconds.
    ///
    pub fn disconnect_after(&self, timeout: Duration) -> MqttResult<()> {
        self.cli.disconnect_after(timeout).wait_for(self.timeout)?;
        Ok(())
    }

    /// Attempts to reconnect to the broker.
    /// This can only be called after a connection was initially made or
    /// attempted. It will retry with the same connect options.
    pub fn reconnect(&self) -> MqttResult<ServerResponse> {
        self.cli.reconnect().wait_for(self.timeout)
    }

    /// Determines if this client is currently connected to an MQTT broker.
    pub fn is_connected(&self) -> bool {
        self.cli.is_connected()
    }

    /// Publishes a message to an MQTT broker
    pub fn publish(&self, msg: Message) -> MqttResult<()> {
        self.cli.publish(msg).wait_for(self.timeout)
    }

    /// Subscribes to a single topic.
    ///
    /// # Arguments
    ///
    /// `topic` The topic name
    /// `qos` The quality of service requested for messages
    ///
    pub fn subscribe(&self, topic: &str, qos: i32) -> MqttResult<ServerResponse> {
        self.cli.subscribe(topic, qos).wait_for(self.timeout)
    }

    /// Subscribes to multiple topics simultaneously.
    ///
    /// # Arguments
    ///
    /// `topic` The topic name
    /// `qos` The quality of service requested for messages
    ///
    pub fn subscribe_many<T>(&self, topics: &[T], qos: &[i32]) -> MqttResult<ServerResponse>
        where T: AsRef<str>
    {
        self.cli.subscribe_many(topics, qos).wait_for(self.timeout)
    }

    /// Unsubscribes from a single topic.
    ///
    /// # Arguments
    ///
    /// `topic` The topic to unsubscribe. It must match a topic from a
    ///         previous subscribe.
    ///
    pub fn unsubscribe(&self, topic: &str) -> MqttResult<()> {
        self.cli.unsubscribe(topic).wait_for(self.timeout)?;
        Ok(())
    }

    /// Unsubscribes from multiple topics simultaneously.
    ///
    /// # Arguments
    ///
    /// `topic` The topics to unsubscribe. Each must match a topic from a
    ///         previous subscribe.
    ///
    pub fn unsubscribe_many<T>(&self, topics: &[T]) -> MqttResult<()>
        where T: AsRef<str>
    {
        self.cli.unsubscribe_many(topics).wait_for(self.timeout)?;
        Ok(())
    }

    /// Starts the client consuming messages.
    /// This starts the client receiving messages and placing them into an
    /// mpsc queue. It returns the receiving-end of the queue for the
    /// application to get the messages.
    /// This can be called at any time after the client is created, but it
    /// should be called before subscribing to any topics, otherwise messages
    /// can be lost.
    //
    pub fn start_consuming(&mut self) -> mpsc::Receiver<Option<Message>> {
        self.cli.start_consuming()
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    // Determine that a client can be sent across threads and signaled.
    // As long as it compiles, this indicates that Client implements the
    // Send trait.
    #[test]
    fn test_send() {
        let cli = Client::new("tcp://localhost:1883").unwrap();
        let thr = thread::spawn(move || {
            assert!(!cli.is_connected());
        });
        let _ = thr.join().unwrap();
    }

    // Determine that a client can be shared across threads using an Arc.
    // As long as it compiles, this indicates that Client implements the
    // Send trait.
    // This is a bit redundant with the previous test, but explicitly
    // addresses GitHub Issue #31.
    #[test]
    fn test_send_arc() {
        let cli = Client::new("tcp://localhost:1883").unwrap();

        let cli = Arc::new(cli);
        let cli2 = cli.clone();

        let thr = thread::spawn(move || {
            assert!(!cli.is_connected());
        });
        assert!(!cli2.is_connected());
        let _ = thr.join().unwrap();
    }
}

