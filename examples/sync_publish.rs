// paho-mqtt/examples/sync_publish.rs
// This is a Paho MQTT Rust client sample application.
//
//! This application is a simple MQTT publisher using the
//! synchronous/blocking client interface.
//!
//! The sample demonstrates:
//!   - Use of the synchronous/blocking API
//!   - Connecting to an MQTT broker
//!   - Publishing messages
//!

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

fn main() {
    // Initialize the logger from the environment
    env_logger::init().unwrap();

    // Create a client & define connect options
    let cli = mqtt::Client::new("tcp://localhost:1883").unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(None) {
        println!("Unable to connect: {:?}", e);
        process::exit(1);
    }

    // Create a message and publish it
    let msg = mqtt::MessageBuilder::new()
        .topic("test")
        .payload("Hello synchronous world!")
        .qos(1)
        .finalize();


    if let Err(e) = cli.publish(msg) {
        println!("Error sending message: {:?}", e);
    }

    // Disconnect from the broker
    cli.disconnect(None).unwrap();
}
