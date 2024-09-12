#!/bin/bash

# Download test data

# Bash strict mode based on http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail

# Settings
SCRIPT_DIR="$( dirname -- "$BASH_SOURCE"; )";

# Source helpers
. ${SCRIPT_DIR:?}/helpers.sh

# Push files
FOLDER_ID_GENERAL=$(get_folder_id_by_name 'General')
FOLDER_ID_REALISATIES=$(get_folder_id_by_name 'Realisaties')
push_file 'banner.jpg' 'faf63493-8440-4be1-9239-d42c8042861a' "${FOLDER_ID_GENERAL}"
push_file 'airco.jpg' 'c1eb4e2d-dcad-4ccf-9172-d372b5a20b6a' "${FOLDER_ID_REALISATIES}"
push_file 'airco2.jpg' 'b60fb273-9476-4ea7-8f9d-7085ff587e7a' "${FOLDER_ID_REALISATIES}"
push_file 'ventilatie.jpg' '2ba0bbcd-8537-467b-9d45-849d29833476' "${FOLDER_ID_REALISATIES}"
push_file 'warmtepomp.jpg' 'f607515a-b3ad-47f3-9620-751b881adadc' "${FOLDER_ID_REALISATIES}"

# Push collections
push_collection "flow_debounce"
push_collection "general_settings"
push_collection "realisations"
push_collection "realisations_files"

# Push users
push_local_user
