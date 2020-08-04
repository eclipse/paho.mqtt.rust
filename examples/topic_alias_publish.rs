// paho-mqtt/examples/topic_publish.rs
// Example application for Paho MQTT Rust library.
//
//! This is a Paho MQTT v5 Rust sample application.
//!
//! It's an asynchronous publisher that uses a topic object to
//! repeatedly publish messages on the same topic using an alias
//!
//! This sample demonstrates:
//!   - Connecting to an MQTT broker
//!   - Publishing a message asynchronously
//!   - Using a 'Topic' object to publish multiple messages to the same topic.
//!   - Using MQTT v5 topic aliases.
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

use std::{env, process};
use paho_mqtt as mqtt;

const MQTTV5: u32 = 5;

const QOS: i32 = 1;
const TOPIC_ALIAS: u16 = 1;
const TOPIC_NAME: &str = "test/very/long/topic_name/just/to_say_hello";

/////////////////////////////////////////////////////////////////////////////

fn main() -> mqtt::Result<()> {
	// Initialize the logger from the environment
	env_logger::init();

    let host = env::args().nth(1).unwrap_or_else(||
        "tcp://localhost:1883".to_string()
    );


    // Create a client to the specified host, no persistence
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(MQTTV5)
        .server_uri(host)
        .finalize();

	let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
		println!("Error creating the client: {}", err);
		process::exit(1);
	});

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .mqtt_version(MQTTV5)
        .clean_start(true)
        .finalize();

	// Connect and wait for it to complete or fail

    let rsp = cli.connect(conn_opts).wait().unwrap_or_else(|err| {
        eprintln!("Unable to connect: {:?}", err);
        process::exit(1);
    });

    match rsp.properties().get_val::<u16>(mqtt::PropertyCode::TopicAliasMaximum) {
        None | Some(0) => {
            eprintln!("The server doesn't support Topic Aliases.");
            process::exit(2);
        },
        Some(n) => println!("The server supports up to {} aliases.", n),
    }

	// Create a topic and publish to it
	println!("Publishing messages on the 'test' topic");
	let mut topic = mqtt::Topic::new(&cli, TOPIC_NAME, QOS);

    // First, publish with the alias to set it on the server
    topic.publish_with_alias(TOPIC_ALIAS, "Hello. Here's an alias.").wait()?;

    // These publishes now use the alias
    for _ in 0..4 {
        topic.publish("Hello there").wait()?;
	}

    // This removes the alias
    topic.publish_with_alias(0, "Hello. Removed the alias.").wait()?;

    // Subsequent publishes use the topic name string.
    topic.publish("No alias here").wait()?;

	// Disconnect from the broker
	let tok = cli.disconnect(None);
	tok.wait()?;

    Ok(())
}
