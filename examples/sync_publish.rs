// sync_publish.rs
//
// Example/test for Paho MQTT Rust library.
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

fn main() {
	// Create a client & define connect options
	let cli = mqtt::Client::new("tcp://localhost:1883", "");

	// Connect and wait for it to complete or fail
	if let Err(e) = cli.connect(None) {
		println!("Unable to connect:\n\t{:?}", e);
		::std::process::exit(1);
	}

	// Create a message and publish it
	let msg = mqtt::Message::new("test", "Hello synchronous world!");

	if let Err(e) = cli.publish(msg) {
		println!("Error sending message: {:?}", e);
	}

	// Disconnect from the broker
	cli.disconnect(None).unwrap();
}