// properties.rs
//
// The set of properties in an MQTT v5 packet.
//
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

//! MQTT v5 properties.

use std::{mem, ptr};
use std::ffi::CString;
use std::os::raw::{c_int, c_char};

use ffi;
use errors::{MqttResult, MqttError};

/// The Property `value` union type.
pub type Value = ffi::MQTTProperty__bindgen_ty_1;

/// The struct to encapsulate property string values.
type LenString = ffi::MQTTLenString;


#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PropertyType {
    BYTE = 0,
    TWO_BYTE_INTEGER = 1,
    FOUR_BYTE_INTEGER = 2,
    VARIABLE_BYTE_INTEGER = 3,
    BINARY_DATA = 4,
    UTF_8_ENCODED_STRING = 5,
    UTF_8_STRING_PAIR = 6,
}

pub type Type = ffi::MQTTPropertyTypes;

impl PropertyType {
    fn from_ctype(typ: Type) -> PropertyType {
        unsafe { mem::transmute(typ) }
    }
}


/// The enumerated codes for the MQTT v5 properties.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PropertyCode {
    PAYLOAD_FORMAT_INDICATOR = 1,
    MESSAGE_EXPIRY_INTERVAL = 2,
    CONTENT_TYPE = 3,
    RESPONSE_TOPIC = 8,
    CORRELATION_DATA = 9,
    SUBSCRIPTION_IDENTIFIER = 11,
    SESSION_EXPIRY_INTERVAL = 17,
    ASSIGNED_CLIENT_IDENTIFER = 18,
    SERVER_KEEP_ALIVE = 19,
    AUTHENTICATION_METHOD = 21,
    AUTHENTICATION_DATA = 22,
    REQUEST_PROBLEM_INFORMATION = 23,
    WILL_DELAY_INTERVAL = 24,
    REQUEST_RESPONSE_INFORMATION = 25,
    RESPONSE_INFORMATION = 26,
    SERVER_REFERENCE = 28,
    REASON_STRING = 31,
    RECEIVE_MAXIMUM = 33,
    TOPIC_ALIAS_MAXIMUM = 34,
    TOPIC_ALIAS = 35,
    MAXIMUM_QOS = 36,
    RETAIN_AVAILABLE = 37,
    USER_PROPERTY = 38,
    MAXIMUM_PACKET_SIZE = 39,
    WILDCARD_SUBSCRIPTION_AVAILABLE = 40,
    SUBSCRIPTION_IDENTIFIERS_AVAILABLE = 41,
    SHARED_SUBSCRIPTION_AVAILABLE = 42,
}

type Code = ffi::MQTTPropertyCodes;

impl PropertyCode {
    fn from_code(code: Code) -> PropertyCode {
        unsafe { mem::transmute(code) }
    }
}

/////////////////////////////////////////////////////////////////////////////

/// A single MQTT v5 property.
pub struct Property {
    pub(crate) cprop: ffi::MQTTProperty,
}

impl Property {
    pub fn new_int(code: PropertyCode, val: i32) -> Option<Property> {
        let mut prop = Property {
            cprop: ffi::MQTTProperty {
                identifier: code as Code,
                value: Value {
                    byte: val as u8
                },
            }
        };

        match Property::get_type(code) {
            PropertyType::BYTE => (),
            PropertyType::TWO_BYTE_INTEGER => prop.cprop.value.integer2 = val as u16,
            PropertyType::FOUR_BYTE_INTEGER => prop.cprop.value.integer4 = val as u32,
            _ => return None,
        }

        Some(prop)
    }

    pub fn new_binary<V>(code: PropertyCode, bin: V) -> Option<Property>
        where V: Into<Vec<u8>>
    {
        if Property::get_type(code) == PropertyType::BINARY_DATA {
            let mut v = bin.into();
            v.shrink_to_fit();

            let n = v.len();
            let p = v.as_mut_ptr() as *mut c_char;
            mem::forget(v);

            debug!("Creating binary property, {} bytes", n);
            Some(Property::new_string_binary(code, p, n, ptr::null_mut(), 0))
        }
        else {
            None
        }
    }

