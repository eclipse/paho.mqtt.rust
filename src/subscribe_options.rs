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

use std::fmt;
use crate::{
    ffi,
    to_c_bool,
};

/// Don't receive our own publications when subscribed to the same topics.
pub const SUBSCRIBE_NO_LOCAL: bool = true;
/// Receive our own publications when subscribed to the same topics.
pub const SUBSCRIBE_LOCAL: bool = false;

/// Retain flag is only set on publications sent by a broker if in
/// response to a subscribe request
pub const NO_RETAIN_AS_PUBLISHED: bool = false;
/// Keep the retain flag as on the original publish message
pub const RETAIN_AS_PUBLISHED: bool = true;

/// The options for subscription retain handling.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RetainHandling {
	/// Send retained messages at the time of the subscribe
	SendRetainedOnSubscribe = 0,
	/// Send retained messages on subscribe only if subscription is new
	SendRetainedOnNew = 1,
	/// Do not send retained messages at all
	DontSendRetained = 2
}

impl Default for RetainHandling {
    fn default() -> Self {
        RetainHandling::SendRetainedOnSubscribe
    }
}

impl fmt::Display for RetainHandling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RetainHandling::SendRetainedOnSubscribe =>
                write!(f, "Send Retain on Subscribe"),
            RetainHandling::SendRetainedOnNew =>
                write!(f, "Send Retain on New"),
            RetainHandling::DontSendRetained =>
                write!(f, "Don't Send Retain"),
        }
    }
}

/// The MQTT v5 subscribe options.
#[derive(Debug, Default, Copy, Clone)]
pub struct SubscribeOptions {
    pub(crate) copts: ffi::MQTTSubscribe_options,
}

impl SubscribeOptions {
    /// Creates set of subscribe options.
    pub fn new(no_local: bool) -> Self {
        SubscribeOptions {
            copts: ffi::MQTTSubscribe_options {
                noLocal: to_c_bool(no_local) as u8,
                ..ffi::MQTTSubscribe_options::default()
            }
        }
    }
}

impl From<bool> for SubscribeOptions {
	fn from(no_local: bool) -> Self {
		SubscribeOptions::new(no_local)
	}
}

impl From<Option<bool>> for SubscribeOptions {
	fn from(no_local: Option<bool>) -> Self {
        match no_local {
            Some(no_local) => SubscribeOptions::new(no_local),
            None => SubscribeOptions::default(),
        }
	}
}

impl From<(bool,bool)> for SubscribeOptions {
	fn from((no_local, retain_as_published): (bool, bool)) -> Self {
		let mut opts = SubscribeOptions::new(no_local);
		opts.copts.retainAsPublished = to_c_bool(retain_as_published) as u8;
		opts
	}
}

impl From<(bool,bool,RetainHandling)> for SubscribeOptions {
	fn from((no_local, retain_as_published, retain_handling): (bool, bool, RetainHandling)) -> Self {
        let mut opts = SubscribeOptions::from((no_local, retain_as_published));
        opts.copts.retainHandling = retain_handling as u8;
		opts
	}
}

/////////////////////////////////////////////////////////////////////////////
//                              Builder
/////////////////////////////////////////////////////////////////////////////

/// Builder for creating subscription options.
#[derive(Debug, Default)]
pub struct SubscribeOptionsBuilder {
    copts: ffi::MQTTSubscribe_options,
}

impl SubscribeOptionsBuilder {
    /// Create a new `SubscribeOptionsBuilder`
    pub fn new() -> Self {
        SubscribeOptionsBuilder::default()
    }

    /// Set so that the client doesn't receive its own messages that it
    /// publishes on the topic.
    pub fn no_local(&mut self, on: bool) -> &mut Self {
        self.copts.noLocal = to_c_bool(on) as u8;
        self
    }

    /// Set to keep the retain flag as on the original published message.
    /// If not set, the original MQTT behavior is where the retain flag is
    /// only set on publications sent by a broker if in response to a
    /// subscribe request.
    pub fn retain_as_published(&mut self, retain: bool) -> &mut Self {
        self.copts.retainAsPublished = to_c_bool(retain) as u8;
        self
    }

    /// Sets how retained messages are handled.
    pub fn retain_handling(&mut self, handling: RetainHandling) -> &mut Self {
        self.copts.retainHandling = handling as u8;
        self
    }

    /// Finalizes the builder to create the subscribe options.
    pub fn finalize(&self) -> SubscribeOptions {
        SubscribeOptions { copts: self.copts }
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let opts = SubscribeOptions::default();
        assert!(opts.copts.noLocal == 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_new() {
        let opts = SubscribeOptions::new(false);
        assert!(opts.copts.noLocal == 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 0);

        let opts = SubscribeOptions::new(true);
        assert!(opts.copts.noLocal != 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_from_tuple_one() {
        let opts = SubscribeOptions::from(true);
        assert!(opts.copts.noLocal != 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_from_tuple_two() {
        let opts = SubscribeOptions::from((true, true));
        assert!(opts.copts.noLocal != 0);
        assert!(opts.copts.retainAsPublished != 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_from_tuple_three() {
        let opts = SubscribeOptions::from(
            (true, true, RetainHandling::SendRetainedOnNew)
        );
        assert!(opts.copts.noLocal != 0);
        assert!(opts.copts.retainAsPublished != 0);
        assert!(opts.copts.retainHandling == 1);
    }

    #[test]
    fn test_builder_default() {
        let opts = SubscribeOptionsBuilder::new().finalize();
        assert!(opts.copts.noLocal == 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_builder_no_local() {
        let opts = SubscribeOptionsBuilder::new()
            .no_local(true)
            .finalize();

        assert!(opts.copts.noLocal != 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_builder_retain_as_published() {
        let opts = SubscribeOptionsBuilder::new()
            .retain_as_published(true)
            .finalize();

        assert!(opts.copts.noLocal == 0);
        assert!(opts.copts.retainAsPublished != 0);
        assert!(opts.copts.retainHandling == 0);
    }

    #[test]
    fn test_builder_retain_handling() {
        let opts = SubscribeOptionsBuilder::new()
            .retain_handling(RetainHandling::DontSendRetained)
            .finalize();

        assert!(opts.copts.noLocal == 0);
        assert!(opts.copts.retainAsPublished == 0);
        assert!(opts.copts.retainHandling == 2);
    }
}

