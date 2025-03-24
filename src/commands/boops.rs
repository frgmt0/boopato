use crate::CommandError;

/// Display your boops and the communal boops
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn boops(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };
    
    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Get personal and communal boops
    let personal_boops = db.get_user_boops(&user_id).await?;
    let communal_boops = db.get_communal_boops(&server_id).await?;
    
    // Get top contributors
    let top_contributors = db.get_top_contributors(&server_id, 5).await?;
    
    // Get distribution round status
    let (current_round, claimed_users, total_users) = db.get_distribution_status(&server_id).await?;
    let has_claimed = db.has_claimed_current_round(&user_id, &server_id).await?;
    
    // Create response
    let mut response = format!(
        "**☭ Communal Boop Report ☭**\n\n"
    );
    
    response.push_str(&format!("**Total Communal Boops**: {:.2} {}\n", communal_boops, if communal_boops > 1000.0 { "🌟" } else { "" }));
    response.push_str(&format!("**Your Personal Boops**: {:.2} {}\n\n", personal_boops, if personal_boops > 100.0 { "⭐" } else { "" }));
    
    // Display top contributors
    if !top_contributors.is_empty() {
        response.push_str("**Top Contributors to Society**:\n");
        
        for (i, (_, username, boops)) in top_contributors.iter().enumerate() {
            let rank = i + 1;
            let medal = match rank {
                1 => "🥇",
                2 => "🥈",
                3 => "🥉",
                _ => "🏅",
            };
            
            response.push_str(&format!("{} **{}**: {:.2} boops\n", medal, username, boops));
        }
    }
    
    // Add Marx quote
    response.push_str("\n\"From each according to their ability, to each according to their needs.\"\n");
    
    // Distribution information
    if communal_boops > 0.0 {
        response.push_str(&format!("\n**Distribution Round**: #{}\n", current_round));
        response.push_str(&format!("**Claims**: {}/{} users have claimed their share\n", claimed_users, total_users));
        
        if has_claimed {
            response.push_str("You have already claimed your share for this round.\n");
        } else {
            let potential_claim = ((communal_boops / total_users as f64) * 100.0).round() / 100.0;
            if potential_claim > 0.0 {
                response.push_str(&format!("You can claim approximately {:.2} boops with `/claim`!\n", potential_claim));
            }
        }
    }
    
    // Check how many users are in the database
    let user_count = db.get_server_user_count(&server_id).await?;
    
    // If the server has less than 3 users registered and the user is an admin, suggest syncing
    if user_count < 3 {
        // Check if user is admin
        let is_admin = match ctx.guild_id() {
            Some(guild_id) => match ctx.author_member().await {
                Some(member) => member.permissions.map_or(false, |perms| perms.administrator()),
                None => false,
            },
            None => false,
        };
        
        if is_admin {
            response.push_str("\n\n**Note to administrators:** I see only a few users in my database. If this server has more members, please use `/sync_users` to ensure all comrades are properly counted for boops distribution.");
        }
    }
    
    ctx.say(response).await?;
    
    Ok(())
}

/// Claim your share of the communal boops
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn claim(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };
    
    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Check if user has already claimed in this round
    if db.has_claimed_current_round(&user_id, &server_id).await? {
        let (current_round, claimed_users, total_users) = db.get_distribution_status(&server_id).await?;
        ctx.say(format!(
            "Comrade, you have already claimed your share for distribution round #{}!\n\
            {}/{} users have claimed their share in this round.\n\
            Wait until the next communal distribution to claim again.",
            current_round, claimed_users, total_users
        )).await?;
        return Ok(());
    }
    
    // Claim boops
    let claimed_amount = db.claim_boops(&user_id, &server_id).await?;
    
    if claimed_amount <= 0.0 {
        ctx.say("There are no boops to claim right now, comrade. The communal stores are empty or your share is too small.").await?;
        return Ok(());
    }
    
    // Get updated personal boops and distribution status
    let personal_boops = db.get_user_boops(&user_id).await?;
    let (current_round, claimed_users, total_users) = db.get_distribution_status(&server_id).await?;
    
    let response = format!(
        "**☭ Boops Claimed! ☭**\n\n\
        You have claimed {:.2} boops from the communal pool in round #{}!\n\
        **Your Personal Boops**: {:.2} {}\n\n\
        {}/{} users have claimed their share in this round.\n\
        The party salutes your participation in our shared economy.",
        claimed_amount,
        current_round,
        personal_boops,
        if personal_boops > 100.0 { "⭐" } else { "" },
        claimed_users,
        total_users
    );
    
    ctx.say(response).await?;
    
    Ok(())
} 