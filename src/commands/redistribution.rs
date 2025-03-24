use crate::CommandError;
use rand::Rng;

/// Implement the glorious redistribution of wealth!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn redistribute(
    ctx: crate::Context<'_>,
    #[description = "Optional percentage to take (1-30%, default: 10%)"] 
    percentage: Option<f64>,
) -> Result<(), CommandError> {
    // Get user info
    let user_id = ctx.author().id.to_string();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server, comrade!".into()),
    };
    
    // Check if user has admin privileges to run this command
    if let Some(guild_id) = ctx.guild_id() {
        match ctx.author_member().await {
            Some(member) => {
                let permissions = member.permissions(ctx).unwrap_or_default();
                if !permissions.administrator() {
                    return Err("Only server administrators can initiate wealth redistribution, comrade!".into());
                }
            },
            None => return Err("Failed to get your member information. Try again later.".into()),
        };
    }
    
    // Validate percentage
    let tax_rate = percentage.unwrap_or(10.0).clamp(1.0, 30.0) / 100.0;
    
    // Defer response to give us time to process
    ctx.defer().await?;
    
    let db = &ctx.data().db;
    
    // Get all users in the server
    let users = match db.get_all_users(&server_id).await {
        Ok(users) => users,
        Err(e) => return Err(format!("Error fetching users: {}", e).into()),
    };
    
    if users.is_empty() {
        return Err("No users found in the database for this server!".into());
    }
    
    // Sort users by boops (richest first)
    let mut users = users;
    users.sort_by(|(_, _, a), (_, _, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    
    // Identify the top 20% of users who will be taxed
    let wealthy_threshold = (users.len() as f64 * 0.2).ceil() as usize;
    let wealthy_users: Vec<_> = users.iter().take(wealthy_threshold).collect();
    
    // Calculate total wealth to redistribute
    let mut total_tax = 0.0;
    let mut tax_details = Vec::new();
    
    for (user_id, username, boops) in wealthy_users {
        if *boops <= 0.0 {
            continue; // Skip users with no boops
        }
        
        let user_tax = boops * tax_rate;
        if user_tax > 0.1 { // Only tax if it's at least 0.1 boops
            total_tax += user_tax;
            let new_balance = boops - user_tax;
            
            // Update user's boops
            if let Err(e) = db.update_user_boops(user_id, new_balance).await {
                eprintln!("Failed to update boops for {}: {}", user_id, e);
                continue;
            }
            
            tax_details.push((user_id.clone(), username.clone(), user_tax));
        }
    }
    
    if total_tax < 1.0 {
        return Err("Not enough wealth to redistribute! The wealthy must accumulate more first.".into());
    }
    
    // Identify bottom 50% who will receive the redistribution
    let poor_threshold = (users.len() as f64 * 0.5).ceil() as usize;
    let poor_users: Vec<_> = users.iter().rev().take(poor_threshold).collect();
    
    if poor_users.is_empty() {
        return Err("No users found to receive redistribution!".into());
    }
    
    // Distribute wealth evenly
    let distribution_per_user = total_tax / poor_users.len() as f64;
    let mut distribution_details = Vec::new();
    
    for (user_id, username, current_boops) in poor_users {
        let new_balance = current_boops + distribution_per_user;
        
        // Update user's boops
        if let Err(e) = db.update_user_boops(user_id, new_balance).await {
            eprintln!("Failed to update boops for {}: {}", user_id, e);
            continue;
        }
        
        distribution_details.push((user_id.clone(), username.clone(), distribution_per_user));
    }
    
    // Generate propaganda message
    let mut response = format!(
        "☭ **THE GREAT REDISTRIBUTION HAS COMMENCED!** ☭\n\n\
        Comrade {} has ordered a {}% tax on the wealthy!\n\n\
        **{:.2} boops** have been redistributed from the bourgeoisie to the proletariat!\n\n",
        ctx.author().name,
        (tax_rate * 100.0) as u32,
        total_tax
    );
    
    // Add wealthy contributors section if it's not too long
    if tax_details.len() <= 5 {
        response.push_str("**Top Contributors to the Cause:**\n");
        for (_, username, tax) in tax_details.iter().take(5) {
            response.push_str(&format!("• {} contributed {:.2} boops\n", username, tax));
        }
        response.push_str("\n");
    } else {
        response.push_str(&format!("**{} wealthy citizens** have contributed to the common good!\n\n", 
            tax_details.len()));
    }
    
    // Add recipients section
    response.push_str(&format!("**{} citizens** received {:.2} boops each!\n\n", 
        distribution_details.len(), distribution_per_user));
    
    // Add propaganda quote
    let propaganda_quotes = [
        "From each according to their ability, to each according to their needs!",
        "The wealth of the few has been justly returned to the many!",
        "The wheel of revolution turns, and the people prosper!",
        "Let the capitalists tremble at our redistribution!",
        "The people's commune grows stronger with equality!",
    ];
    let quote = propaganda_quotes[rand::thread_rng().gen_range(0..propaganda_quotes.len())];
    response.push_str(&format!("**{}**", quote));
    
    ctx.say(response).await?;
    
    Ok(())
}

/// Schedule a weekly redistribution of wealth (Admin only)
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn schedule_redistribution(
    ctx: crate::Context<'_>,
    #[description = "Optional percentage to take (1-30%, default: 10%)"] 
    percentage: Option<f64>,
) -> Result<(), CommandError> {
    ctx.say("This feature is planned for a future update, comrade! For now, use the `/redistribute` command manually to spread the wealth.").await?;
    Ok(())
} 