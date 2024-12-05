job "commerce" {
  datacenters = ["dc1"]
  type        = "service"

  group "commerce-api" {
    count = 1

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
              destination_name = "postgres-sql"
              local_bind_port  = 5432
            }
          }
        }
      }
    }

    task "commerce-api" {
      driver = "docker"

      resources {
        cpu        = 100
        memory     = 256
        memory_max = 256
      }

      vault {
        policies = ["service-commerce"]
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/database_root_cert.crt"
        env         = false 
        change_mode = "restart"
        data        = <<EOF
{{- with secret "kv2/data/services" -}}
{{ .Data.data.DATABASE_ROOT_CERT }}
{{- end -}}
EOF
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/.env"
        env         = true
        change_mode = "restart"
        data        = <<EOF
{{ with nomadVar "nomad/jobs/commerce" }}
RUST_LOG='{{ .RUST_LOG }}'
{{ end }}

HOST='0.0.0.0:{{ env "NOMAD_PORT_grpc" }}'

{{ with nomadVar "nomad/jobs/commerce"}}
DB_HOST='{{ .DB_HOST }}'
DB_PORT='{{ .DB_PORT }}'
DB_DBNAME='{{ .DB_DBNAME }}'
DB_USER='{{ .DB_USER }}'
{{ end }}
DB_ROOT_CERT='{{ env "NOMAD_SECRETS_DIR" }}/database_root_cert.crt'
{{ with secret "kv2/data/services/commerce" }}
DB_PASSWORD='{{ .Data.data.DB_PASSWORD }}'
{{ end }}

{{ with nomadVar "nomad/jobs/" }}
JWKS_HOST='{{ .JWKS_HOST }}'
JWKS_URL='{{ .JWKS_URL }}'
{{ end }}

{{ with nomadVar "nomad/jobs/commerce" }}
BUCKET_NAME='{{ .BUCKET_NAME }}'
BUCKET_URL='{{ .BUCKET_URL }}'
BUCKET_ENDPOINT='{{ .BUCKET_ENDPOINT }}'
IMAGE_MAX_SIZE='{{ .IMAGE_MAX_SIZE }}'
{{ end }}
{{ with secret "kv2/data/services/commerce" }}
BUCKET_ACCESS_KEY_ID='{{ .Data.data.BUCKET_ACCESS_KEY_ID }}'
BUCKET_SECRET_ACCESS_KEY='{{ .Data.data.BUCKET_SECRET_ACCESS_KEY }}'
{{ end }}

{{ with nomadVar "nomad/jobs" }}
NATS_HOST='{{ .NATS_HOST }}'
NATS_USER='{{ .NATS_USER }}'
{{ end }}
{{ with secret "kv2/data/services" }}
NATS_PASSWORD='{{ .Data.data.NATS_PASSWORD }}'
{{ end }}

{{ with nomadVar "nomad/jobs/commerce" }}
ALLOWED_MIN_PLATFORM_FEE_PERCENT='{{ .ALLOWED_MIN_PLATFORM_FEE_PERCENT }}'
ALLOWED_MIN_MINIMUM_PLATFORM_FEE_CENT='{{ .ALLOWED_MIN_MINIMUM_PLATFORM_FEE_CENT }}'
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
