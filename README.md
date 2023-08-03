# Commerce service for peoplesmarkets.com

## Build

```sh
buf mod update service-apis/proto/
buf generate service-apis/proto --template buf.gen.yaml
cargo build
```

## Run locally

Ensure environment variables are set.

```sh
export HOST="[::1]:10000"
```

Then run:

```sh
cargo run
```
