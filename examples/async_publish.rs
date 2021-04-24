// paho-mqtt/examples/async_publish.rs
//
//! This is a simple MQTT asynchronous message publisher using the
//! Paho Rust library.
//!
//! This sample demonstrates:
//!   - Connecting to an MQTT broker
//!   - Publishing a message asynchronously
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

use futures::executor::block_on;
use paho_mqtt as mqtt;
use std::{env, process};

/////////////////////////////////////////////////////////////////////////////

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    // Command-line option(s)
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "tcp://localhost:1883".to_string());

    // Create the client
    let cli = mqtt::AsyncClient::new(host).unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Connect with default options and wait for it to complete or fail
        println!("Connecting to the MQTT server");
        cli.connect(None).await?;

        // Create a message and publish it
        println!("Publishing a message on the topic 'test'");
        let msg = mqtt::Message::new("test", "Hello Rust MQTT world!", mqtt::QOS_1);
        cli.publish(msg).await?;

        // Disconnect from the broker
        println!("Disconnecting");
        cli.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}
