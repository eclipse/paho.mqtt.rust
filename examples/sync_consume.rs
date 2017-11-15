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

extern crate paho_mqtt as mqtt;

use std::{thread};
use std::time::Duration;

/////////////////////////////////////////////////////////////////////////////

fn main() {
	// Create the client connection
	let cli = mqtt::Client::new("tcp://localhost:1883", "");

	// Define the set of options for the connection
	let will_opts = mqtt::WillOptionsBuilder::new()
		.topic("test")
		.payload("Consumer lost connection".as_bytes().to_vec())
		.finalize();

	let conn_opts = mqtt::ConnectOptionsBuilder::new()
		.keep_alive_interval(Duration::from_secs(20))
		.clean_session(true)
		.will_options(will_opts)
		.finalize();

	// Make the connection to the broker
	println!("Connecting to the MQTT broker...");
	if let Err(e) = cli.connect(conn_opts) {
		println!("Error connecting to the broker: {:?}", e);
		::std::process::exit(1);
	};

	// Initialize the consumer
	let rx = cli.start_consuming();
	cli.subscribe_many(vec!("test".to_string(), "hello".to_string()), vec!(1, 1));


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

