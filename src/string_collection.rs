// string_collection.rs
// 
// This file is part of the Eclipse Paho MQTT Rust Client library.
//
// A string_collection is a helper to bridge between a collection of 
// strings in Rust to an array of NUL terminated char string pointers that
// the C library expects.
//
// It is useful when a C API takes a `const char *arg[]` parameter.
//

/*******************************************************************************
 * Copyright (c) 2017 Frank Pagliughi <fpagliughi@mindspring.com>
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
	/// Creates a StringCollection from a vector of strings.
	pub fn new(coll: &Vec<String>) -> StringCollection {
		let sc = StringCollection {
			coll: StringCollection::to_cstring(coll),
			c_coll: Vec::new(),
		};
		StringCollection::fixup(sc)
	}

	// Convert a vector of strings to a vector of CStrings.
	fn to_cstring(coll: &Vec<String>) -> Vec<CString> {
 		coll.iter()
			.map(|s| CString::new(s.as_str()).unwrap())
			.collect()
	}

	// Convert a vector of CString's to a vector of C char pointers.
	// Note that the pointers are invalidated if the original vector or
	// any of the strings in it change.
	fn to_c_vec(sv: &Vec<CString>) -> Vec<*const c_char> {
		sv.iter()
			.map(|cs| cs.as_ptr())
			.collect()
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