    pub fn new_string(code: PropertyCode, s: &str) -> Option<Property> {
        if Property::get_type(code) == PropertyType::UTF_8_ENCODED_STRING {
            let n = s.len();
            let p = CString::new(s).unwrap().into_raw();

            debug!("Creating string property, {} bytes", n);
            Some(Property::new_string_binary(code, p, n, ptr::null_mut(), 0))
        }
        else {
            None
        }
    }

    pub fn new_string_pair(code: PropertyCode, key: &str, val: &str) -> Option<Property> {
        if Property::get_type(code) == PropertyType::UTF_8_STRING_PAIR {
            let nkey = key.len();
            let pkey = CString::new(key).unwrap().into_raw();

            let nval = val.len();
            let pval = CString::new(val).unwrap().into_raw();

            debug!("Creating string pair property, {}/{} bytes", nkey, nval);
            Some(Property::new_string_binary(code, pkey, nkey, pval, nval))
        }
        else {
            None
        }
    }

    /// Creates a property from a C lib MQTTProperty struct.
    /// As this is originating from the C lib, we don't want to
    fn from_c_property(cprop: &ffi::MQTTProperty) -> Option<Property> {
        let mut cprop = cprop.clone();

        unsafe {
            let typ = Property::get_type_from_code(cprop.identifier);

            let mut pdata = cprop.value.__bindgen_anon_1.data.data;
            let n = cprop.value.__bindgen_anon_1.data.len as usize;

            match typ {
                PropertyType::BINARY_DATA => {
                    if pdata.is_null() { return None; }
                    let v = Vec::from_raw_parts(pdata, n, n);
                    let mut vc = v.clone();
                    pdata = vc.as_mut_ptr() as *mut c_char;
                    mem::forget(v);
                    mem::forget(vc);
                },
                PropertyType::UTF_8_ENCODED_STRING => {
                    if pdata.is_null() { return None; }
                    let v = Vec::from_raw_parts(pdata as *mut u8, n, n);
                    let sr = CString::new(v.clone());
                    if sr.is_err() { return None; }
                    pdata = sr.unwrap().into_raw();
                    mem::forget(v);
                },
                PropertyType::UTF_8_STRING_PAIR => {
                    let pvalue = cprop.value.__bindgen_anon_1.value.data;
                    if pdata.is_null() || pvalue.is_null() { return None; }

                    let v = Vec::from_raw_parts(pdata as *mut u8, n, n);
                    let sr = CString::new(v.clone());
                    if sr.is_err() { return None; }
                    pdata = sr.unwrap().into_raw();
                    mem::forget(v);

                    let n = cprop.value.__bindgen_anon_1.value.len as usize;
                    let v = Vec::from_raw_parts(pvalue as *mut u8, n, n);
                    let sr = CString::new(v.clone());
                    if sr.is_err() { return None; }
                    cprop.value.__bindgen_anon_1.value.data = sr.unwrap().into_raw();
                    mem::forget(v);
                },
                _ => (),
            }

            // Lengths are the same as the originals
            cprop.value.__bindgen_anon_1.data.data = pdata;

        }
        Some(Property { cprop, })
    }

    /// Creates a new string, string pair, or binary property given the raw
    /// pointers and sizes.
    /// This is a low-level, internal call to create a preperty that contains
    /// dynamic data. It does no error checking; it simply assembles the
    /// struct.
    fn new_string_binary(code: PropertyCode, pdata: *mut c_char, ndata: usize,
                         pval: *mut c_char, nval: usize) -> Property {
        Property {
            cprop: ffi::MQTTProperty {
                identifier: code as Code,
                value: Value {
                    __bindgen_anon_1: ffi::MQTTProperty__bindgen_ty_1__bindgen_ty_1 {
                        data: LenString {
                            len: ndata as c_int,
                            data: pdata,
                        },
                        value: LenString {
                            len: nval as c_int,
                            data: pval,
                        },
                    },
                },
            },
        }
    }

    /// Get the property type from the code identifier.
    pub fn get_type(code: PropertyCode) -> PropertyType {
        let typ = unsafe { ffi::MQTTProperty_getType(code as Code) as Type };
        PropertyType::from_ctype(typ)
    }

