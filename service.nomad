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

      resources {
        cpu    = 100
        memory = 256
      }

      vault {
        policies = ["service-commerce"]
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/.env"
        env         = true
        change_mode = "restart"
        data        = <<EOF
{{ with nomadVar "nomad/jobs/commerce" }}
RUST_LOG='{{ .LOG_LEVEL }}'
{{ end }}

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
BUCKET_ENDPOINT='{{ .BUCKET_ENDPOINT }}'
IMAGE_MAX_SIZE='{{ .IMAGE_MAX_SIZE }}'
ALLOWED_MIN_PLATFORM_FEE_PERCENT='{{ .ALLOWED_MIN_PLATFORM_FEE_PERCENT }}'
ALLOWED_MIN_MINIMUM_PLATFORM_FEE_CENT='{{ .ALLOWED_MIN_MINIMUM_PLATFORM_FEE_CENT }}'
{{ end }}

{{ with secret "kv2/data/services/commerce" }}
BUCKET_ACCESS_KEY_ID='{{ .Data.data.BUCKET_ACCESS_KEY_ID }}'
BUCKET_SECRET_ACCESS_KEY='{{ .Data.data.BUCKET_SECRET_ACCESS_KEY }}'
{{ end }}
EOF
      }

      config {
        image      = "__IMAGE__"
        force_pull = true
      }
    }
  }
}
