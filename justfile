set shell := ["bash", "-cu"]

# Run all services combined in one terminal/log
run-services: kill-services
    trap "echo Killing services...; kill 0" SIGINT
    mkdir -p logs
    echo "" > logs/services.log
    stdbuf -oL cargo run --package auth_service         | tee -a logs/services.log &
    stdbuf -oL cargo run --package data_storage_service | tee -a logs/services.log &
    stdbuf -oL cargo run --package device_service       | tee -a logs/services.log &
    stdbuf -oL cargo run --package scripting_service    | tee -a logs/services.log &
    wait

# Run all APIs combined in one terminal/log
run-apis: kill-apis
    trap "echo Killing APIs...; kill 0" SIGINT
    mkdir -p logs
    echo "" > logs/apis.log
    stdbuf -oL cargo run --package web_api         | tee -a logs/apis.log &
    stdbuf -oL cargo run --package scripting_api      | tee -a logs/apis.log &
    wait

# Run all (services + APIs) combined separately but together
start-all:
    just run-services &  # runs services in background combined output
    just run-apis &      # runs apis in background combined output
    wait

# Run all services and APIs except the specified ones
start-all-except *services: kill-services kill-apis
    #!/bin/bash
    trap "echo Killing services...; kill 0" SIGINT
    mkdir -p logs
    echo "" > logs/services.log
    echo "" > logs/apis.log
    echo "Excluding services: {{ services }}"
    excluded_services="{{ services }}"
    if [[ ! "$excluded_services" =~ "auth_service" ]]; then
        stdbuf -oL cargo run --package auth_service | tee -a logs/services.log &
    else
        echo "Skipping auth_service"
    fi
    if [[ ! "$excluded_services" =~ "data_storage_service" ]]; then
        stdbuf -oL cargo run --package data_storage_service | tee -a logs/services.log &
    else
        echo "Skipping data_storage_service"
    fi
    if [[ ! "$excluded_services" =~ "device_service" ]]; then
        stdbuf -oL cargo run --package device_service | tee -a logs/services.log &
    else
        echo "Skipping device_service"
    fi
    if [[ ! "$excluded_services" =~ "web_api" ]]; then
        stdbuf -oL cargo run --package web_api | tee -a logs/apis.log &
    else
        echo "Skipping web_api"
    fi
    if [[ ! "$excluded_services" =~ "scripting_api" ]]; then
        stdbuf -oL cargo run --package scripting_api | tee -a logs/apis.log &
    else
        echo "Skipping scripting_api"
    fi
    if [[ ! "$excluded_services" =~ "scripting_service" ]]; then
        stdbuf -oL cargo run --package scripting_service | tee -a logs/services.log &
    else
        echo "Skipping scripting_service"
    fi
    wait

kill-services:
    killall auth_service || true
    killall data_storage_service || true
    killall device_service || true
    killall scripting_service || true

kill-apis:
    killall web_api || true
    killall scripting_api || true

stop-all: kill-services kill-apis

lint:
    cargo clippy --all-targets --all-features --workspace  -- -D warnings

test:
    cargo test --release --workspace --all-features -- --test-threads=1

fmt:
    cargo fmt --all -- --color always

device:
    cargo run -p examples --example input_output_int_saver &
    cargo run -p examples --example input_alert_trigger

ci: lint test fmt