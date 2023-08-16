FROM debian:bookworm-slim

RUN apt update && apt install -y --no-install-recommends ca-certificates adduser
RUN update-ca-certificates

# Copy our build
COPY target/release/commerce /usr/local/bin/commerce

# Create appuser
ENV USER=commerce_user
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

# Use an unprivileged user.
USER commerce:commerce

ENTRYPOINT ["commerce"]
