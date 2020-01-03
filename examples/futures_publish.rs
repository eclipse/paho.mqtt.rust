// paho-mqtt/examples/futures_publish.rs
//
//! This is a simple MQTT asynchronous message publisher with Rust Futures
//! using the Paho Rust library.
//!
//! This sample demonstrates:
//!   - Connecting to an MQTT broker
//!   - Publishing a message asynchronously using Futures
//

/*******************************************************************************
 * Copyright (c) 2018 Frank Pagliughi <fpagliughi@mindspring.com>
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

extern crate futures;
extern crate log;
extern crate env_logger;
extern crate paho_mqtt as mqtt;

use std::{env, process};
use futures::Future;

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    // Get the server URL, if one is given on the command line
    let host = env::args().nth(1).unwrap_or_else(||
        "tcp://localhost:1883".to_string()
    );

    // Create a client & define connect options
    let cli = mqtt::AsyncClient::new(host).unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    // Connect, send a message, and disconnect
    cli.connect(None)
        .and_then(|_| {
            println!("Publishing a message on the 'test' topic");
            let msg = mqtt::Message::new("test", "Hello futures world!", 0);
            cli.publish(msg)
        })
        .and_then(|_| cli.disconnect(None))
        .wait().unwrap_or_else(|err| {
            println!("Error: {}", err);
            mqtt::ServerResponse::default()
        });
}

