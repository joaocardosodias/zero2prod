#!/usr/bin/env bash
set -x
set -eo pipefail

# Garante que binários do cargo estejam no PATH (necessário ao rodar com sudo)
export PATH="$PATH:/home/kali/.cargo/bin"

if ! command -v sqlx &> /dev/null; then
    echo "sqlx is not installed. Please install it by running: cargo install sqlx-cli"
    exit 1
fi

DB_PORT="${POSTGRES_PORT:-5432}"
SUPERUSER="${POSTGRES_USER:-postgres}"
SUPERUSER_PWD="${POSTGRES_PASSWORD:-postgres}"
APP_USER="${APP_USER:=app}"
APP_USER_PWD="${APP_USER_PWD:=secret}"
APP_DB_NAME="${APP_DB_NAME:=newsletter}"

CONTAINER_NAME="postgres"


if [ "$(docker ps -aq -f name=^/${CONTAINER_NAME}$)" ]; then
  docker rm -f "${CONTAINER_NAME}"
fi

docker run \
  --env POSTGRES_USER=${SUPERUSER} \
  --env POSTGRES_PASSWORD=${SUPERUSER_PWD} \
  --health-cmd="pg_isready -U ${SUPERUSER}" \
  --health-interval="10s" \
  --health-timeout="5s" \
  --health-retries="5" \
  --env POSTGRES_DB=${APP_DB_NAME} \
  --publish ${DB_PORT}:5432 \
  --name ${CONTAINER_NAME} \
  --detach \
  postgres -N 1000

until [ "$(docker inspect -f '{{.State.Health.Status}}' ${CONTAINER_NAME})" == "healthy" ]; do
    echo "Postgress is still unavailable - sleeping"
    sleep 1
done
echo "PostgreSQL is up running on port ${DB_PORT}"

CREATE_QUERY="CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}';"
docker exec -i "${CONTAINER_NAME}" psql -U ${SUPERUSER} -c "${CREATE_QUERY}"

GRANT_QUERY="ALTER USER ${APP_USER} CREATEDB;"
docker exec -i "${CONTAINER_NAME}" psql -U ${SUPERUSER} -c "${GRANT_QUERY}"

export DATABASE_URL="postgres://${APP_USER}:${APP_USER_PWD}@localhost:${DB_PORT}/${APP_DB_NAME}"

sqlx database create

# Necessário no PostgreSQL 15+: concede permissão no schema public ao usuário da aplicação
SCHEMA_QUERY="GRANT ALL ON SCHEMA public TO ${APP_USER};"
docker exec -i "${CONTAINER_NAME}" psql -U ${SUPERUSER} -d ${APP_DB_NAME} -c "${SCHEMA_QUERY}"

sqlx migrate run --source ./migrations

