// sync_consume.rs
//
// This is a Paho MQTT Rust client, sample application.
//
// This application is an MQTT consumer/subscriber using the Rust
// synchronous client interface, which uses the queuing API to
// receive messages.
//
// The sample demonstrates:
//  - Connecting to an MQTT server/broker
//  - Subscribing to multiple topics
//  - Receiving messages through the queueing consumer API
//  - Recieving and acting upon commands via MQTT topics
//  - Manual reconnects
//  - Using a persistent (non-clean) session
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

extern crate log;
extern crate env_logger;

extern crate paho_mqtt as mqtt;

use std::process;
use std::time::Duration;

/////////////////////////////////////////////////////////////////////////////

fn main() {
	// Initialize the logger from the environment
	env_logger::init().unwrap();

	// Create the client connection
	let mut cli = mqtt::Client::new("tcp://localhost:1883").unwrap_or_else(|e| {
		println!("Error creating the client: {:?}", e);
		process::exit(1);
	});

	// Define the set of options for the connection
	let lwt = mqtt::MessageBuilder::new()
		.topic("test")
		.payload("Sync consumer lost connection")
		.finalize();

	let conn_opts = mqtt::ConnectOptionsBuilder::new()
		.keep_alive_interval(Duration::from_secs(20))
		.clean_session(true)
		.will_message(lwt)
		.finalize();

	// Make the connection to the broker
	println!("Connecting to the MQTT broker...");
	if let Err(e) = cli.connect(conn_opts) {
		println!("Error connecting to the broker: {:?}", e);
		process::exit(1);
	};

	// Initialize the consumer & subscribe to topics
	let rx = cli.start_consuming();
	let subscriptions = vec!("test".to_string(), "hello".to_string());

	if let Err(e) = cli.subscribe_many(subscriptions, vec!(1, 1)) {
		println!("Error subscribing to topics: {:?}", e);
		cli.disconnect(None).unwrap();
		::std::process::exit(1);
	}

	// Just wait for incoming messages.
	loop {
		let rsp = rx.recv();

		if let Err(e) = rsp {
			println!("Error receiving a message: {:?}", e);
			break;
		}

		let msg = rsp.unwrap();
		println!("Message: {:?}", msg);
	}

	cli.unsubscribe_many(vec!("test".to_string(), "hello".to_string())).unwrap();
	cli.disconnect(None).unwrap();
}

