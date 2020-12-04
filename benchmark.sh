#!/bin/bash
declare -a curves=(
    'bls12_377'
    'bls12_381'
    'bn254'
    'bw6_761'
    'cp6_782'
)

declare -a exs=(
    'add'
    'mul'
    'pairing_two'
    'pairing_six'
)

function bm() {
    ./target/release/node-template \
        benchmark \
        -p template \
        -e "${@:0:1}"
}


function main() {
    for p in "${curves[@]}"
    do
        echo "Curve: ${p}"
        echo '=========================µs'
        for q in "${exs[@]}"
        do
            echo "--->${q}"
            echo -n "Wasm:   "
            bm wasm_"${p}"_"${q}" --execution Wasm --wasm-execution Compiled | grep 'Time ~=' -m 1
            echo -n "Native: "
            bm wasm_"${p}"_"${q}" --execution Native | grep 'Time ~=' -m 1
        done
        echo ""
        echo ""
    done
}

main $@
