// response_options.rs
//
// The set of options for responses coming back to an MQTT client.
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2019-2020 Frank Pagliughi <fpagliughi@mindspring.com>
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

//! Response options for the Paho MQTT Rust client library.

use std::{
    ptr,
    pin::Pin,
    os::raw::c_int,
};

use crate::{
    ffi,
    token::{
        Token,
        TokenInner,
    },
    subscribe_options::SubscribeOptions,
};

/// The collection of options for responses coming back to the client.
#[derive(Debug)]
pub struct ResponseOptions {
    pub(crate) copts: ffi::MQTTAsync_responseOptions,
    data: Pin<Box<ResponseOptionsData>>,
}

/// Cached data for the response options
#[derive(Debug, Default, Clone)]
struct ResponseOptionsData {
    sub_opts: Option<Vec<ffi::MQTTSubscribe_options>>,
}

impl ResponseOptions {
    /// Creates a `ResponseOptions` intance for the provided token.
    /// The option's `copts` can be passed to one of the C library's
    /// asynchronous functions to set the callbacks to complete the token.
    ///
    /// Note: We leak an Arc clone of the token into the internal C
    /// structure to act as the context pointer for the callback. It is
    /// up to the callback (or calling function) to recapture and release
    /// this token.
    pub(crate) fn new<T>(tok: T, mqtt_version: u32) -> Self
        where T: Into<Token>
    {
        let copts = Self::c_options(tok, mqtt_version);
        Self::from_data(copts, ResponseOptionsData::default())
    }

    fn from_data(
        mut copts: ffi::MQTTAsync_responseOptions,
        data: ResponseOptionsData
    ) -> Self {
        let mut data = Box::pin(data);

        let (n, p) = match data.sub_opts {
            Some(ref mut sub_opts) => (
                sub_opts.len() as c_int,
                sub_opts.as_mut_ptr()
            ),
            _ => (0 as c_int, ptr::null_mut())
        };

        copts.subscribeOptionsCount = n;
        copts.subscribeOptionsList = p;

        Self { copts, data }
    }

    // Gets the default C options struct for the specified MQTT version
    fn c_options<T>(tok: T, mqtt_version: u32) -> ffi::MQTTAsync_responseOptions
        where T: Into<Token>
    {
        let context = tok.into().into_raw();
        debug!("Created response for token at: {:?}", context);

        if mqtt_version < 5 {
            ffi::MQTTAsync_responseOptions {
                onSuccess: Some(TokenInner::on_success),
                onFailure: Some(TokenInner::on_failure),
                context,
                ..ffi::MQTTAsync_responseOptions::default()
            }
        }
        else {
            ffi::MQTTAsync_responseOptions {
                onSuccess5: Some(TokenInner::on_success5),
                onFailure5: Some(TokenInner::on_failure5),
                context,
                ..ffi::MQTTAsync_responseOptions::default()
            }
        }
    }

    pub(crate) fn from_subscribe_options<T>(tok: T, opts: SubscribeOptions) -> Self
        where T: Into<Token>
    {
        let mut copts = Self::c_options(tok, ffi::MQTTVERSION_5);
        copts.subscribeOptions = opts.copts;
        Self::from_data(copts, ResponseOptionsData::default())
    }

    pub(crate) fn from_subscribe_many_options<T>(tok: T, opts: &[SubscribeOptions]) -> Self
        where T: Into<Token>
    {
        let copts = Self::c_options(tok, ffi::MQTTVERSION_5);
        let sub_opts: Vec<_> = opts.iter().map(|opt| opt.copts).collect();
        Self::from_data(copts, ResponseOptionsData { sub_opts: Some(sub_opts), })
    }
}


/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token};
    use crate::types::*;

    #[test]
    fn test_new_v3() {
        let tok = Token::new();
        let opts = ResponseOptions::new(tok.clone(), MQTT_VERSION_3_1_1);

        let inner = Token::into_raw(tok);

        assert!(opts.copts.onSuccess.is_some());
        assert!(opts.copts.onFailure.is_some());
        // Check that the context is pointing to the right Token
        assert_eq!(inner, opts.copts.context);
        assert!(opts.copts.onSuccess5.is_none());
        assert!(opts.copts.onFailure5.is_none());

        let _ = unsafe { Token::from_raw(inner) };
    }

    #[test]
    fn test_new_v5() {
        let tok = Token::new();
        let opts = ResponseOptions::new(tok.clone(), MQTT_VERSION_5);

        let inner = Token::into_raw(tok);

        assert!(opts.copts.onSuccess.is_none());
        assert!(opts.copts.onFailure.is_none());
        // Check that the context is pointing to the right Token
        assert_eq!(inner, opts.copts.context);
        assert!(opts.copts.onSuccess5.is_some());
        assert!(opts.copts.onFailure5.is_some());

        let _ = unsafe { Token::from_raw(inner) };
    }

    #[test]
    fn test_from_opts() {
        let tok = Token::new();
        let sub_opts = SubscribeOptions::new(true);
        let opts = ResponseOptions::from_subscribe_options(tok.clone(), sub_opts);

        let inner = Token::into_raw(tok);

        assert!(opts.copts.onSuccess.is_none());
        assert!(opts.copts.onFailure.is_none());
        // Check that the context is pointing to the right Token
        assert_eq!(inner, opts.copts.context);
        assert!(opts.copts.onSuccess5.is_some());
        assert!(opts.copts.onFailure5.is_some());

        assert!(opts.copts.subscribeOptions.noLocal != 0);

        let _ = unsafe { Token::from_raw(inner) };
    }

    #[test]
    fn test_from_many_opts() {
        let tok = Token::new();
        let sub_opts = vec![SubscribeOptions::new(true) ; 4];
        let opts = ResponseOptions::from_subscribe_many_options(
            tok.clone(), &sub_opts
        );

        let inner = Token::into_raw(tok);

        assert!(opts.copts.onSuccess.is_none());
        assert!(opts.copts.onFailure.is_none());
        // Check that the context is pointing to the right Token
        assert_eq!(inner, opts.copts.context);
        assert!(opts.copts.onSuccess5.is_some());
        assert!(opts.copts.onFailure5.is_some());

        assert_eq!(0, opts.copts.subscribeOptions.noLocal);

        assert_eq!(4, opts.copts.subscribeOptionsCount);
        assert!(!opts.copts.subscribeOptionsList.is_null());

        let _ = unsafe { Token::from_raw(inner) };
    }
}

