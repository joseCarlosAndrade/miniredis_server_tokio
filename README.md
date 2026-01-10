# miniredis implementation in rust

simple frame protocol for miniredis implemented in rust

run miniredis on port 6000

## setting up

```bash
mini-redis-server --port 6000
```

execute the server

```bash
cargo run
```

## trying some commands

```bash
mini-redis-cli --port 6000 get aa    # get
mini-redis-cli --port 6000 set aa bb # set
```

## protocol

sends messages via bulk in the format:

```text
Array([Bulk(b"get"), Bulk(b"aa")])
Array([Bulk(b"set"), Bulk(b"aa"), Bulk(b"bb")])
```

## executing separate binaries

```bash
cargo run --build echo-server-manual
cargo run --examples hello-redis
```

## websocket

using websocat to debug this

installation:

```bash
    sudo apt install libssl-dev # dependency
    cargo install websocat
```
