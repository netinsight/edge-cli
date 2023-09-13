#!/usr/bin/env bash

# the video node will have one incoming tunnel and two outgoing tunnels
# the video-node under test should be alone in a region. The generator
# appliance should have that region set as its primary region
#
# The output appliance should be in a different region than the video-node
# under test. It will simply discard all incoming streams

generator_appliance="${1?missing argument: Generator appliance}"
output_appliance="${1?missing argument: Output appliance}"

set -e
set -x

edge input create "Perftest-0-sdi" --appliance "$generator_appliance" --interface av1 --mode sdi 
edge output create "Perftest-0-multicast" --appliance "$generator_appliance" --interface lo --mode rtp --source 127.0.0.1 --dest "224.0.0.44:4444" --input "Perftest-0-sdi"

for i in {1..50}; do 
    edge input create   "Perftest-$i-RTP_input"  --appliance "$generator_appliance" --interface lo --mode rtp --port 4444 --multicast 224.0.0.44
    edge output create  "Perftest-$i-UDP_output" --appliance "$output_appliance" --interface lo --mode udp --dest "127.0.0.1:1234" --input "Perftest-$i-RTP_input"
done
