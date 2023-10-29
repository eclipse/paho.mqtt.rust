// types.rs
//
// Miscellaneous small types for the MQTT library.
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2019-2023 Frank Pagliughi <fpagliughi@mindspring.com>
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

use std::os::raw::c_int;

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

/// Quality of Service One: At least once
pub const QOS_1: i32 = 1;

/// Quality of Service Two: Exactly Once
pub const QOS_2: i32 = 2;

/// Supported MQTT protocol versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MqttVersion {
    /// The default v3.1.1 or v3.1
    Default = MQTT_VERSION_DEFAULT,
    /// Version 3.1 (byte 3)
    V3_1 = MQTT_VERSION_3_1,
    /// Version 3.1.1 (byte 4)
    V3_1_1 = MQTT_VERSION_3_1_1,
    /// Version 5
    V5 = MQTT_VERSION_5,
}

impl From<u32> for MqttVersion {
    fn from(ver: u32) -> Self {
        use MqttVersion::*;
        match ver {
            MQTT_VERSION_3_1 => V3_1,
            MQTT_VERSION_3_1_1 => V3_1_1,
            MQTT_VERSION_5 => V5,
            _ => Default,
        }
    }
}

impl From<c_int> for MqttVersion {
    fn from(ver: c_int) -> Self {
        Self::from(ver as u32)
    }
}
