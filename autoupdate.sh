#!/bin/bash

# Config
REPO_PATH="/app/boopato"
LOG_FILE="/var/log/boopato-autoupdate.log"
POLL_INTERVAL=300  # 5 minutes in seconds

# log msgs
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# create log file if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"
touch "$LOG_FILE"

log "Starting auto-update service for Boopato"

# ensure correct directory
cd "$REPO_PATH" || {
  log "ERROR: Could not change to repository directory $REPO_PATH"
  exit 1
}

# make update.sh executable
chmod +x "${REPO_PATH}/update.sh"

while true; do
  log "Checking for updates..."
  
  # fetch latest changes but don't merge them yet
  git fetch origin
  
  # check if there are any changes
  LOCAL=$(git rev-parse HEAD)
  REMOTE=$(git rev-parse @{u})
  
  if [ "$LOCAL" != "$REMOTE" ]; then
    log "Updates available. Running update script..."
    "${REPO_PATH}/update.sh"
  else
    log "No updates available."
  fi
  
  log "Sleeping for $POLL_INTERVAL seconds..."
  sleep "$POLL_INTERVAL"
done