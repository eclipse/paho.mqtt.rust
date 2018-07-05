// ssl_options.rs
// 
// This file is part of the Eclipse Paho MQTT Rust Client library.
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

use std::ptr;
use std::ffi::{CString};
use std::os::raw::{c_char};

use ffi;

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
    trust_store: CString,
    key_store: CString,
    private_key: CString,
    private_key_password: CString,
    enabled_cipher_suites: CString,
}

impl SslOptions {
    pub fn new() -> SslOptions {
        let opts = SslOptions {
            copts: ffi::MQTTAsync_SSLOptions::default(),
            trust_store: CString::new("").unwrap(),
            key_store: CString::new("").unwrap(),
            private_key: CString::new("").unwrap(),
            private_key_password: CString::new("").unwrap(),
            enabled_cipher_suites: CString::new("").unwrap(),
        };
        SslOptions::fixup(opts)
    }

    // The C library expects unset values in the SSL options struct to be
    // NULL rather than empty string.
    fn c_str(str: &CString) -> *const c_char {
        if str.to_bytes().len() == 0 { ptr::null() } else { str.as_ptr() }
    }

    // Updates the underlying C structure to match the cached strings.
    fn fixup(mut opts: SslOptions) -> SslOptions {
        opts.copts.trustStore = SslOptions::c_str(&opts.trust_store);
        opts.copts.keyStore = SslOptions::c_str(&opts.key_store);
        opts.copts.privateKey = SslOptions::c_str(&opts.private_key);
        opts.copts.privateKeyPassword = SslOptions::c_str(&opts.private_key_password);
        opts.copts.enabledCipherSuites = SslOptions::c_str(&opts.enabled_cipher_suites);
        opts
    }

    pub fn trust_store(&self) -> String {
        self.trust_store.to_str().unwrap().to_string()
    }

    pub fn key_store(&self) -> String {
        self.key_store.to_str().unwrap().to_string()
    }

    pub fn private_key(&self) -> String {
        self.private_key.to_str().unwrap().to_string()
    }

    pub fn private_key_password(&self) -> String {
        self.private_key_password.to_str().unwrap().to_string()
    }

    pub fn enabled_cipher_suites(&self) -> String {
        self.enabled_cipher_suites.to_str().unwrap().to_string()
    }

    pub fn enable_server_cert_auth(&self) -> bool {
        self.copts.enableServerCertAuth != 0
    }
}

impl Clone for SslOptions {
    fn clone(&self) -> SslOptions {
        let ssl = SslOptions {
            copts: self.copts.clone(),
            trust_store: self.trust_store.clone(),
            key_store: self.key_store.clone(),
            private_key: self.private_key.clone(),
            private_key_password: self.private_key_password.clone(),
            enabled_cipher_suites: self.enabled_cipher_suites.clone(),
        };
        SslOptions::fixup(ssl)
    }
}


#[derive(Debug)]
pub struct SslOptionsBuilder {
    trust_store: String,
    key_store: String,
    private_key: String,
    private_key_password: String,
    enabled_cipher_suites: String,
    enable_server_cert_auth: bool,
}

impl SslOptionsBuilder {
    pub fn new() -> SslOptionsBuilder {
        SslOptionsBuilder {
            trust_store: "".to_string(),
            key_store: "".to_string(),
            private_key: "".to_string(),
            private_key_password: "".to_string(),
            enabled_cipher_suites: "".to_string(),
            enable_server_cert_auth: true,
        }
    }

    pub fn trust_store(&mut self, trust_store: &str) -> &mut SslOptionsBuilder {
        self.trust_store = trust_store.to_string();
        self
    }

    pub fn key_store(&mut self, key_store: &str) -> &mut SslOptionsBuilder {
        self.key_store = key_store.to_string();
        self
    }

    pub fn private_key(&mut self, private_key: &str) -> &mut SslOptionsBuilder {
        self.private_key = private_key.to_string();
        self
    }

    pub fn private_key_password(&mut self, private_key_password: &str) -> &mut SslOptionsBuilder {
        self.private_key_password = private_key_password.to_string();
        self
    }

    pub fn enabled_cipher_suites(&mut self, enabled_cipher_suites: &str) -> &mut SslOptionsBuilder {
        self.enabled_cipher_suites = enabled_cipher_suites.to_string();
        self
    }

    pub fn finalize(&self) -> SslOptions {
        let mut opts = SslOptions {
            copts: ffi::MQTTAsync_SSLOptions::default(),
            trust_store: CString::new(self.trust_store.clone()).unwrap(),
            key_store: CString::new(self.key_store.clone()).unwrap(),
            private_key: CString::new(self.private_key.clone()).unwrap(),
            private_key_password: CString::new(self.private_key_password.clone()).unwrap(),
            enabled_cipher_suites: CString::new(self.enabled_cipher_suites.clone()).unwrap(),
        };
        opts.copts.enableServerCertAuth = if self.enable_server_cert_auth { 1 } else { 0 };
        SslOptions::fixup(opts)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Unit Tests

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
        assert_eq!(0, opts.copts.struct_version);
        assert_eq!(ptr::null(), opts.copts.trustStore);
        // TODO: Check the other strings
    }

    #[test]
    fn test_builder_dflt() {
        let opts = SslOptionsBuilder::new()
            .finalize();

        assert_eq!(STRUCT_ID, opts.copts.struct_id);
        assert_eq!(0, opts.copts.struct_version);
        assert_eq!(ptr::null(), opts.copts.trustStore);
        // TODO: Check the other strings
    }

    #[test]
    fn test_builder_trust_store() {
        const TRUST_STORE: &str = "some_file.crt";
        let opts = SslOptionsBuilder::new()
            .trust_store(TRUST_STORE)
            .finalize();

        assert_eq!(TRUST_STORE, opts.trust_store.to_str().unwrap());
        let ts = unsafe { CStr::from_ptr(opts.copts.trustStore) };
        assert_eq!(TRUST_STORE, ts.to_str().unwrap());
    }

    #[test]
    fn test_builder_key_store() {
        const KEY_STORE: &str = "some_file.crt";
        let opts = SslOptionsBuilder::new()
            .key_store(KEY_STORE)
            .finalize();

        assert_eq!(KEY_STORE, opts.key_store.to_str().unwrap());
    }

    // TODO: Test the other builder initializers

    #[test]
    fn test_copy() {
        const TRUST_STORE: &str = "some_file.crt";
        let org_opts = SslOptionsBuilder::new()
            .trust_store(TRUST_STORE)
            .finalize();

        let opts = org_opts;
        assert_eq!(TRUST_STORE, opts.trust_store.to_str().unwrap());
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

        assert_eq!(TRUST_STORE, opts.trust_store.to_str().unwrap());
        let ts = unsafe { CStr::from_ptr(opts.copts.trustStore) };
        assert_eq!(TRUST_STORE, ts.to_str().unwrap());
    }
}