    /// Gets the property type from an integer code identifier.
    fn get_type_from_code(code: Code) -> PropertyType {
        let pcode: PropertyCode = PropertyCode::from_code(code);
        Property::get_type(pcode)
    }

    /// Gets the property value as an integer.
    /// This extracts an integer value from the property. It works with any
    /// of the int types, one, two, or four bytes.
    /// If the Property contains an integer type it will be returned as
    /// Some(val), otherwise it will return None.
    pub fn get_int(&self) -> Option<i32> {
        unsafe {
            match Property::get_type_from_code(self.cprop.identifier) {
                PropertyType::BYTE => Some(self.cprop.value.byte as i32),
                PropertyType::TWO_BYTE_INTEGER => Some(self.cprop.value.integer2 as i32),
                PropertyType::FOUR_BYTE_INTEGER => Some(self.cprop.value.integer4 as i32),
                _ => None
            }
        }
    }

    pub fn get_binary(&self) -> Option<Vec<u8>> {
        unsafe {
            if Property::get_type_from_code(self.cprop.identifier) == PropertyType::BINARY_DATA {
                let n = self.cprop.value.__bindgen_anon_1.data.len as usize;
                let p = self.cprop.value.__bindgen_anon_1.data.data as *mut u8;
                let v = Vec::from_raw_parts(p, n, n);
                let vc = v.clone();
                mem::forget(v);
                Some(vc)
            }
            else {
                None
            }
        }
    }

    pub fn get_string(&self) -> Option<String> {
        unsafe {
            if Property::get_type_from_code(self.cprop.identifier) == PropertyType::UTF_8_ENCODED_STRING {
                let s = CString::from_raw(self.cprop.value.__bindgen_anon_1.data.data);
                let sc = s.clone();
                s.into_raw();
                sc.into_string().ok()
            }
            else {
                None
            }
        }
    }

    pub fn get_string_pair(&self) -> Option<(String,String)> {
        unsafe {
            if Property::get_type_from_code(self.cprop.identifier) == PropertyType::UTF_8_STRING_PAIR {
                let s = CString::from_raw(self.cprop.value.__bindgen_anon_1.data.data);
                let sc = s.clone();
                s.into_raw();
                let keyopt = sc.into_string().ok();

                let s = CString::from_raw(self.cprop.value.__bindgen_anon_1.value.data);
                let sc = s.clone();
                s.into_raw();
                let valopt = sc.into_string().ok();
                keyopt.and_then(|key| valopt.map(|val| (key, val)))
            }
            else {
                None
            }
        }
    }
}

impl Drop for Property {
    /// Drops the property.
    /// For string any binary types, the heap memory will be freed.
    fn drop(&mut self) {
        unsafe {
            match Property::get_type_from_code(self.cprop.identifier) {
                PropertyType::BINARY_DATA => {
                    debug!("Dropping binary property: {:?}", self.cprop.value.__bindgen_anon_1.data.data);
                    let n = self.cprop.value.__bindgen_anon_1.data.len as usize;
                    let _ = Vec::from_raw_parts(self.cprop.value.__bindgen_anon_1.data.data, n, n);
                },
                PropertyType::UTF_8_ENCODED_STRING => {
                    debug!("Dropping string property: {:?}", self.cprop.value.__bindgen_anon_1.data.data);
                    let _ = CString::from_raw(self.cprop.value.__bindgen_anon_1.data.data);
                },
                PropertyType::UTF_8_STRING_PAIR => {
                    debug!("Dropping string pair property: {:?}, {:?}", self.cprop.value.__bindgen_anon_1.data.data, self.cprop.value.__bindgen_anon_1.value.data);
                    let _ = CString::from_raw(self.cprop.value.__bindgen_anon_1.data.data);
                    let _ = CString::from_raw(self.cprop.value.__bindgen_anon_1.value.data);
                },
                _ => (),
            }
        }
    }
}

