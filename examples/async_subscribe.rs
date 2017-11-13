// async_subscribe.rs
//
// This is a Paho MQTT Rust client, sample application.
//
// This application is an MQTT subscriber using the C++ asynchronous client
// interface, employing callbacks to receive messages and status updates.
//
// The sample demonstrates:
//  - Connecting to an MQTT server/broker.
//  - Subscribing to a topic
//  - Receiving messages through the callback API
//  - Receiving network disconnect updates and attempting manual reconnects.
//  - Using a "clean session" and manually re-subscribing to topics on
//    reconnect.
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

extern crate paho_mqtt;

use std::{thread};
use std::time::Duration;
use paho_mqtt as mqtt;


// Callback for a successful connection to the broker.
// We subscribe to the topic(s) we want here.
fn on_connect_success(cli: &mqtt::AsyncClient, _msgid: u16) {
	println!("Connection succeeded\n");
	// Subscribe to the desired topic(s).
	//cli.subscribe("test", 1);
	cli.subscribe_many(vec!("test".to_string(), "hello".to_string()), vec!(1, 1));
	println!("Subscribing to topics.");
	// TODO: This doesn't yet handle a failed subscription.
}

// Callback for a failed attempt to connect to the server.
// We simply sleep and then try again.
//
// Note that normally we don't want to do a blocking operation or sleep 
// from  within a callback. But in this case, we know that the client is 
// *not* conected, and thus not doing anything important. So we don't worry 
// too much about stopping its callback thread.
fn on_connect_failure(cli: &mqtt::AsyncClient, _msgid: u16, rc: i32) {
	println!("Connection attempt failed with error code {}.\n", rc);
	thread::sleep(Duration::from_millis(2500));
	cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
}

/////////////////////////////////////////////////////////////////////////////

fn main() {
	// Create the client connection
	let mut cli = mqtt::AsyncClient::new("tcp://localhost:1883", "");

	// Set a closure to be called whenever the client loses the connection.
	// It will attempt to reconnect, and set up function callbacks to keep
	// retrying until the connection is re-established.
	cli.set_connection_lost_callback(|cli: &mut mqtt::AsyncClient| {
		println!("Connection lost. Attempting reconnect.");
		thread::sleep(Duration::from_millis(2500));
		cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
	});

	// Attach a closure to the client to receive callback
	// on incoming messages.
	cli.set_message_callback(|_cli,msg| {
		let topic = msg.get_topic().unwrap();
		let payload_str = msg.get_payload_str().unwrap();
		println!("Message:  {} - {}\n", topic, payload_str);
	});

	// Define the set of options for the connection
	let will_opts = mqtt::WillOptionsBuilder::new()
		.topic("test")
		.payload("Lost connection".as_bytes().to_vec())
		.finalize();

	println!("Starting conn_opts builder");
	let conn_opts = mqtt::ConnectOptionsBuilder::new()
		.keep_alive_interval(Duration::from_secs(20))
		.clean_session(true)
		.will_options(will_opts)
		.finalize();

	println!("\nFinished conn_opts builder");

	//let copts = conn_opts.to_c_struct();
	//println!("copts: {:?}", copts);

	//conn_opts = mqtt::ConnectOptions::fixup(conn_opts);
	let co = conn_opts.clone();
	println!("****");
	println!("\nConn: {:?}", conn_opts);
	println!("\nConn: {:?}", co);
	println!("****");

	// Make the connection to the broker
	println!("Connecting to the MQTT server...");
	cli.connect_with_callbacks(conn_opts, on_connect_success, on_connect_failure);

	// Just wait for incoming messages.
	loop {
		thread::sleep(Duration::from_millis(1000));
	}

	/*
	cli.unsubscribe_many(vec!("test".to_string(), "hello".to_string())).wait();

	let tok = cli.disconnect(None);
	tok.wait();
	*/
}

