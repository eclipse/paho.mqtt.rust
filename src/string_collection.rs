// string_collection.rs
// 
// This file is part of the Eclipse Paho MQTT Rust Client library.
//
// A string_collection is a helper to bridge between a collection of 
// strings in Rust to an array of NUL terminated char string pointers
// that  the C library expects.
//
// It is useful when a C API takes a `const char *arg[]` parameter.
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

use std::ffi::{CString};
use std::os::raw::{c_char};

/// A collection of C-compatible (NUL-terminated) strings that is useful with
/// C API's that require an array of strings, normally specified as:
/// `const char* arr[]` or  `const char** arr`
#[derive(Debug)]
pub struct StringCollection
{
    /// The owned NUL-terminated strings
    coll: Vec<CString>,
    /// A vector cache of pointers into `coll`
    /// This must be updated any time `coll` is modified.
    c_coll: Vec<*const c_char>,
}

impl StringCollection
{
    /// Creates a StringCollection from a vector of string references.
    ///
    /// # Arguments
    ///
    /// `coll` A collection of string references.
    ///
    pub fn new<T>(coll: &[T]) -> StringCollection
        where T: AsRef<str>
    {
        let sc = StringCollection {
            coll: StringCollection::to_cstring(coll),
            c_coll: Vec::new(),
        };
        StringCollection::fixup(sc)
    }

    // Convert a collection of string references to a vector of CStrings.
    fn to_cstring<T>(coll: &[T]) -> Vec<CString>
        where T: AsRef<str>
    {
        coll.iter()
            .map(|s| CString::new(s.as_ref()).unwrap())
            .collect()
    }

    // Convert a collection of CString's to a vector of C char pointers.
    // Note that the pointers are invalidated if the original vector or
    // any of the strings in it change.
    fn to_c_vec(sv: &[CString]) -> Vec<*const c_char> {
        sv.iter().map(|cs| cs.as_ptr()).collect()
    }

    // Updates the cached vector to correspond to the string.
    fn fixup(mut coll: StringCollection) -> StringCollection {
        coll.c_coll = StringCollection::to_c_vec(&coll.coll);
        coll
    }

    /// Gets the number of strings in the collection.
    pub fn len(&self) -> usize {
        self.coll.len()
    }

    /// Gets the collection as a pointer to C string pointers.
    /// This returns a pointer that can be sent to a C API that takes a
    /// pointer to an array of char pointers, like `const char* arr[]`
    /// This function is inherently unsafe. The pointer it returns is only
    /// valid while the collection remains unmodified. In general, it
    /// should be requested when needed and not stored for future use.
    pub fn as_c_arr_ptr(&self) -> *const *const c_char {
        self.c_coll.as_ptr()
    }
}

impl Default for StringCollection
{
    fn default() -> StringCollection {
        let sc = StringCollection {
            coll: Vec::new(),
            c_coll: Vec::new(),
        };
        StringCollection::fixup(sc)
    }
}

impl Clone for StringCollection
{
    fn clone(&self) -> StringCollection {
        let sc = StringCollection {
            coll: self.coll.clone(),
            c_coll: Vec::new(),
        };
        StringCollection::fixup(sc)
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_of_strings {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn test_default() {
        let sc = StringCollection::default();
        assert_eq!(0, sc.len());
    }

    #[test]
    fn test_new() {
        let v = ["string0", "string1", "string2"];
        let n = v.len();

        let sc = StringCollection::new(&v);

        assert_eq!(n, sc.len());
        assert_eq!(n, sc.coll.len());
        assert_eq!(n, sc.c_coll.len());

        assert_eq!(v[0].as_bytes(), sc.coll[0].as_bytes());
        assert_eq!(v[1].as_bytes(), sc.coll[1].as_bytes());
        assert_eq!(v[2].as_bytes(), sc.coll[2].as_bytes());

        assert_eq!(sc.coll[0].as_ptr(), sc.c_coll[0]);
        assert_eq!(sc.coll[1].as_ptr(), sc.c_coll[1]);
        assert_eq!(sc.coll[2].as_ptr(), sc.c_coll[2]);
    }

    #[test]
    fn test_new_from_vec_strings() {
        let v = vec_of_strings![ "string0", "string1", "string2" ];
        let n = v.len();

        let sc = StringCollection::new(&v);

        assert_eq!(n, sc.len());
        assert_eq!(n, sc.coll.len());
        assert_eq!(n, sc.c_coll.len());

        assert_eq!(v[0].as_bytes(), sc.coll[0].as_bytes());
        assert_eq!(v[1].as_bytes(), sc.coll[1].as_bytes());
        assert_eq!(v[2].as_bytes(), sc.coll[2].as_bytes());

        assert_eq!(sc.coll[0].as_ptr(), sc.c_coll[0]);
        assert_eq!(sc.coll[1].as_ptr(), sc.c_coll[1]);
        assert_eq!(sc.coll[2].as_ptr(), sc.c_coll[2]);
    }

    #[test]
    fn test_assign() {
        let v = [ "string0", "string1", "string2" ];;
        let n = v.len();

        let org_sc = StringCollection::new(&v);

        let sc = org_sc;

        assert_eq!(n, sc.len());
        assert_eq!(n, sc.coll.len());
        assert_eq!(n, sc.c_coll.len());

        assert_eq!(v[0].as_bytes(), sc.coll[0].as_bytes());
        assert_eq!(v[1].as_bytes(), sc.coll[1].as_bytes());
        assert_eq!(v[2].as_bytes(), sc.coll[2].as_bytes());

        assert_eq!(sc.coll[0].as_ptr(), sc.c_coll[0]);
        assert_eq!(sc.coll[1].as_ptr(), sc.c_coll[1]);
        assert_eq!(sc.coll[2].as_ptr(), sc.c_coll[2]);
    }

    #[test]
    fn test_clone() {
        let v = [ "string0", "string1", "string2" ];
        let n = v.len();

        let sc = {
            let org_sc = StringCollection::new(&v);
            org_sc.clone()
        };

        assert_eq!(n, sc.len());
        assert_eq!(n, sc.coll.len());
        assert_eq!(n, sc.c_coll.len());

        assert_eq!(v[0].as_bytes(), sc.coll[0].as_bytes());
        assert_eq!(v[1].as_bytes(), sc.coll[1].as_bytes());
        assert_eq!(v[2].as_bytes(), sc.coll[2].as_bytes());

        assert_eq!(sc.coll[0].as_ptr(), sc.c_coll[0]);
        assert_eq!(sc.coll[1].as_ptr(), sc.c_coll[1]);
        assert_eq!(sc.coll[2].as_ptr(), sc.c_coll[2]);
    }
}

