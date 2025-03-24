use crate::CommandError;

/// Show the top talkers in the server (our great orators)
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn yappers(
    ctx: crate::Context<'_>,
    #[description = "Number of top talkers to show (default: 5)"] limit: Option<u32>,
) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    let limit = limit.unwrap_or(5).min(20); // Cap at 20
    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    let talkers = db.get_top_talkers(&server_id, limit).await?;
    
    if talkers.is_empty() {
        ctx.say("No comrades have spoken in this server yet!").await?;
        return Ok(());
    }
    
    let mut response = "**The Party's Most Vocal Comrades**\n\n".to_string();
    
    for (i, (_user_id, username, count)) in talkers.iter().enumerate() {
        let rank = i + 1;
        let medal = match rank {
            1 => "🥇",
            2 => "🥈",
            3 => "🥉",
            _ => "🏅",
        };
        
        response.push_str(&format!("{} **{}**: {} messages\n", medal, username, count));
    }
    
    response.push_str("\nThese comrades are leading our revolutionary discourse! Glory to the vocal patriots!");
    
    ctx.say(response).await?;
    
    Ok(())
} 