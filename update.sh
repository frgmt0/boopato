#!/bin/bash

# config
REPO_PATH="${REPO_PATH:-/app/boopato}"
CONTAINER_NAME="boopato"
NETWORK_NAME="boopato-network"
VOLUME_NAME="boopato-data"
LOG_FILE="/var/log/boopato-update.log"

# log msgs
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# create log file if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"
touch "$LOG_FILE"

# ensure correct directory
cd "$REPO_PATH" || {
  log "ERROR: Could not change to repository directory $REPO_PATH"
  exit 1
}

# check if container is running
is_container_running() {
  docker ps --format "{{.Names}}" | grep -q "^$1$"
}

# pull latest changes from the repository
log "Pulling latest changes from git repository..."
if ! git pull; then
  log "ERROR: Failed to pull latest changes"
  exit 1
fi

# check if there are any changes
if [[ $(git diff HEAD@{1} --name-only | wc -l) -eq 0 ]]; then
  log "No changes detected. Exiting."
  exit 0
fi

log "Changes detected. Rebuilding container..."

# build a new image with the updated code
if ! docker build -t boopato:new .; then
  log "ERROR: Failed to build new Docker image"
  exit 1
fi

# check if the original container is running
if is_container_running "$CONTAINER_NAME"; then
  # start the new container with a different name, connecting to the same network and volume
  log "Starting new container..."
  if ! docker run -d --name "${CONTAINER_NAME}_new" \
      --network "$NETWORK_NAME" \
      -v "$VOLUME_NAME:/app/data" \
      --env-file .env \
      boopato:new; then
    log "ERROR: Failed to start new container"
    exit 1
  fi
  
  # wait for the new container to initialize
  log "Waiting for new container to initialize (30 seconds)..."
  sleep 30
  
  # check if the new container is running
  if is_container_running "${CONTAINER_NAME}_new"; then
    log "New container is running. Stopping and removing the old container..."
    # stop and remove the old container
    if ! docker stop "$CONTAINER_NAME"; then
      log "ERROR: Failed to stop the old container"
      exit 1
    fi
    
    if ! docker rm "$CONTAINER_NAME"; then
      log "ERROR: Failed to remove the old container"
      exit 1
    fi
    
    # rename the new container to take over the original name
    log "Renaming new container to $CONTAINER_NAME..."
    if ! docker rename "${CONTAINER_NAME}_new" "$CONTAINER_NAME"; then
      log "ERROR: Failed to rename the new container"
      exit 1
    fi
    
    log "Update completed successfully. The bot is now running with the latest changes."
  else
    log "ERROR: New container failed to start properly. Keeping the old container running."
    docker rm "${CONTAINER_NAME}_new" || true
    exit 1
  fi
else
  log "Original container is not running. Starting a fresh container..."
  if ! docker run -d --name "$CONTAINER_NAME" \
      --network "$NETWORK_NAME" \
      -v "$VOLUME_NAME:/app/data" \
      --env-file .env \
      boopato:new; then
    log "ERROR: Failed to start container"
    exit 1
  fi
  
  # tag the new image as the current version
  docker tag boopato:new boopato:latest
  
  log "Fresh container started successfully."
fi

# clean up old images
log "Cleaning up old images..."
docker image prune -f

log "Update process completed."