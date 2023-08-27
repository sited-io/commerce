# Commerce service for peoplesmarkets.com

## Prerequesites

Ensure `service-apis` git submodule is initialized. If not yet done run:

```sh
git submodule update --init --recursive
```

If `service-apis` git submodule was already initialized, ensure to pull the newest changes:

```sh
git submodule update --recursive --remote
```

## Build

```sh
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
  -v "roach-single:/cockroach/cockroach-data" \
  cockroachdb/cockroach start-single-node --sql-addr=localhost:5432 --http-addr localhost:8080 --insecure
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
