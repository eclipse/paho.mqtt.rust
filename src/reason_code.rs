// paho-mqtt/src/reason_code.rs
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2019-2022 Frank Pagliughi <fpagliughi@mindspring.com>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v20.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

//! The Reason Code module for the Paho MQTT Rust client library.

use std::{ffi::CStr, fmt};

/// MQTT v5 single-byte reason codes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ReasonCode {
    Success = 0, // also: NormalDisconnection & GrantedQos0
    GrantedQos1 = 1,
    GrantedQos2 = 2,
    DisconnectWithWillMessage = 4,
    NoMatchingSubscribers = 16,
    NoSubscriptionFound = 17,
    ContinueAuthentication = 24,
    ReAuthenticate = 25,

    UnspecifiedError = 128,
    MalformedPacket = 129,
    ProtocolError = 130,
    ImplementationSpecificError = 131,
    UnsupportedProtocolVersion = 132,
    ClientIdentifierNotValid = 133,
    BadUserNameOrPassword = 134,
    NotAuthorized = 135,
    ServerUnavailable = 136,
    ServerBusy = 137,
    Banned = 138,
    ServerShuttingDown = 139,
    BadAuthenticationMethod = 140,
    KeepAliveTimeout = 141,
    SessionTakenOver = 142,
    TopicFilterInvalid = 143,
    TopicNameInvalid = 144,
    PacketIdentifierInUse = 145,
    PacketIdentifierNotFound = 146,
    ReceiveMaximumExceeded = 147,
    TopicAliasInvalid = 148,
    PacketTooLarge = 149,
    MessageRateTooHigh = 150,
    QuotaExceeded = 151,
    AdministrativeAction = 152,
    PayloadFormatInvalid = 153,
    RetainNotSupported = 154,
    QosNotSupported = 155,
    UseAnotherServer = 156,
    ServerMoved = 157,
    SharedSubscriptionsNotSupported = 158,
    ConnectionRateExceeded = 159,
    MaximumConnectTime = 160,
    SubscriptionIdentifiersNotSupported = 161,
    WildcardSubscriptionsNotSupported = 162,
    MqttppV3Code = 255, // This is not a protocol code; used internally by the library
}

// Some aliased ReasonCode values
impl ReasonCode {
    /// Reason code for a normal disconnect
    #[allow(non_upper_case_globals)]
    pub const NormalDisconnection: ReasonCode = ReasonCode::Success;

    /// Reason code for QoS 0 granted
    #[allow(non_upper_case_globals)]
    pub const GrantedQos0: ReasonCode = ReasonCode::Success;
}

impl ReasonCode {
    /// Reason codes less than 0x80 indicate a successful operation.
    pub fn is_ok(&self) -> bool {
        (*self as u32) < 0x80
    }

    /// Reason codes of 0x80 or greater indicate failure.
    pub fn is_err(&self) -> bool {
        (*self as u32) >= 0x80
    }
}

impl Default for ReasonCode {
    fn default() -> Self {
        ReasonCode::Success
    }
}

impl From<ffi::MQTTReasonCodes> for ReasonCode {
    fn from(code: ffi::MQTTReasonCodes) -> Self {
        use ReasonCode::*;
        match code {
            0 => Success, // also: NormalDisconnection & GrantedQos0
            1 => GrantedQos1,
            2 => GrantedQos2,
            4 => DisconnectWithWillMessage,
            16 => NoMatchingSubscribers,
            17 => NoSubscriptionFound,
            24 => ContinueAuthentication,
            25 => ReAuthenticate,

            128 => UnspecifiedError,
            129 => MalformedPacket,
            130 => ProtocolError,
            131 => ImplementationSpecificError,
            132 => UnsupportedProtocolVersion,
            133 => ClientIdentifierNotValid,
            134 => BadUserNameOrPassword,
            135 => NotAuthorized,
            136 => ServerUnavailable,
            137 => ServerBusy,
            138 => Banned,
            139 => ServerShuttingDown,
            140 => BadAuthenticationMethod,
            141 => KeepAliveTimeout,
            142 => SessionTakenOver,
            143 => TopicFilterInvalid,
            144 => TopicNameInvalid,
            145 => PacketIdentifierInUse,
            146 => PacketIdentifierNotFound,
            147 => ReceiveMaximumExceeded,
            148 => TopicAliasInvalid,
            149 => PacketTooLarge,
            150 => MessageRateTooHigh,
            151 => QuotaExceeded,
            152 => AdministrativeAction,
            153 => PayloadFormatInvalid,
            154 => RetainNotSupported,
            155 => QosNotSupported,
            156 => UseAnotherServer,
            157 => ServerMoved,
            158 => SharedSubscriptionsNotSupported,
            159 => ConnectionRateExceeded,
            160 => MaximumConnectTime,
            161 => SubscriptionIdentifiersNotSupported,
            162 => WildcardSubscriptionsNotSupported,
            _ => MqttppV3Code, // This is not a protocol code; used internally by the library
        }
    }
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let p = ffi::MQTTReasonCode_toString(*self as ffi::MQTTReasonCodes);

