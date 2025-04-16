use crate::{Context, CommandError};

/// Show information about the Boopato bot
#[poise::command(prefix_command, slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), CommandError> {
    // Version information
    const VERSION: &str = "1.0 Beta";
    const AUTHOR: &str = "coldie";
    
    // Reply with the embed using builder pattern
    ctx.send(|m| {
        m.embed(|e| {
            e.title("About Boopato")
                .description("The glorious redistributor of boops!")
                .field("Version", VERSION, true)
                .field("Author", AUTHOR, true)
                .field("Made with", "Rust", true)
                .field("Status", "Beta", true)
                .field("Framework", "Poise + Serenity", true)
                .field("Database", "SQLite", true)
                .color(0xE74C3C) // Red color for Soviet theme
                .footer(|f| f.text("☭ From each according to their ability, to each according to their boops ☭"))
        })
    }).await?;
    
    Ok(())
}