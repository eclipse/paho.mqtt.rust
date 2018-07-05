// paho-mqtt/examples/sync_consume.rs
// This is a Paho MQTT Rust client, sample application.
//
//! This application is an MQTT consumer/subscriber using the Rust
//! synchronous client interface, which uses the queuing API to
//! receive messages.
//!
//! The sample demonstrates:
//!   - Connecting to an MQTT server/broker
//!   - Subscribing to multiple topics
//!   - Receiving messages through the queueing consumer API
//!   - Recieving and acting upon commands via MQTT topics
//!   - Manual reconnects
//!   - Using a persistent (non-clean) session
//!   - Last will and testament
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

use std::{process, thread};
use std::time::Duration;

// --------------------------------------------------------------------------
// This will attempt to reconnect to the broker. It can be called after 
// connection is lost. In this example, we try to reconnect several times,
// with a few second pause between each attempt. A real system might keep
// trying indefinitely, with a backoff, or something like that.

fn try_reconnect(cli: &mqtt::Client) -> bool
{
    println!("Connection lost. Waiting to retry connection");
    for _ in 0..12 {
        thread::sleep(Duration::from_millis(5000));
        if cli.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }
    println!("Unable to reconnect after several attempts.");
    false
}

/////////////////////////////////////////////////////////////////////////////

fn main() {
    // Initialize the logger from the environment
    env_logger::init().unwrap();

    // Create the client. Use an ID for a persisten session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri("tcp://localhost:1883")
        .client_id("rust_sync_consumer")
        .finalize();

    let mut cli = mqtt::Client::new(create_opts).unwrap_or_else(|e| {
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
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    // Make the connection to the broker
    println!("Connecting to the MQTT broker...");
    if let Err(e) = cli.connect(conn_opts) {
        println!("Error connecting to the broker: {:?}", e);
        process::exit(1);
    };

    // Initialize the consumer & subscribe to topics
    println!("Subscribing to topics...");
    let rx = cli.start_consuming();

    let subscriptions = [ "test", "hello" ];
    let qos = [1, 1];

    if let Err(e) = cli.subscribe_many(&subscriptions, &qos) {
        println!("Error subscribing to topics: {:?}", e);
        cli.disconnect(None).unwrap();
        process::exit(1);
    }

    // Just loop on incoming messages.
    // If we get a None message, check if we got disconnected,
    // and then try a reconnect.
    println!("Waiting for messages...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            println!("{}", msg);
        }
        else if cli.is_connected() ||
                !try_reconnect(&cli) {
            break;
        }
    }

    // If we're still connected, then disconnect now,
    // otherwise we're already disconnected.
    if cli.is_connected() {
        println!("Disconnecting");
        cli.unsubscribe_many(&subscriptions).unwrap();
        cli.disconnect(None).unwrap();
    }
    println!("Exiting");
}

