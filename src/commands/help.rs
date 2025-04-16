use crate::{Context, CommandError};

/// Display help information about all available commands
#[poise::command(prefix_command, slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), CommandError> {
    ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ Boopato Help ☭")
                .description("The glorious redistributor of boops! Here's a list of all available commands.")
                .field("Economy Commands", "
**/boops** - Display your personal boops and communal treasury
**/claim** - Claim your share of communal boops
**/work** - Earn boops (3-hour cooldown)
**/commit** - Commit crimes for boops (30-min cooldown)", false)
                .field("Job System", "
**/jobs_list** - View available jobs
**/jobs_apply [job]** - Apply for a job (50% success)
**/jobs_quit** - Quit your current job
**/prosper** - Level up your job (33% success)", false)
                .field("Games", "
**/game** - Shows available games
**/tictactoe [@user]** - Play tic-tac-toe
**/connect4 [@user]** - Play Connect 4
**/clicker** - Test your reaction time
**/kremlin_secrets [difficulty]** - Word challenge
**/soviet_hangman** - Word guessing game", false)
                .field("Admin Commands", "
**/distribute** - Distribute all communal boops
**/sync_users** - Sync server members to database
**/reset_cooldowns** - Clear your cooldowns (owner)
**/list_users** - List all users in database (owner)
**/reset_server** - Reset server data (owner)
**/redistribute [percentage]** - Redistribute wealth", false)
                .field("Utilities", "
**/about** - Bot information
**/help** - Show this message", false)
                .color(0xE74C3C) // Red color for Soviet theme
                .footer(|f| f.text("From each according to their ability, to each according to their boops"))
        })
    }).await?;
    
    Ok(())
}