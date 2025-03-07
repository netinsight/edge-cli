#!/usr/bin/env bash

# This script sets up a performance test where one device generates streams and
# sends them via RTP to the device under test. The device under test then sends
# that stream via RTP to the output device which does TR101290 analysis.
#
# This test does not involve a video node in the traditional sense because the
# stream isn't routed via a core node.

bitrate="$((20 * 1000000))"
test_appliance=dut
protocol=srt
fanout=1

while [[ $# -gt 0 ]]; do
    case $1 in
        -v | --verbose)
            shift
            set -x
            ;;
        --bitrate)
            bitrate=$(numfmt --from=auto "${2}")
            shift 2
            ;;
        --test-appliance)
            test_appliance="$2"
            shift 2
            ;;
        --protocol)
            protocol="$2"
            shift 2
            ;;
        --fanout)
            fanout="$2"
            shift 2
            ;;
        -*)
            echo "Unknown option $1"
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

num_outputs="${1?missing argument: Number of outputs}"

set -euo pipefail

generator_appliance=$(edge appliance list | awk '$3 == "edgeConnect" && $1 ~ /input/ { print $1 }' | sort | head)
output_appliance=$(edge appliance list | awk '$3 == "edgeConnect" && $1 ~ /output/ { print $1 }' | sort | head)

generator_interface=$(edge appliance show "$generator_appliance" | awk '/Name:/ { name=$3; found=0 } /Networks:.*streaming/ { found=1 } /Address:/ && found { print(name) }')
read -r output_interface output_ip <<< "$(edge appliance show "$output_appliance" | awk '/Name:/ { name=$3; found=0 } /Networks:.*streaming/ { found=1 } /Address:/ && found { print(name, $3) }')"
read -r test_interface test_ip <<< "$(edge appliance show "$test_appliance" | awk '/Name:/ { name=$3; found=0 } /Networks:.*streaming/ { found=1 } /Address:/ && found { print(name, $3) }')"

cat >&2 <<EOF
Generator appliance:    ${generator_appliance}
Output appliance:       ${output_appliance}
Test appliance:         ${test_appliance}
EOF

inputs="$(edge input list | awk '/^ID/ { next } { print $2 }')"
outputs="$(edge output list | awk '/^ID/ { next } { print $2 }')"

echo >&2 "Setting up generators"

if ! grep -qw "Perftest-generator-$generator_appliance" <<<"$inputs"; then
    edge input create "Perftest-generator-$generator_appliance" \
        --appliance "$generator_appliance" \
        --mode generator \
        --bitrate "$bitrate" \
        --thumbnail none
fi
if ! grep -qw "Perftest-generator-$generator_appliance" <<<"$outputs"; then
    edge output create "Perftest-generator-$generator_appliance" \
        --input "Perftest-generator-$generator_appliance" \
        --appliance "$generator_appliance" \
        --interface lo \
        --source 127.0.0.1 \
        --mode rtp \
        --dest 224.0.0.44:4444 
fi

for i in $(seq "$num_outputs"); do
    gen_in="Perftest-$i-s1-RTP_gen_input"
    if ! grep -qw "$gen_in" <<<"$inputs"; then
        edge input create "$gen_in" --appliance "$generator_appliance" --interface lo --mode rtp --port 4444 --multicast 224.0.0.44 --thumbnail none
    fi

    case $protocol in
        rtp)
            gen_out="Perftest-$i-s2-RTP_gen_output"
            if ! grep -qw "$gen_out" <<<"$outputs"; then
                edge output create "$gen_out" --appliance "$generator_appliance" --interface "$generator_interface" --mode rtp --dest "$test_ip:$((4000 + i*6))" --input "$gen_in"
            fi

            input="Perftest-$i-s3-RTP_input"
            if ! grep -qw "$input" <<<"$inputs"; then
                edge input create "$input" --appliance "$test_appliance" --interface "$test_interface" --mode rtp --port "$((4000 + i*6))" --thumbnail edge
            fi

            for n in $(seq "$fanout"); do
                output="Perftest-$i-$n-s4-RTP_output"
                port=$((4000 + (i*fanout+(n-1))*6))
                if ! grep -qw "$output" <<<"$outputs"; then
                    edge output create "$output" --appliance "$test_appliance" --interface "$test_interface" --mode rtp --fec 2D --fec-rows 5 --fec-cols 5 --dest "$output_ip:$port" --input "$input"
                fi

                tr101290_input="Perftest-$i-$n-s5-RTP_input_tr101290"
                if ! grep -qw "$tr101290_input" <<<"$inputs"; then
                    edge input create "$tr101290_input" --appliance "$output_appliance" --interface "$output_interface" --mode rtp --fec --port "$port" --thumbnail none
                fi
            done

            ;;
        srt)
            gen_out="Perftest-$i-s2-SRT_gen_output"
            if ! grep -qw "$gen_out" <<<"$outputs"; then
                edge output create "$gen_out" --appliance "$generator_appliance" --interface "$generator_interface" --mode srt --caller --dest "$test_ip:$((4000 + i*6))" --input "$gen_in"
            fi

            input="Perftest-$i-s3-SRT_input"
            if ! grep -qw "$input" <<<"$inputs"; then
                edge input create "$input" --appliance "$test_appliance" --interface "$test_interface" --mode srt --listener --port "$((4000 + i*6))" --thumbnail edge
            fi

            for n in $(seq "$fanout"); do
                output="Perftest-$i-$n-s4-SRT_output"
                port=$((4000 + (i*fanout+(n-1))*6))
                if ! grep -qw "$output" <<<"$outputs"; then
                    edge output create "$output" --appliance "$test_appliance" --interface "$test_interface" --mode srt --caller --dest "$output_ip:$port" --input "$input"
                fi

                tr101290_input="Perftest-$i-$n-s5-SRT_input_tr101290"
                if ! grep -qw "$tr101290_input" <<<"$inputs"; then
                    edge input create "$tr101290_input" --appliance "$output_appliance" --interface "$output_interface" --mode srt --listener --port "$port" --thumbnail none
                fi
            done
            ;;
        rist)
            gen_out="Perftest-$i-s2-RIST_gen_output"
            if ! grep -qw "$gen_out" <<<"$outputs"; then
                edge output create "$gen_out" --appliance "$generator_appliance" --interface "$generator_interface" --mode rist --dest "$test_ip:$((4000 + i*6))" --input "$gen_in"
            fi

            input="Perftest-$i-s3-RIST_input"
            if ! grep -qw "$input" <<<"$inputs"; then
                edge input create "$input" --appliance "$test_appliance" --interface "$test_interface" --mode rist --port "$((4000 + i*6))" --thumbnail edge
            fi

            for n in $(seq "$fanout"); do
                output="Perftest-$i-$n-s4-RIST_output"
                port=$((4000 + (i*fanout+(n-1))*6))
                if ! grep -qw "$output" <<<"$outputs"; then
                    edge output create "$output" --appliance "$test_appliance" --interface "$test_interface" --mode rist --dest "$output_ip:$port" --input "$input"
                fi

                tr101290_input="Perftest-$i-$n-s5-RIST_input_tr101290"
                if ! grep -qw "$tr101290_input" <<<"$inputs"; then
                    edge input create "$tr101290_input" --appliance "$output_appliance" --interface "$output_interface" --mode rist --port "$port" --thumbnail none
                fi
            done
            ;;
        *)
            echo >&2 "Unknown protocol $protocol"
            exit 1
            ;;
    esac
done
