// async_client.rs
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

use std::time::Duration;
use std::sync::mpsc::{Receiver};
//use std::sync::mpsc;

use async_client::{AsyncClient};
use connect_options::{ConnectOptions};
use disconnect_options::{DisconnectOptions};
use message::{Message};
use errors::{MqttResult, /*MqttError, ErrorKind*/};

/////////////////////////////////////////////////////////////////////////////
// Client

pub struct Client {
	cli: Box<AsyncClient>,
	timeout: Duration,
}

impl Client {
	pub fn new(server_uri: &str, client_id: &str) -> MqttResult<Client> {
		let async_cli = AsyncClient::new(server_uri, client_id)?;
		let cli = Client {
			cli: Box::new(async_cli),
			timeout: Duration::from_secs(5*60),
		};
		//cli.start_consuming();
		Ok(cli)
	}

	pub fn connect<T: Into<Option<ConnectOptions>>>(&self, opt_opts:T) -> MqttResult<()> {
		self.cli.connect(opt_opts).wait_for(self.timeout)
	}

	pub fn disconnect<T: Into<Option<DisconnectOptions>>>(&self, opt_opts:T) -> MqttResult<()> {
		self.cli.disconnect(opt_opts).wait_for(self.timeout)
	}

	pub fn disconnect_after(&self, timeout: Duration) -> MqttResult<()> {
		self.cli.disconnect_after(timeout).wait_for(self.timeout)
	}

	pub fn reconnect(&self) -> MqttResult<()> {
		self.cli.reconnect().wait_for(self.timeout)
	}

	pub fn is_connected(&self) -> bool {
		self.cli.is_connected()
	}

	pub fn publish(&self, msg: Message) -> MqttResult<()> {
		self.cli.publish(msg).wait_for(self.timeout)
	}

	pub fn subscribe(&self, topic: &str, qos: i32) -> MqttResult<()> {
		self.cli.subscribe(topic, qos).wait_for(self.timeout)
	}

	pub fn subscribe_many(&self, topics: Vec<String>, qos: Vec<i32>) -> MqttResult<()> {
		self.cli.subscribe_many(topics, qos).wait_for(self.timeout)
	}

	pub fn unsubscribe(&self, topic: &str) -> MqttResult<()> {
		self.cli.unsubscribe(topic).wait_for(self.timeout)
	}

	pub fn unsubscribe_many(&self, topics: Vec<String>) -> MqttResult<()> {
		self.cli.unsubscribe_many(topics).wait_for(self.timeout)
	}

	pub fn start_consuming(&mut self) -> Receiver<Message> {
		self.cli.start_consuming()
	}
}