            if p.is_null() {
                write!(f, "Unknown")
            }
            else {
                let s = CStr::from_ptr(p).to_string_lossy();
                write!(f, "{}", s)
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as() {
        assert_eq!(
            ReasonCode::Success as ffi::MQTTReasonCodes,
            ffi::MQTTReasonCodes_MQTTREASONCODE_SUCCESS
        );

        assert_eq!(
            ReasonCode::DisconnectWithWillMessage as ffi::MQTTReasonCodes,
            ffi::MQTTReasonCodes_MQTTREASONCODE_DISCONNECT_WITH_WILL_MESSAGE
        );

        assert_eq!(
            ReasonCode::UnspecifiedError as ffi::MQTTReasonCodes,
            ffi::MQTTReasonCodes_MQTTREASONCODE_UNSPECIFIED_ERROR
        );

        assert_eq!(
            ReasonCode::MaximumConnectTime as ffi::MQTTReasonCodes,
            ffi::MQTTReasonCodes_MQTTREASONCODE_MAXIMUM_CONNECT_TIME
        );
    }

    #[test]
    fn test_from() {
        assert_eq!(
            ReasonCode::Success,
            ReasonCode::from(ffi::MQTTReasonCodes_MQTTREASONCODE_SUCCESS)
        );

        assert_eq!(
            ReasonCode::DisconnectWithWillMessage,
            ReasonCode::from(ffi::MQTTReasonCodes_MQTTREASONCODE_DISCONNECT_WITH_WILL_MESSAGE)
        );

        assert_eq!(
            ReasonCode::UnspecifiedError,
            ReasonCode::from(ffi::MQTTReasonCodes_MQTTREASONCODE_UNSPECIFIED_ERROR)
        );

        assert_eq!(
            ReasonCode::MaximumConnectTime,
            ReasonCode::from(ffi::MQTTReasonCodes_MQTTREASONCODE_MAXIMUM_CONNECT_TIME)
        );
    }

    #[test]
    fn test_is_ok() {
        assert!(ReasonCode::Success.is_ok());
        assert!(ReasonCode::ReAuthenticate.is_ok());

        assert!(!ReasonCode::UnspecifiedError.is_ok());
        assert!(!ReasonCode::ServerMoved.is_ok());
    }

    #[test]
    fn test_is_err() {
        assert!(!ReasonCode::Success.is_err());
        assert!(!ReasonCode::ReAuthenticate.is_err());

        assert!(ReasonCode::UnspecifiedError.is_err());
        assert!(ReasonCode::ServerMoved.is_err());
    }

    // Note: These strings are from the Paho C library in MQTTReasonCodes.c
    // They may change between versions, but we mainly want to see that
    // the Display trait is working.
    #[test]
    fn test_display() {
        let s = format!("{}", ReasonCode::GrantedQos2);
        assert_eq!(&s, "Granted QoS 2");

        let s = format!("{}", ReasonCode::UnspecifiedError);
        assert_eq!(&s, "Unspecified error");

        let s = format!("{}", ReasonCode::Banned);
        assert_eq!(&s, "Banned");
    }
}
