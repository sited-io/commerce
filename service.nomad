job "commerce" {
  datacenters = ["dc1"]
  type        = "service"

  group "commerce-api" {
    count = 2

    network {
      mode = "bridge"

      port "grpc" {}
    }

    service {
      name = "commerce-api"
      port = "grpc"

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

      check {
        type     = "grpc"
        interval = "20s"
        timeout  = "2s"
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
RUST_LOG='INFO'

HOST='0.0.0.0:{{ env "NOMAD_PORT_grpc" }}'

DB_HOST='{{ env "NOMAD_UPSTREAM_IP_cockroach-sql" }}'
DB_PORT='{{ env "NOMAD_UPSTREAM_PORT_cockroach-sql" }}'
DB_DBNAME='commerce'
DB_USER='commerce_user'
{{ with secret "database/static-creds/commerce_user" }}
DB_PASSWORD='{{ .Data.password }}'
{{ end }}

{{ with nomadVar "nomad/jobs/" }}
JWKS_HOST='{{ .JWKS_HOST }}'
{{ end }}
JWKS_URL='http://{{ env "NOMAD_UPSTREAM_ADDR_zitadel" }}/oauth/v2/keys'

{{ with nomadVar "nomad/jobs/commerce" }}
BUCKET_NAME='{{ .BUCKET_NAME }}'
BUCKET_URL='{{ .BUCKET_URL }}'
IMAGE_MAX_SIZE='{{ .IMAGE_MAX_SIZE }}'
{{ end }}

{{ with secret "kv2/data/services/commerce" }}
BUCKET_ACCESS_KEY_ID='{{ .BUCKET_ACCESS_KEY_ID }}'
BUCKET_SECRET_ACCESS_KEY='{{ .BUCKET_SECRET_ACCESS_KEY }}'
BUCKET_ACCOUTN_ID='{{ .BUCKET_ACCOUTN_ID }}'
{{ end }}

{{ with nomadVar "nomad/jobs/commerce" }}
IMAGE='{{ .IMAGE }}'
{{ end }}
EOF
      }

      config {
        image      = "${IMAGE}"
        force_pull = true
      }
    }
  }
}
