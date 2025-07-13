set shell := ["bash", "-cu"]

# Run all services combined in one terminal/log
run-services: kill-services
    trap "echo Killing services...; kill 0" SIGINT
    mkdir -p logs
    echo "" > logs/services.log
    stdbuf -oL cargo run --package auth_service         | tee -a logs/services.log &
    stdbuf -oL cargo run --package data_storage_service | tee -a logs/services.log &
    stdbuf -oL cargo run --package device_service       | tee -a logs/services.log &
    wait

# Run all APIs combined in one terminal/log
run-apis: kill-apis
    trap "echo Killing APIs...; kill 0" SIGINT
    mkdir -p logs
    echo "" > logs/apis.log
    stdbuf -oL cargo run --package web_api         | tee -a logs/apis.log &
    stdbuf -oL cargo run --package script_api      | tee -a logs/apis.log &
    wait

# Run all (services + APIs) combined separately but together
start-all:
    just run-services &  # runs services in background combined output
    just run-apis &      # runs apis in background combined output
    wait

kill-services:
    killall auth_service || true
    killall data_storage_service || true
    killall device_service || true

kill-apis:
    killall web_api || true
    killall script_api || true

stop-all: kill-services kill-apis
