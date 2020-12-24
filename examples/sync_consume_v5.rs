// paho-mqtt/examples/sync_consume_v5.rs
// This is a Paho MQTT Rust client, sample application.
//
//! This application is an MQTT v5 consumer/subscriber using the Rust
//! synchronous client interface, which uses the queuing API to
//! receive messages.
//!
//! The sample demonstrates:
//!   - Connecting to an MQTT server/broker
//!   - Checking server responses
//!   - Subscribing to multiple topics
//!   - Receiving messages through the queueing consumer API
//!   - Recieving and acting upon commands via MQTT topics
//!   - Manual reconnects
//!   - Using a persistent (non-clean) session
//!   - Last will and testament
//!

/*******************************************************************************
 * Copyright (c) 2020 Frank Pagliughi <fpagliughi@mindspring.com>
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

use std::{
    env,
    process,
    thread,
    time::Duration,
};
use paho_mqtt as mqtt;

// Handler for data messages (i.e. topic "data/#")
fn data_handler(msg: mqtt::Message) -> bool {
    println!("{}", msg);
    true
}

// Handler for command messages (i.e. topic "command")
// Return false to exit the application
fn command_handler(msg: mqtt::Message) -> bool {
    if msg.payload_str() == "exit" {
        println!("Exit command received");
        false
    }
    else {
        true
    }
}

/////////////////////////////////////////////////////////////////////////////

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

// --------------------------------------------------------------------------

fn main() -> mqtt::Result<()> {
    // Initialize the logger from the environment
    env_logger::init();

    let host = env::args().nth(1).unwrap_or_else(||
        "tcp://localhost:1883".to_string()
    );

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_5)
        .server_uri(host)
        .client_id("paho_rust_sync_cons_v5")
        .finalize();

    let mut cli = mqtt::Client::new(create_opts)?;

    // Initialize the consumer before connecting
    let rx = cli.start_consuming();

    // Define the set of options for the connection
    let lwt = mqtt::MessageBuilder::new()
        .topic("lwt")
        .payload("Sync consumer v5 lost connection")
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_5)
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    let handler: Vec<fn(mqtt::Message) -> bool> = vec![
        data_handler,
        command_handler
    ];

    // Make the connection to the broker

    let rsp = cli.connect(conn_opts)?;

    // We're connecting with a persistent session. So we check if
    // the server already knows about us and rembers about out
    // subscription(s). If not, we subscribe for incoming requests.

    if let Some(conn_rsp) = rsp.connect_response() {
        println!("Connected to: '{}' with MQTT version {}",
                 conn_rsp.server_uri, conn_rsp.mqtt_version);

        if !conn_rsp.session_present {
            // Register subscriptions on the server
            println!("Subscribing to topics...");

            let props = mqtt::properties![
                mqtt::PropertyCode::SubscriptionIdentifier => 1
            ];
            cli.subscribe_with_options("data/#", 0, mqtt::SubscribeOptions::default(), props)?;
            let props = mqtt::properties![
                mqtt::PropertyCode::SubscriptionIdentifier => 2
            ];
            cli.subscribe_with_options("command", 1, mqtt::SubscribeOptions::default(), props)?;
        }
    }

    // Just loop on incoming messages.
    // If we get a None message, check if we got disconnected,
    // and then try a reconnect.
    println!("Waiting for messages...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            let sub_id = msg.properties()
                .get_int(mqtt::PropertyCode::SubscriptionIdentifier)
                .expect("No Subscription ID") as usize;

            if !handler[sub_id-1](msg) {
                break;
            }
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
        cli.disconnect(None).unwrap();
    }
    println!("Exiting");

    Ok(())
}
