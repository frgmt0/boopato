#!/bin/bash

# Configuration
REPO_PATH="${REPO_PATH:-/app/boopato}"
CONTAINER_NAME="boopato"
LOG_FILE="/var/log/boopato-update.log"

# Function to log messages
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Create log file if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"
touch "$LOG_FILE"

# Ensure we're in the repository directory
cd "$REPO_PATH" || {
  log "ERROR: Could not change to repository directory $REPO_PATH"
  exit 1
}

log "Starting update process..."

# Build a new image using docker-compose
log "Building new container..."
if ! docker-compose build boopato; then
  log "ERROR: Failed to build new Docker image"
  exit 1
fi

# Stop and start the container with the new image
log "Stopping and starting container with updated image..."
if ! docker-compose up -d --no-deps boopato; then
  log "ERROR: Failed to restart container"
  exit 1
fi

log "Update process completed successfully!"