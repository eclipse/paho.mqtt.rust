// ssl_options.rs
// 
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2017-2019 Frank Pagliughi <fpagliughi@mindspring.com>
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
    ptr,
    ffi::{CString},
    os::raw::{c_char},
    pin::Pin,
};

use crate::ffi;

// Implementation note:
// The C library seems to require the SSL Options struct to provide valid
// strings for all the entries. Empty entries require a zero-length string
// and should not be left as NULL. Therefore, out ffi::MQTTAsync_SSLOptions
// struct needs to be fixed up to always point to the cached CString 
// values.
// Caching the CStrings in the struct with the fixed-up pointers in the 
// 

/// The options for SSL socket connections to the broker.
#[derive(Debug)]
pub struct SslOptions {
    pub(crate) copts: ffi::MQTTAsync_SSLOptions,
    data: Pin<Box<SslOptionsData>>,
}

#[derive(Debug, Default, Clone)]
struct SslOptionsData {
    trust_store: CString,
    key_store: CString,
    private_key: CString,
    private_key_password: CString,
    enabled_cipher_suites: CString,
}

impl SslOptions {
    /// Creates a new set of default SSL options
    pub fn new() -> Self {
        Self::default()
    }

    // The C library expects unset values in the SSL options struct to be
    // NULL rather than empty string.
    fn c_str(str: &CString) -> *const c_char {
        if str.to_bytes().len() == 0 { ptr::null() } else { str.as_ptr() }
    }

    // Updates the underlying C structure to match the cached strings.
    fn from_data(mut copts: ffi::MQTTAsync_SSLOptions, data: SslOptionsData) -> Self {
        let data = Box::pin(data);
        copts.trustStore = SslOptions::c_str(&data.trust_store);
        copts.keyStore = SslOptions::c_str(&data.key_store);
        copts.privateKey = SslOptions::c_str(&data.private_key);
        copts.privateKeyPassword = SslOptions::c_str(&data.private_key_password);
        copts.enabledCipherSuites = SslOptions::c_str(&data.enabled_cipher_suites);
        Self { copts, data }
    }

    pub fn trust_store(&self) -> String {
        self.data.trust_store.to_str().unwrap().to_string()
    }

    pub fn key_store(&self) -> String {
        self.data.key_store.to_str().unwrap().to_string()
    }

    pub fn private_key(&self) -> String {
        self.data.private_key.to_str().unwrap().to_string()
    }

    pub fn private_key_password(&self) -> String {
        self.data.private_key_password.to_str().unwrap().to_string()
    }

    pub fn enabled_cipher_suites(&self) -> String {
        self.data.enabled_cipher_suites.to_str().unwrap().to_string()
    }

    pub fn enable_server_cert_auth(&self) -> bool {
        self.copts.enableServerCertAuth != 0
    }
}

impl Default for SslOptions {
    fn default() -> Self {
        Self::from_data(
            ffi::MQTTAsync_SSLOptions::default(),
            SslOptionsData::default()
        )
    }
}

impl Clone for SslOptions {
    fn clone(&self) -> Self {
        Self::from_data(
            self.copts.clone(),
            (&*self.data).clone()
        )
    }
}

unsafe impl Send for SslOptions {}
unsafe impl Sync for SslOptions {}


/////////////////////////////////////////////////////////////////////////////
//                              Builder
/////////////////////////////////////////////////////////////////////////////

/// Builder to create SSL Options.
#[derive(Default)]
pub struct SslOptionsBuilder {
    copts: ffi::MQTTAsync_SSLOptions,
    data: SslOptionsData,
}

impl SslOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trust_store<S>(&mut self, trust_store: S) -> &mut Self
        where S: Into<String>
    {
        self.data.trust_store = CString::new(trust_store.into()).unwrap();
        self
    }

    pub fn key_store<S>(&mut self, key_store: S) -> &mut Self
        where S: Into<String>
    {
        self.data.key_store = CString::new(key_store.into()).unwrap();
        self
    }

    pub fn private_key<S>(&mut self, private_key: S) -> &mut Self
        where S: Into<String>
    {
        self.data.private_key = CString::new(private_key.into()).unwrap();
        self
    }

    pub fn private_key_password<S>(&mut self, private_key_password: S) -> &mut Self
        where S: Into<String>
    {
        self.data.private_key_password =
            CString::new(private_key_password.into()).unwrap();
        self
    }

    pub fn enabled_cipher_suites<S>(&mut self, enabled_cipher_suites: S) -> &mut Self
        where S: Into<String>
    {
        self.data.enabled_cipher_suites =
            CString::new(enabled_cipher_suites.into()).unwrap();
        self
    }

    pub fn finalize(&self) -> SslOptions {
        SslOptions::from_data(
            self.copts.clone(),
            self.data.clone()
        )
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr};

    // Identifier for the C SSL options structure
    const STRUCT_ID: [c_char; 4] = [ b'M' as c_char, b'Q' as c_char, b'T' as c_char, b'S' as c_char ];

    #[test]
    fn test_new() {
        let opts = SslOptions::new();
        //let copts = ffi::MQTTAsync_SSLOptions::default();

        assert_eq!(STRUCT_ID, opts.copts.struct_id);
        assert_eq!(4, opts.copts.struct_version);
        assert_eq!(ptr::null(), opts.copts.trustStore);
        // TODO: Check the other strings
    }

    #[test]
    fn test_builder_dflt() {
        let opts = SslOptionsBuilder::new()
            .finalize();

        assert_eq!(STRUCT_ID, opts.copts.struct_id);
        assert_eq!(4, opts.copts.struct_version);
        assert_eq!(ptr::null(), opts.copts.trustStore);
        // TODO: Check the other strings
    }

    #[test]
    fn test_builder_trust_store() {
        const TRUST_STORE: &str = "some_file.crt";
        let opts = SslOptionsBuilder::new()
            .trust_store(TRUST_STORE)
            .finalize();

        assert_eq!(TRUST_STORE, opts.data.trust_store.to_str().unwrap());
        let ts = unsafe { CStr::from_ptr(opts.copts.trustStore) };
        assert_eq!(TRUST_STORE, ts.to_str().unwrap());
    }

    #[test]
    fn test_builder_key_store() {
        const KEY_STORE: &str = "some_file.crt";
        let opts = SslOptionsBuilder::new()
            .key_store(KEY_STORE)
            .finalize();

        assert_eq!(KEY_STORE, opts.data.key_store.to_str().unwrap());
    }

    // TODO: Test the other builder initializers

    #[test]
    fn test_copy() {
        const TRUST_STORE: &str = "some_file.crt";
        let org_opts = SslOptionsBuilder::new()
            .trust_store(TRUST_STORE)
            .finalize();

        let opts = org_opts;
        assert_eq!(TRUST_STORE, opts.data.trust_store.to_str().unwrap());
        let ts = unsafe { CStr::from_ptr(opts.copts.trustStore) };
        assert_eq!(TRUST_STORE, ts.to_str().unwrap());
    }

    #[test]
    fn test_clone() {
        const TRUST_STORE: &str = "some_file.crt";
        // Make sure the original goes out of scope
        // before testing the clone.
        let opts = {
            let org_opts = SslOptionsBuilder::new()
                .trust_store(TRUST_STORE)
                .finalize();

            org_opts.clone()
        };

        assert_eq!(TRUST_STORE, opts.data.trust_store.to_str().unwrap());
        let ts = unsafe { CStr::from_ptr(opts.copts.trustStore) };
        assert_eq!(TRUST_STORE, ts.to_str().unwrap());
    }
}

