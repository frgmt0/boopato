# Boopato

A Discord bot for managing and distributing "boops" among community members.

## Docker Deployment

### Prerequisites

- Docker and Docker Compose
- Git
- A Discord bot token

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/boopato.git
   cd boopato
   ```

2. Create an environment file:
   ```bash
   cp .env.example .env
   ```

3. Edit the `.env` file to add your Discord token and optionally the Groq API key:
   ```
   DISCORD_TOKEN=your_discord_token_here
   GROQ_API_KEY=your_groq_api_key_here
   ```

4. Make the update scripts executable:
   ```bash
   chmod +x update.sh autoupdate.sh
   ```

### Deployment

Deploy using Docker Compose:

```bash
docker-compose up -d
```

This will:
- Build the Docker image
- Create a persistent volume for the database
- Start the bot

### Auto-Updates

To enable automatic updates that poll for changes in the git repository:

1. Create a systemd service file (on Linux):
   ```bash
   sudo nano /etc/systemd/system/boopato-autoupdate.service
   ```

2. Add the following content:
   ```
   [Unit]
   Description=Boopato Auto-Update Service
   After=docker.service
   Requires=docker.service

   [Service]
   Type=simple
   User=root
   ExecStart=/app/boopato/autoupdate.sh
   Restart=always
   RestartSec=10

   [Install]
   WantedBy=multi-user.target
   ```

3. Enable and start the service:
   ```bash
   sudo systemctl enable boopato-autoupdate.service
   sudo systemctl start boopato-autoupdate.service
   ```

### Manual Updates

To manually update the bot:

```bash
cd /app/boopato
./update.sh
```

## Volume Management

The bot's database is stored in a Docker volume named `boopato-data`. This ensures that your data persists even when containers are updated or rebuilt.

To backup the database:

```bash
docker run --rm -v boopato-data:/data -v $(pwd):/backup alpine sh -c "cp /data/boopato.db /backup/boopato-backup.db"
```

To restore from a backup:

```bash
docker run --rm -v boopato-data:/data -v $(pwd):/backup alpine sh -c "cp /backup/boopato-backup.db /data/boopato.db"
```

## Development

For local development:

```bash
cargo run
```

See CLAUDE.md for more development guidelines.