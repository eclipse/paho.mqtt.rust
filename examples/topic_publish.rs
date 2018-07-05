// paho-mqtt/examples/topic_publish.rs
// Example application for Paho MQTT Rust library.
//
//! This is a simple asynchronous publisher that uses a topic object to
//! repeatedly publish messages on the same topic.
//!
//! This sample demonstrates:
//!   - Connecting to an MQTT broker
//!   - Publishing a message asynchronously
//!   - Using a 'Topic' object to publish multiple messages to the same topic.
//!

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

extern crate log;
extern crate env_logger;
extern crate paho_mqtt as mqtt;

use std::process;

const QOS: i32 = 1;

fn main() {
	// Initialize the logger from the environment
	env_logger::init().unwrap();

	// Create a client & define connect options
	let cli = mqtt::AsyncClient::new("tcp://localhost:1883").unwrap_or_else(|err| {
		println!("Error creating the client: {}", err);
		process::exit(1);
	});

	let conn_opts = mqtt::ConnectOptions::new();

	// Connect and wait for it to complete or fail
	if let Err(e) = cli.connect(conn_opts).wait() {
		println!("Unable to connect: {:?}", e);
		process::exit(1);
	}

	// Create a topic and publish to it
	println!("Publishing messages on the 'test' topic");
	let topic = mqtt::Topic::new(&cli, "test", QOS);
	for _ in 0..5 {
		let tok = topic.publish("Hello there");

		if let Err(e) = tok.wait() {
			println!("Error sending message: {:?}", e);
			break;
		}
	}

	// Disconnect from the broker
	let tok = cli.disconnect(None);
	tok.wait().unwrap();
}

