// client_persistence.rs
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

extern crate libc;

use std::{ptr, slice, mem};
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char, c_int};

use ffi;

use errors::{MqttResult, /*MqttError,*/};

// TODO: Should we have a specific PersistenceResult/Error?

const PERSISTENCE_SUCCESS: c_int = ffi::MQTTASYNC_SUCCESS as c_int;
const PERSISTENCE_ERROR: c_int = ffi::MQTTCLIENT_PERSISTENCE_ERROR;

/**
 * Trait to implement custom persistence in the client.
 */
pub trait ClientPersistence {
    /**
     * Open and initialize the persistent store.
     * @param client_id The unique client identifier.
     * @param server_uri The address of the server to which the client is
     *                   connected.
     */
    fn open(&mut self, client_id: &str, server_uri: &str) -> MqttResult<()>;
    /**
     * Close the persistence store.
     */
    fn close(&mut self) -> MqttResult<()>;
    /**
     * Put data into the persistence store.
     * @param key The key to the data.
     * @param The data to place into the store.
     */
    fn put(&mut self, key: &str, buffers: Vec<&[u8]>) -> MqttResult<()>;
    /**
     * Gets data from the persistence store.
     * @param key They key for the desired data.
     */
    fn get(&self, key: &str) -> MqttResult<Vec<u8>>;
    /**
     * Removes data for the specified key.
     * @param key The key for the data to remove.
     */
    fn remove(&mut self, key: &str) -> MqttResult<()>;
    /**
     * Gets the keys that are currently in the persistence store
     */
    fn keys(&self) -> MqttResult<Vec<String>>;
    /**
     * Clear the persistence store so that it no longer contains any data.
     */
    fn clear(&mut self) -> MqttResult<()>;
    /**
     * Determines if the persistence store contains the key.
     * @param key The key
     * @return _true_ if the key is found in the store, _false_ otherwise.
     */
    fn contains_key(&self, key: &str) -> bool;
}

/////////////////////////////////////////////////////////////////////////////
///

/**
 * An empty struct used to collect the persistence callback funtions from 
 * the C library.
 * These functions receive the persistence callbacks from the C library and
 * then pass them on to the user-supplied struct which implements the 
 * ClientPersistence trait.
 */
pub struct ClientPersistenceBridge;

impl ClientPersistenceBridge
{
    // Callback from the C library to open the persistence store.
    // On entry, the 'context' has the address of the user's persistence
    // object which is reassigned to the 'handle'.
    // Subsequent calls have the object address as the handle.
    pub unsafe extern "C" fn on_open(handle: *mut *mut c_void,
                                     client_id: *const c_char,
                                     server_uri: *const c_char,
                                     context: *mut c_void) -> c_int {
        trace!("ClientPersistenceBridge::on_open");
        if !handle.is_null() && !client_id.is_null() && !server_uri.is_null() && !context.is_null() {
            let client_id = CStr::from_ptr(client_id).to_str().unwrap();
            let server_uri = CStr::from_ptr(server_uri).to_str().unwrap();

            let persist: &mut Box<ClientPersistence> = mem::transmute(context);

            if let Ok(_) = persist.open(client_id, server_uri) {
                *handle = context;
                return PERSISTENCE_SUCCESS;
            }
        }
        PERSISTENCE_ERROR
    }

    /// Callback from the C library to close the persistence store.
    pub unsafe extern "C" fn on_close(handle: *mut c_void) -> c_int {
        trace!("ClientPersistenceBridge::on_close");
        if handle.is_null() {
            return PERSISTENCE_ERROR;
        }

        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);

