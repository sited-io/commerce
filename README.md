# Commerce service for peoplesmarkets.com

## Build

```sh
buf mod update service-apis/proto/
buf generate service-apis/proto --template buf.gen.yaml
cargo build
```

## Run locally

### local database

```sh
docker run --rm -d \
  --env COCKROACH_DATABASE=commerce \
  --env COCKROACH_USER=commerce_user \
  --name cockroachdb \
  --hostname cockroachdb \
  --network "host" \
  -p 26257:26257 \
  -p 8080:8080 \
  -v "roach-single:/cockroach/cockroach-data" \
  cockroachdb/cockroach start-single-node --sql-addr=localhost:5432 --http-addr localhost:8080 --insecure
```

### local kong gateway for gRPC -> gRPC-web

```sh
docker run --rm -d \
  --name kong \
  --mount type=bind,source="$(pwd)"/kong.yaml,target=/kong/kong.yaml \
  --network "host" \
  --env "KONG_DATABASE=off" \
  --env "KONG_DECLARATIVE_CONFIG=/kong/kong.yaml" \
  --env "KONG_PROXY_ACCESS_LOG=/dev/stdout" \
  --env "KONG_ADMIN_ACCESS_LOG=/dev/stdout" \
  --env "KONG_PROXY_ERROR_LOG=/dev/stderr" \
  --env "KONG_ADMIN_ERROR_LOG=/dev/stderr" \
  --env "KONG_PROXY_LISTEN=0.0.0.0:8001" \
  --env "KONG_ADMIN_LISTEN=0.0.0.0:8002" \
  --env "KONG_LOG_LEVEL=info" \
  kong
```

Ensure environment variables are set.

```sh
export HOST="[::1]:10000"

export DB_HOST='127.0.0.1'
export DB_PORT='5432'
export DB_USER='commerce_user'
export DB_PASSWORD=''
export DB_DBNAME='commerce'

export JWKS_URL='https://auth.peoplesmarkets.com/oauth/v2/keys'
```

Then run:

```sh
cargo run
```