impl Clone for Property {
    /// Creates a clone of the property.
    /// For string any binary properties, this also clones the heap memory
    /// so that each property is managing separate allocations.
    fn clone(&self) -> Self {
        let mut cprop = self.cprop.clone();

        unsafe {
            let typ = Property::get_type_from_code(self.cprop.identifier);

            match typ {
                PropertyType::BINARY_DATA => {
                    // TODO: Can we just do a low-level mem copy?
                    let n = cprop.value.__bindgen_anon_1.data.len as usize;
                    let v = Vec::from_raw_parts(cprop.value.__bindgen_anon_1.data.data, n, n);
                    let mut vc = v.clone();
                    let p = vc.as_mut_ptr() as *mut c_char;
                    cprop.value.__bindgen_anon_1.data.data = p;
                    mem::forget(v);
                    mem::forget(vc);
                },
                PropertyType::UTF_8_ENCODED_STRING => {
                    let s = CString::from_raw(cprop.value.__bindgen_anon_1.data.data);
                    let sc = s.clone();
                    s.into_raw();
                    cprop.value.__bindgen_anon_1.data.data = sc.into_raw();
                },
                PropertyType::UTF_8_STRING_PAIR => {
                    let s = CString::from_raw(cprop.value.__bindgen_anon_1.data.data);
                    let sc = s.clone();
                    cprop.value.__bindgen_anon_1.data.data = sc.into_raw();
                    s.into_raw();

                    let s = CString::from_raw(cprop.value.__bindgen_anon_1.value.data);
                    let sc = s.clone();
                    cprop.value.__bindgen_anon_1.value.data = sc.into_raw();
                    s.into_raw();
                },
                _ => (),
            }
        }
        Property { cprop, }
    }
}


/////////////////////////////////////////////////////////////////////////////
// Properties

/// A list of MQTT v5 properties.
///
/// This is a collection of properties that can be added to outgoing packets
/// or retrieved from incoming packets.
#[derive(Debug)]
pub struct Properties {
    pub(crate) cprops: ffi::MQTTProperties,
}

impl Properties {
    /// Creates a new, empty collection of properties.
    pub fn new() -> Properties {
        Properties::default()
    }

    pub fn from_c_struct(cprops: &ffi::MQTTProperties) -> Properties {
        // This does a deep copy in the C lib
        let cprops = unsafe { ffi::MQTTProperties_copy(cprops) };
        Properties { cprops, }
    }

    /// Gets the number of property items in the collection.
    pub fn len(&self) -> usize {
        self.cprops.count as usize
    }

    /// Determines if the property list has no items in it.
    pub fn is_empty(&self) -> bool {
        self.cprops.count == 0
    }

    /// Gets the number of bytes required for the serialized list on
    /// the wire.
    pub fn byte_len(&self) -> usize {
        let p = &self.cprops as *const _ as *mut ffi::MQTTProperties;
        unsafe { ffi::MQTTProperties_len(p) as usize }
    }

    /// Removes all the items from the property list.
    fn clear(&mut self) {
        unsafe { ffi::MQTTProperties_free(&mut self.cprops) };
        self.cprops = ffi::MQTTProperties::default();
    }

    /// Adds a property to the colletion.
    pub fn push(&mut self, prop: Property) {    // TODO return a Result
        let _rc = unsafe { ffi::MQTTProperties_add(&mut self.cprops, &prop.cprop) };
        mem::forget(prop);
    }

    pub fn get(&self, code: PropertyCode) -> Option<Property> {
        let ps = &self.cprops as *const _ as *mut ffi::MQTTProperties;
        unsafe {
            let p = ffi::MQTTProperties_getPropertyAt(ps, code as Code, 0);
            if p.is_null() { None } else { Property::from_c_property(&*p) }
        }
    }

    pub fn push_int(&mut self, code: PropertyCode, val: i32) -> MqttResult<()> {
        let prop = match Property::new_int(code, val) {
            Some(p) => p,
            None => return Err(MqttError::from(ffi::MQTTASYNC_FAILURE)),
        };
        self.push(prop);
        Ok(())
    }

    pub fn push_binary<V>(&mut self, code: PropertyCode, bin: V) -> MqttResult<()>
            where V: Into<Vec<u8>> {
        let prop = match Property::new_binary(code, bin) {
            Some(p) => p,
            None => return Err(MqttError::from(ffi::MQTTASYNC_FAILURE)),
        };
        self.push(prop);
        Ok(())
    }

