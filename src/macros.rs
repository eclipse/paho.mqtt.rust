// macros.rs
// 
// This file is part of the Eclipse Paho MQTT Rust Client library.
//
// This contains the macro definitions for the crate.
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

#![macro_use]

/// Return an error from a function. 

macro_rules! fail {
    ($expr:expr) => (
        return Err(::std::convert::From::from($expr));
    )
}

/*
/// Create a vector of string from str references.
macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
*/
