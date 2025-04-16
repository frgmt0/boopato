#!/bin/bash

# Make scripts executable
chmod +x update.sh autoupdate.sh

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
  echo "Creating .env file..."
  echo "# Discord Bot Token" > .env
  echo "DISCORD_TOKEN=your_discord_token_here" >> .env
  echo "" >> .env
  echo "# Groq API Key (Optional)" >> .env
  echo "GROQ_API_KEY=your_groq_api_key_here" >> .env
  
  echo ".env file created. Please edit it to add your Discord token."
  exit 1
fi

# Start everything up
echo "Starting Boopato and watcher containers..."
docker-compose down
docker-compose up -d

echo "Deployment complete! Check logs with:"
echo "  docker logs boopato"
echo "  docker logs boopato-watcher"