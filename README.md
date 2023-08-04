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
docker run --rm \
  --env COCKROACH_DATABASE=commerce \
  --env COCKROACH_USER=commerce_user \
  --env COCKROACH_PASSWORD=commerce \
  --name=cockroachdb \
  --hostname=cockroachdb \
  -p 26257:26257 \
  -p 8080:8080 \
  -v "roach-single:/cockroach/cockroach-data" \
    cockroachdb/cockroach:v23.1.7 start-single-node \
      --http-addr=localhost:8080 \
      --accept-sql-without-tls
```


Ensure environment variables are set.

```sh
export HOST="[::1]:10000"

export DB_HOST='127.0.0.1'
export DB_PORT='26257'
export DB_USER='commerce_user'
export DB_PASSWORD='commerce'
export DB_DBNAME='commerce'

export JWKS_URL='https://auth-dev.peoplesmarkets.com/oauth/v2/keys'
```

Then run:

```sh
cargo run
```
