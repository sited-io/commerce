job "commerce" {
  datacenters = ["dc1"]
  type        = "service"

  group "commerce-api" {
    count = 2

    network {
      mode = "bridge"
    }

    service {
      name = "commerce-api"

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "zitadel"
              local_bind_port  = 8080
            }
            upstreams {
              destination_name = "cockroach-sql"
              local_bind_port  = 5432
            }
          }
        }
      }
    }

    task "commerce-api" {
      driver = "docker"

      vault {
        policies = ["service-commerce"]
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/.env"
        env         = true
        change_mode = "restart"
        data        = <<EOF
RUST_LOG='DEBUG'

HOST='[::1]:10000'

DB_HOST='{{ env "NOMAD_UPSTREAM_IP_cockroach-sql" }}'
DB_PORT='{{ env "NOMAD_UPSTREAM_PORT_cockroach-sql" }}'
DB_DBNAME='commerce'
DB_USER='commerce_user'
{{ with secret "database/static-creds/commerce_user" }}
DB_PASSWORD='{{ .Data.password }}'
{{ end }}

JWKS_URL='http://{{ env "NOMAD_UPSTREAM_ADDR_zitadel" }}/oauth/v2/keys'

{{ with nomadVar "nomad/jobs/commerce" }}
IMAGE='{{ .IMAGE }}'
{{ end }}
EOF
      }

      config {
        image = "${IMAGE}"
      }
    }
  }
}
