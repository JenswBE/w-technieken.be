#!/bin/bash
# Bash strict mode: http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail

# Get real user ID
if [ -z "${SUDO_UID:-}" ]; then
    echo "Must be run with sudo ..."
    exit 1
fi

# Fetch OpenAPI spec from Directus
wget -O openapi.json 'http://localhost:8055/server/specs/oas?access_token=MfoD5hw-BJbRbv7qUo4JU0zzxikOckJD'

# Clean directory if exists
rm -rf openapi/* || true

# Generate models
podman run --pull always --user ${SUDO_UID:?} --rm -v "$(pwd):/local:z" \
docker.io/openapitools/openapi-generator-cli generate \
--input-spec /local/openapi.json \
--generator-name rust \
--output /local/openapi

# Delete unneeded files
rm -rf openapi/docs openapi/.travis.yml openapi/.git_push.sh  openapi/README.md
