// paho-mqtt/examples/futures_roundtrip.rs
// This is a Paho MQTT Rust client, sample application.
//
//! This application is an MQTT publisher and subscriber using Rust futures.
//! It just creates a round-trip loop where it subscribes to its own message(s),
//! sends one, then re-sends whatever it receives. This creates an endless loop.
//! It also serves as a simple life test that can run forever.
//!
//! The sample demonstrates:
//!   - Using the Futures interfaces to publish and subscribe
//!   - Connecting to an MQTT server/broker.
//!   - Receiving messages through the futures stream
//!   - Automatic reconnects
//!

/*******************************************************************************
 * Copyright (c) 2019 Frank Pagliughi <fpagliughi@mindspring.com>
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
extern crate futures;
extern crate paho_mqtt as mqtt;

use std::{process};
use std::time::Duration;
use futures::{Future, Stream};
use futures::future::ok;

// The topic that we use for the test
const TOPIC: &str = "test/roundtrip";
const QOS: i32 = 1;

/////////////////////////////////////////////////////////////////////////////

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(("tcp://localhost:1883", "rust_roundtrip")).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    // Open a receiver stream
    let rx = cli.get_stream(10);

    // Set the connect options
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .clean_session(true)
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30))
        .finalize();

    // Connect, subscribe, and then send a message
    println!("Connecting to the MQTT server...");
    cli.connect(conn_opts)
        .and_then(|_| {
            println!("Subscribing to topic");
            cli.subscribe(TOPIC, QOS)
        })
        .and_then(|_| {
            println!("Publishing a message on the '{}' topic", TOPIC);
            let msg = mqtt::Message::new(TOPIC, "This is the roundtrip message!", QOS);
            cli.publish(msg)
        })
        .wait().unwrap_or_else(|err| {
            println!("Error: {}", err);
            process::exit(2);
        });

    // Loop, waiting for messages, and then resubmit them

    println!("Waiting for messages...");
    rx.for_each(|opt_msg| {
        if let Some(msg) = opt_msg {
            println!("{}", msg);
            cli.publish(msg);
        }
        else {
            println!("Stream disruption");
        }
        ok(())
    }).wait().unwrap_or_else(|_| {
        println!("Error receiving messages");
        process::exit(3);
    });
}

