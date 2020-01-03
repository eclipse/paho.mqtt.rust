// paho-mqtt/examples/async_publish_time.rs
//
//! This is a simple MQTT asynchronous message publisher using the Paho
//! Rust library. It shows a typical use case (although with contrived
//! "data"), where a client might poll a sensor and then publish the
//! reading when the data changes. In this case we use the system clock
//! in place of a sensor, looking at an integer time_t value in units
//! of 1/100 sec. Thus, it should publish the current time 100x per
//! second.
//!
//! This sample demonstrates:
//!   - Connecting to an MQTT broker
//!   - Repeatedly publishing message asynchronously
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

extern crate futures;
extern crate log;
extern crate env_logger;
extern crate paho_mqtt as mqtt;

use std::{env, process, thread};
use std::time::{SystemTime, Duration};
use futures::Future;

// Read the system time in units of 1/100 sec since epoch.
fn time_now_hundredths() -> u64 {
    (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()/10) as u64
}

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    let host = env::args().nth(1).unwrap_or_else(||
        "tcp://localhost:1883".to_string()
    );

    // Create a client & define connect options
    let cli = mqtt::AsyncClient::new(host).unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptions::new();

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        println!("Unable to connect: {:?}", e);
        process::exit(1);
    }

    let topic = "data/time";

    // Create messages and publish them
    println!("Publishing time on the topic '{}'", topic);

    loop {
        let t0 = time_now_hundredths();
        let mut t = t0;

        // Wait until the time reading changes
        while t == t0 {
            thread::sleep(Duration::from_millis(10));
            t = time_now_hundredths();
        }

        let tf = 0.01 * (t as f64);

        let msg = mqtt::Message::new(topic, format!("{:.3}", tf), 1);
        if let Err(e) = cli.publish(msg).wait() {
            println!("Error sending message: {:?}", e);
        }
    }
}

