#!/bin/bash
#
# devenv.sh
#
# Sets up a development environment for working on the the Paho Rust Library on Linux,
# using the development tree of the Paho C library.
#
# This is _not_ necessary for building applications against the library once it has
# been installed - it's just for library developers.
#
# Source it into the current shell as:
#     $ source devenv.sh
#

# If readlink doesn't exist, consider using 'realpath', if available.
PAHO_MQTT_C_PATH=$(readlink -e ../paho.mqtt.c)

PAHO_MQTT_C_INC_PATH=${PAHO_MQTT_C_PATH}/src
PAHO_MQTT_C_LIB_PATH=${PAHO_MQTT_C_PATH}/build/output

export DEVELOP=1
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${PAHO_MQTT_C_LIB_PATH}
