#!/usr/bin/env bash

# the video node will have one incoming tunnel and two outgoing tunnels
# the video-node under test should be alone in a region. The generator
# appliance should have that region set as its primary region
#
# The output appliance should be in a different region than the video-node
# under test. It will simply discard all incoming streams

generator_appliance="${1?missing argument: Generator appliance}"
output_appliance="${2?missing argument: Output appliance}"
num_outputs="${3?missing argument: number of outputs}"

set -e

inputs="$(edgectl input list | awk '/^ID/ { next } { print $2 }')"
outputs="$(edgectl output list | awk '/^ID/ { next } { print $2 }')"

if ! grep -qw Perftest-0-sdi <<<"$inputs"; then
    edgectl input create "Perftest-0-sdi" --appliance "$generator_appliance" --interface av1 --mode sdi --disable-thumbnails
fi
if ! grep -qw Perftest-0-multicast <<<"$outputs"; then
    edgectl output create "Perftest-0-multicast" --appliance "$generator_appliance" --interface lo --mode rtp --source 127.0.0.1 --dest "224.0.0.44:4444" --input "Perftest-0-sdi"
fi

for i in $(seq 1 "$num_outputs"); do
    input="Perftest-$i-RTP_input"
    output="Perftest-$i-UDP_output"
    if ! grep -qw "$input" <<<"$inputs"; then
        edgectl input create   "$input"  --appliance "$generator_appliance" --interface lo --mode rtp --port 4444 --multicast 224.0.0.44
    fi
    if ! grep -qw "$output" <<<"$outputs"; then
        edgectl output create  "$output" --appliance "$output_appliance" --interface lo --mode udp --dest "127.0.0.1:1234" --input "$input"
    fi
done
