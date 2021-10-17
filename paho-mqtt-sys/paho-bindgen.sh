#!/bin/bash
#
# Generates the Paho C bindings for the current host.

if [ -z "$1" ]; then
	printf "USAGE: paho-bindgen.sh <paho-c-version>\n"
	exit 1
fi

VERSION="$1"
HOST_TRIPLE="$(rustc -vV | awk '/^host/ { print $2 }')"

if [ -z "${HOST_TRIPLE}" ]; then
	printf "Unable to determine host target.\n"
	exit 1
fi

printf "Generating bindings for: Paho C v%s, on %s\n" "${VERSION}" "${HOST_TRIPLE}"

bindgen wrapper.h -- -I./paho.mqtt.c/src \
	> bindings/bindings_paho_mqtt_c_${VERSION}-${HOST_TRIPLE}.rs
