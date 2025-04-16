#!/bin/bash

# Config
REPO_PATH="${REPO_PATH:-/app/boopato}"
REPO_URL="https://github.com/frgmt0/boopato.git"
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
  
  # Check if git repo exists, if not clone it
  if [ ! -d "$REPO_PATH/.git" ]; then
    log "Git repository not found. Cloning from $REPO_URL..."
    git clone "$REPO_URL" "$REPO_PATH"
    cd "$REPO_PATH"
  else
    # We're already in the repo directory
    # Make sure we have the right remote
    git remote set-url origin "$REPO_URL" 2>/dev/null || git remote add origin "$REPO_URL"
    
    # fetch latest changes but don't merge them yet
    git fetch origin
  fi
  
  # check if there are any changes
  LOCAL=$(git rev-parse HEAD 2>/dev/null || echo "none")
  REMOTE=$(git rev-parse origin/master 2>/dev/null || echo "none")
  
  if [ "$LOCAL" != "$REMOTE" ]; then
    log "Updates available. Pulling changes..."
    git pull origin master
    log "Running update script..."
    "${REPO_PATH}/update.sh"
  else
    log "No updates available."
  fi
  
  log "Sleeping for $POLL_INTERVAL seconds..."
  sleep "$POLL_INTERVAL"
done