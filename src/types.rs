// types.rs
//
// Miscellaneous small types for the MQTT library.
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

//! MQTT support types

/// The default version to connect with.
/// First try v3.1.1, and if that fails, try v3.1
pub const MQTT_VERSION_DEFAULT: u32 = ffi::MQTTVERSION_DEFAULT;

/// Connect with MQTT v3.1
pub const MQTT_VERSION_3_1: u32 = ffi::MQTTVERSION_3_1;

/// Connect with MQTT v3.1.1
pub const MQTT_VERSION_3_1_1: u32 = ffi::MQTTVERSION_3_1_1;

/// Connect with MQTT v5
pub const MQTT_VERSION_5: u32 = ffi::MQTTVERSION_5;

/// Quality of Service Zero: At most once
pub const QOS_0: i32 = 0;

/// Quality of Service Zero: At least once
pub const QOS_1: i32 = 1;

/// Quality of Service Two: Exactly Once
pub const QOS_2: i32 = 2;
