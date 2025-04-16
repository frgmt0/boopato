use crate::CommandError;
use poise::serenity_prelude as serenity;
use rand::seq::SliceRandom;

/// Implement the glorious redistribution of wealth!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn redistribute(
    ctx: crate::Context<'_>,
    #[description = "Optional percentage to take (1-30%, default: 10%)"] 
    percentage: Option<f64>,
) -> Result<(), CommandError> {
    // Get user info
    let _user_id = ctx.author().id.to_string();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server, comrade!".into()),
    };
    
    // Check if user has admin privileges to run this command
    if let Some(_guild_id) = ctx.guild_id() {
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
        ctx.send(|m| {
            m.embed(|e| {
                e.title("☭ Ministry of Economic Affairs ☭")
                 .description("Redistribution Decree Denied")
                 .color(serenity::Color::from_rgb(139, 0, 0)) // Dark red
                 .field(
                    "Insufficient Resources", 
                    "The wealth inequality is insufficient to justify central intervention at this time.", 
                    false
                 )
                 .field(
                    "Suggested Action",
                    "Allow the citizens to continue their collective labor and accumulate more resources before attempting redistribution.",
                    false
                 )
                 .footer(|f| f.text("Economic stability must be maintained for the good of the State."))
            })
        }).await?;
        return Ok(());
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
    
    // Calculate percentages
    let percent_taxed = (tax_details.len() as f64 / users.len() as f64 * 100.0).round() as i32;
    let percent_receiving = (distribution_details.len() as f64 / users.len() as f64 * 100.0).round() as i32;
    
    // Create visual bar for redistribution flow
    let create_flow_bar = || -> String {
        "  Bourgeoisie ───(☭)→ Proletariat  ".to_string()
    };
    
    // Get the top 5 contributors for display
    let top_contributors = tax_details.iter()
        .take(5)
        .map(|(_, name, amount)| {
            format!(
                "`{:<20}` `{:>7.2}` boops", 
                if name.len() > 18 { &name[0..18] } else { name },
                amount
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    
    // Soviet propaganda quotes
    let propaganda_quotes = [
        "From each according to their ability, to each according to their needs!",
        "The wealth of the few has been justly returned to the many!",
        "The wheel of revolution turns, and the people prosper!",
        "Let the capitalists tremble at our redistribution!",
        "The people's commune grows stronger with equality!",
        "The chains of economic oppression have been broken!",
        "When the people share equally, all of society advances!",
        "True freedom comes through economic equality!",
        "The wealth of the nation belongs to all its citizens!",
        "The foundation of collectivism is equal distribution of resources!"
    ];
    
    // Choose a random quote
    let quote = propaganda_quotes.choose(&mut rand::thread_rng()).unwrap_or(&propaganda_quotes[0]);
    
    // Get server name
    let server_name = match ctx.guild_id() {
        Some(guild_id) => guild_id.to_partial_guild(&ctx.serenity_context()).await.map(|g| g.name).unwrap_or_else(|_| "Our Collective".to_string()),
        None => "Our Collective".to_string(),
    };
    
    // Send embed response
    ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ THE GREAT REDISTRIBUTION ☭")
             .description(format!("Economic Reformation of {}", server_name))
             .color(serenity::Color::RED)
             .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a1/Hammer_and_sickle_transparent.svg/240px-Hammer_and_sickle_transparent.svg.png")
             .field(
                "Decree", 
                format!(
                    "By order of Party Official **{}**, a **{}%** redistribution tax has been imposed on the wealthy elite!",
                    ctx.author().name,
                    (tax_rate * 100.0) as u32
                ), 
                false
             )
             .field(
                "Redistribution Summary", 
                format!(
                    "**{:.2}** boops have been seized from the top **{}%** of citizens\n**{:.2}** boops distributed to each of the bottom **{}%**",
                    total_tax, percent_taxed, distribution_per_user, percent_receiving
                ),
                false
             )
             .field(
                "Redistribution Flow", 
                create_flow_bar(),
                false
             );
            
            // Add contributor details if available
            if !tax_details.is_empty() {
                let contributor_title = if tax_details.len() > 5 {
                    format!("Top Contributors (of {} total)", tax_details.len())
                } else {
                    "Contributors to the Cause".to_string()
                };
                
                e.field(
                    contributor_title,
                    if !top_contributors.is_empty() { top_contributors } else { "None".to_string() },
                    false
                );
            }
            
            // Add beneficiary information
            e.field(
                "Beneficiaries",
                format!(
                    "**{}** citizens received economic support\nTotal economic benefits: **{:.2}** boops",
                    distribution_details.len(),
                    total_tax
                ),
                false
            )
            .footer(|f| f.text(quote))
        })
    }).await?;
    
    Ok(())
}

/// Schedule a weekly redistribution of wealth (Admin only)
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn schedule_redistribution(
    ctx: crate::Context<'_>,
    #[description = "Optional percentage to take (1-30%, default: 10%)"] 
    _percentage: Option<f64>,
) -> Result<(), CommandError> {
    ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ Five-Year Plan Notification ☭")
             .description("Automated Wealth Redistribution System")
             .color(serenity::Color::RED)
             .field(
                "Feature Status", 
                "This feature is part of our next Five-Year Plan and is currently in development by our central planning committee.", 
                false
             )
             .field(
                "Current Directive",
                "For now, use the `/redistribute` command to manually implement the will of the Party.",
                false
             )
             .footer(|f| f.text("The Ministry of Economic Affairs appreciates your patience."))
        })
    }).await?;
    Ok(())
} 