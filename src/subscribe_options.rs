// subscribe_options.rs
//
// The set of options for responses coming back to an MQTT client.
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

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

//! MQTT v5 subscribe options for the Paho MQTT Rust client library.
//! These are defined in section 3.8.3.1 of the MQTT v5 spec.
//! The defaults use the behavior that was present in MQTT v3.1.1.

use ffi;

/// Don't receive our own publications
pub const SUBSCRIBE_NO_LOCAL: bool = true;
/// Receive our own publications
pub const SUBSCRIBE_LOCAL: bool = false;

/// Retain flag is only set on publications sent by a broker if in
/// response to a subscribe request
pub const NO_RETAIN_AS_PUBLISHED: bool = false;
/// Keep the retain flag as on the original publish message
pub const RETAIN_AS_PUBLISHED: bool = true;

/// The options for subscription retain handling
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RetainHandling {
	/** Send retained messages at the time of the subscribe */
	SEND_RETAINED_ON_SUBSCRIBE = 0,
	/** Send retained messages on subscribe only if subscription is new */
	SEND_RETAINED_ON_NEW = 1,
	/** Do not send retained messages at all */
	DONT_SEND_RETAINED = 2
}

/// The MQTT v5 subscribe options.
#[derive(Debug, Copy, Clone)]
pub struct SubscribeOptions {
    pub(crate) copts: ffi::MQTTSubscribe_options,
}

impl SubscribeOptions {
    /// Creates set of subscribe options.
    pub fn new(no_local: bool) -> Self
    {
        SubscribeOptions {
            copts: ffi::MQTTSubscribe_options {
                noLocal: if no_local { 1 } else { 0 },
                ..ffi::MQTTSubscribe_options::default()
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct SubscribeOptionsBuilder {
    copts: ffi::MQTTSubscribe_options,
}

impl SubscribeOptionsBuilder {
    pub fn new() -> Self {
        SubscribeOptionsBuilder::default()
    }

    pub fn no_local(mut self, on: bool) -> Self {
        self.copts.noLocal = if on { 1 } else { 0 };
        self
    }

    pub fn retain_as_published(mut self, retain: bool) -> Self {
        self.copts.retainAsPublished = if retain { 1 } else { 0 };
        self
    }

    pub fn retain_handling(mut self, handling: RetainHandling) -> Self {
        self.copts.retainHandling = handling as u8;
        self
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use token::{Token};

    #[test]
    fn test_new() {
        let opts = SubscribeOptions::new(false);
        assert!(opts.copts.noLocal == 0);

        let opts = SubscribeOptions::new(true);
        assert!(opts.copts.noLocal != 0);
    }
}