        match persist.close() {
            Ok(_) => PERSISTENCE_SUCCESS,
            Err(_) => PERSISTENCE_ERROR,
        }
    }

    /// Callback from the C library to add data to the persistence store
    pub unsafe extern "C" fn on_put(handle: *mut c_void,
                                    key: *mut c_char,
                                    bufcount: c_int,
                                    buffers: *mut *mut c_char,
                                    buflens: *mut c_int) -> c_int {
        trace!("ClientPersistenceBridge::on_put");
        if handle.is_null() || key.is_null() ||
                buffers.is_null() || buflens.is_null() {
            return PERSISTENCE_ERROR;
        }
        if bufcount == 0 {
            return PERSISTENCE_SUCCESS;
        }
        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);
        let key = CStr::from_ptr(key).to_str().unwrap();

        let mut bufs: Vec<&[u8]> = Vec::new();

        for i in 0..bufcount as isize {
            let buf = slice::from_raw_parts_mut(*buffers.offset(i) as *mut u8,
                                                *buflens.offset(i) as usize);
            bufs.push(buf);
        }
        match persist.put(key, bufs) {
            Ok(_)  => PERSISTENCE_SUCCESS,
            Err(_) => PERSISTENCE_ERROR,
        }
    }

    /// Callback from the C library to retrieve data from the
    /// persistence store.
    pub unsafe extern "C" fn on_get(handle: *mut c_void,
                                    key: *mut c_char,
                                    buffer: *mut *mut c_char,
                                    buflen: *mut c_int) -> c_int {
        trace!("ClientPersistenceBridge::on_get");
        if handle.is_null() || key.is_null() ||
                buffer.is_null() || buflen.is_null() {
            return PERSISTENCE_ERROR;
        }
        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);
        let key = CStr::from_ptr(key).to_str().unwrap();

        match persist.get(key) {
            Ok(buf) => {    // buf: Vec<u8>
                let n = buf.len();
                let cbuf = libc::malloc(n) as *mut u8;
                ptr::copy(buf.as_ptr(), cbuf, n);
                *buffer = cbuf as *mut c_char;
                *buflen = n as c_int;
                PERSISTENCE_SUCCESS
            },
            Err(_) => PERSISTENCE_ERROR,
        }
    }

    /// Callback from the C library to delete specific data from the
    /// persistence store.
    pub unsafe extern "C" fn on_remove(handle: *mut c_void,
                                       key: *mut c_char) -> c_int {
        trace!("ClientPersistenceBridge::on_remove");
        if handle.is_null() || key.is_null() {
            return PERSISTENCE_ERROR;
        }
        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);
        let key = CStr::from_ptr(key).to_str().unwrap();

        match persist.remove(key) {
            Ok(_) => PERSISTENCE_SUCCESS,
            Err(_) => PERSISTENCE_ERROR
        }
    }

    /// Callback from the C library to retrieve the set of keys from the
    /// persistence store.
    pub unsafe extern "C" fn on_keys(handle: *mut c_void,
                                     keys: *mut *mut *mut c_char,
                                     nkeys: *mut c_int) -> c_int {
        trace!("ClientPersistenceBridge::on_keys");
        if handle.is_null() || keys.is_null() || nkeys.is_null() {
            return PERSISTENCE_ERROR;
        }

        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);

        *keys = ptr::null_mut();
        *nkeys = 0;

        match persist.keys() {
            Ok(k) => {      // k: Vec<String>
                let n = k.len();
                if n != 0 {
                    // TODO OPTIMIZE: This does a lot of copying
                    let ckeys = libc::malloc(n * mem::size_of::<usize>()) as *mut *mut c_char;
                    for i in 0..n {
                        let s = CString::new(k[i].clone()).unwrap();
                        let sb = s.as_bytes_with_nul();
                        let sn = sb.len();
                        let cbuf = libc::malloc(sn) as *mut c_char;
                        ptr::copy(sb.as_ptr(), cbuf as *mut u8, sn);

                        *ckeys.offset(i as isize) = cbuf;
                    }
                    *keys = ckeys;
                    *nkeys = n as c_int;
                }
                PERSISTENCE_SUCCESS
            },
            Err(_) => PERSISTENCE_ERROR
        }
    }

    /// Callback from the C library to remove all the data from the
    /// persistence store.
    pub unsafe extern "C" fn on_clear(handle: *mut c_void) -> c_int {
        trace!("ClientPersistenceBridge::on_clear");
        if handle.is_null() {
            return PERSISTENCE_ERROR;
        }
        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);

        match persist.clear() {
            Ok(_) => PERSISTENCE_SUCCESS,
            Err(_) => PERSISTENCE_ERROR,
        }
    }

    /// Callback from the C library to determine if the store contains
    /// the specified key.
    pub unsafe extern "C" fn on_contains_key(handle: *mut c_void,
                                             key: *mut c_char) -> c_int {
        trace!("ClientPersistenceBridge::on_contains_key");
        if handle.is_null() || key.is_null() {
            return PERSISTENCE_ERROR;
        }
        let persist: &mut Box<ClientPersistence> = mem::transmute(handle);
        let key = CStr::from_ptr(key).to_str().unwrap();

        if persist.contains_key(key) { 1 } else { 0 }
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    /*
    use super::*;
    //use std::ffi::{CStr};

    struct TestClientPersistence;

    impl ClientPersistence for TestClientPersistence {
        fn open(&self, client_id: &str, server_uri: &str) -> MqttResult<()> {
            Ok(())
        }

        fn close(&self) -> MqttResult<()> {
            Ok(())
        }

        fn clear(&self) -> MqttResult<()> {
            Ok(())
        }

        fn put(&self, key: &str, buffers: Vec<&[u8]>) -> MqttResult<()> {
            Ok(())
        }

        fn get(&self, key: &str) -> MqttResult<&[u8]> {
            let x = b"Bubba";   //: &'static [u8] = &'static [ 0u8, 1u8, 2u8, 3u8 ];
            Ok(x)
        }

        fn remove(&self, key: &str) -> MqttResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_new() {
        let tcp = TestClientPersistence {};
        let tcpp = Box::new(Box::new(tcp));
        let context = Box::into_raw(tcpp);

        let persist: &mut Box<ClientPersistence> = unsafe { mem::transmute(context) };
        /*
        let persist: &mut Box<ClientPersistence> = unsafe { mem::transmute(context) };
        let res = persist.open("clientid", "tcp://localhost:1883");
        assert!(res.is_ok());
        */

        let _ = unsafe { Box::from_raw(context) };
    }
    */
}