    pub fn push_string(&mut self, code: PropertyCode, s: &str) -> MqttResult<()> {
        let prop = match Property::new_string(code, s) {
            Some(p) => p,
            None => return Err(MqttError::from(ffi::MQTTASYNC_FAILURE)),
        };
        self.push(prop);
        Ok(())
    }

    pub fn push_string_pair(&mut self, code: PropertyCode, key: &str, val: &str) -> MqttResult<()> {
        let prop = match Property::new_string_pair(code, key, val) {
            Some(p) => p,
            None => return Err(MqttError::from(ffi::MQTTASYNC_FAILURE)),
        };
        self.push(prop);
        Ok(())
    }

    pub fn get_int(&self, code: PropertyCode) -> Option<i32> {
        self.get(code).and_then(|prop| prop.get_int())
    }

    pub fn get_binary(&self, code: PropertyCode) -> Option<Vec<u8>> {
        self.get(code).and_then(|prop| prop.get_binary())
    }

    pub fn get_string(&self, code: PropertyCode) -> Option<String> {
        self.get(code).and_then(|prop| prop.get_string())
    }

    pub fn get_string_pair(&self, code: PropertyCode) -> Option<(String,String)> {
        self.get(code).and_then(|prop| prop.get_string_pair())
    }
}

impl Default for Properties {
    fn default() -> Self {
        Properties {
            cprops: ffi::MQTTProperties::default(),
        }
    }
}

unsafe impl Send for Properties {}

impl Clone for Properties {
    /// Creates a clone of the property.
    /// For string any binary properties, this also clones the heap memory
    /// so that each property is managing separate allocations.
    fn clone(&self) -> Self {
        // This does a deep copy in the C lib
        let cprops = unsafe { ffi::MQTTProperties_copy(&self.cprops) };
        Properties { cprops, }
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        // This deletes the collection and all the elements in it.
        unsafe { ffi::MQTTProperties_free(&mut self.cprops) };
    }
}


