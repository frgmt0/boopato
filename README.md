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
   git clone https://github.com/frgmt0/boopato.git
   cd boopato
   ```

2. Create an environment file:

   ```bash
   cp .env.example .env
   ```

3. Edit the `.env` file to add your Discord token and optionally the Groq API key:

   ```bash
   DISCORD_TOKEN=your_discord_token_here
   GROQ_API_KEY=your_groq_api_key_here
   ```

## Development

For local development:

```bash
cargo run
```

See CLAUDE.md for more development guidelines.