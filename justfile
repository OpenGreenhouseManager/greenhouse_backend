run-services:
    tmux new-session -d -s rust_services \; \
        send-keys 'cargo run --package auth_service' C-m \; \
        split-window -v \; send-keys 'cargo run --package data_storage_service' C-m \; \
        split-window -v \; send-keys 'cargo run --package device_service' C-m \; \
        select-layout tiled \; \
        attach-session -t rust_services

run-apis:
    tmux new-session -d -s rust_apis \; \
        send-keys 'cargo run --package web_api' C-m \; \
        split-window -v \; send-keys 'cargo run --package script_api' C-m \; \
        select-layout tiled \; \
        attach-session -t rust_apis

run-all:
    just run-services
    just run-apis

stop-all:
    tmux kill-session -t rust_services || true
    tmux kill-session -t rust_apis || true