/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_new_property_byte() {
        let val = 1;
        let prop = Property::new_int(PropertyCode::PAYLOAD_FORMAT_INDICATOR, val).unwrap();

        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::PAYLOAD_FORMAT_INDICATOR as Code);
            assert_eq!(prop.cprop.value.byte, val as u8);
        }

        assert_eq!(prop.get_int(), Some(val));
    }

    #[test]
    fn test_new_property_i16() {
        let val = 1024;
        let prop = Property::new_int(PropertyCode::RECEIVE_MAXIMUM, val).unwrap();

        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::RECEIVE_MAXIMUM as Code);
            assert_eq!(prop.cprop.value.integer2, val as u16);
        }
    }

    #[test]
    fn test_new_property_i32() {
        let val = 1024*1024;
        let prop = Property::new_int(PropertyCode::MESSAGE_EXPIRY_INTERVAL, val).unwrap();

        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::MESSAGE_EXPIRY_INTERVAL as Code);
            assert_eq!(prop.cprop.value.integer4, val as u32);
        }
    }

    #[test]
    fn test_new_property_binary() {
        let val = "12345";  // Anything that can be converted into Vec<u8>
        let prop = Property::new_binary(PropertyCode::CORRELATION_DATA, val).unwrap();

        // We should have non-null data pointer, null value pointer
        unsafe {
            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, 5);

            assert!(prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, 0);
        }
    }

    #[test]
    fn test_new_property_string() {
        let val = "replies/myqueue";
        let prop = Property::new_string(PropertyCode::RESPONSE_TOPIC, val).unwrap();

        // We should have non-null data pointer, null value pointer
        unsafe {
            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, val.len() as c_int);

            let s = CStr::from_ptr(prop.cprop.value.__bindgen_anon_1.data.data);
            assert_eq!(val, s.to_str().unwrap());

            assert!(prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, 0);
        }

        assert_eq!(val, &prop.get_string().unwrap());
    }

    #[test]
    fn test_new_property_string_pair() {
        let key = "mykey";
        let val = "myvalue";
        let prop = Property::new_string_pair(PropertyCode::USER_PROPERTY, key, val).unwrap();

        // We should have non-null data and value pointers
        unsafe {
            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, key.len() as c_int);

            let s = CStr::from_ptr(prop.cprop.value.__bindgen_anon_1.data.data);
            assert_eq!(key, s.to_str().unwrap());

            assert!(!prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, val.len() as c_int);

            let s = CStr::from_ptr(prop.cprop.value.__bindgen_anon_1.value.data);
            assert_eq!(val, s.to_str().unwrap());
        }

        let (keys, vals) = prop.get_string_pair().unwrap();

        assert_eq!(key, &keys);
        assert_eq!(val, &vals);
    }

    #[test]
    fn test_move_property_int() {
        let val = 1024*1024;

        let org_prop = Property::new_int(PropertyCode::MESSAGE_EXPIRY_INTERVAL, val).unwrap();
        unsafe {
            assert_eq!(org_prop.cprop.identifier, PropertyCode::MESSAGE_EXPIRY_INTERVAL as Code);
            assert_eq!(org_prop.cprop.value.integer4, val as u32);
        }

        let prop = org_prop;
        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::MESSAGE_EXPIRY_INTERVAL as Code);
            assert_eq!(prop.cprop.value.integer4, val as u32);
        }
    }


    #[test]
    fn test_move_property_binary() {
        let val = "12345";  // Anything that can be converted into Vec<u8>
        let org_prop = Property::new_binary(PropertyCode::CORRELATION_DATA, val).unwrap();

        let prop = org_prop;

        // We should have non-null data pointer, null value pointer
        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::CORRELATION_DATA as Code);

            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, 5);

            assert!(prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, 0);
        }
    }


    #[test]
    fn test_move_property_string() {
        let val = "replies/myqueue";
        let org_prop = Property::new_string(PropertyCode::RESPONSE_TOPIC, val).unwrap();

        let prop = org_prop;

        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::RESPONSE_TOPIC as Code);

            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, val.len() as c_int);

            assert!(prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, 0);
        }
    }

    #[test]
    fn test_clone_property_binary() {
        let val = "12345";  // Anything that can be converted into Vec<u8>
        let org_prop = Property::new_binary(PropertyCode::CORRELATION_DATA, val).unwrap();

        let prop = org_prop.clone();

        // We should have non-null data pointer, null value pointer
        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::CORRELATION_DATA as Code);

            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, 5);

            assert!(prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, 0);

            assert_ne!(org_prop.cprop.value.__bindgen_anon_1.data.data,
                       prop.cprop.value.__bindgen_anon_1.data.data);

        }
    }

    #[test]
    fn test_clone_property_string() {
        let val = "replies/myqueue";
        let org_prop = Property::new_string(PropertyCode::RESPONSE_TOPIC, val).unwrap();

        let prop = org_prop.clone();

        unsafe {
            assert_eq!(prop.cprop.identifier, PropertyCode::RESPONSE_TOPIC as Code);

            assert!(!prop.cprop.value.__bindgen_anon_1.data.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.data.len, val.len() as c_int);

            assert!(prop.cprop.value.__bindgen_anon_1.value.data.is_null());
            assert_eq!(prop.cprop.value.__bindgen_anon_1.value.len, 0);
        }
    }

    /////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_properties_default() {
        let props = Properties::default();

        assert_eq!(props.len(), 0);
        assert!(props.is_empty());

        // Empty list needs 1 byte for zero len byte
        assert_eq!(props.byte_len(), 1);
    }

    #[test]
    fn test_properties_push() {
        let mut props = Properties::new();

        let val = 1;
        let prop = Property::new_int(PropertyCode::PAYLOAD_FORMAT_INDICATOR, val).unwrap();

        props.push(prop);

        assert_eq!(props.len(), 1);
        assert!(!props.is_empty());
    }


    #[test]
    fn test_properties_get_int() {
        let code = PropertyCode::PAYLOAD_FORMAT_INDICATOR;

        let mut props = Properties::new();

        let val = 1;
        let org_prop = Property::new_int(code, val).unwrap();

        props.push(org_prop);

        let prop = props.get(code).unwrap();
        assert_eq!(prop.get_int().unwrap(), val);

    }
}

