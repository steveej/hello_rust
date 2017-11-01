export DATABASE_URL = "postgres://postgres:myexample@localhost:5432/invsearch-dev"
export RUST_LOG = "invsearch=info"

diesel-do:
    diesel setup
    diesel migration run

diesel-redo:
    diesel migration redo

build:
    cargo build

test:
    cargo test

run-deps:
    cd ci
    docker-compose up

run:
    cargo run