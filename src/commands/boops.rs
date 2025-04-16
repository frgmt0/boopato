use crate::CommandError;
use poise::serenity_prelude as serenity;
use rand::seq::SliceRandom;

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
    
    // Get top contributors - increased to 10
    let top_contributors = db.get_top_contributors(&server_id, 10).await?;
    
    // Get distribution round status
    let (current_round, claimed_users, total_users) = db.get_distribution_status(&server_id).await?;
    let has_claimed = db.has_claimed_current_round(&user_id, &server_id).await?;
    
    // Admin check for low user count warning
    let user_count = db.get_server_user_count(&server_id).await?;
    let is_admin = match ctx.guild_id() {
        Some(_guild_id) => match ctx.author_member().await {
            Some(member) => member.permissions.map_or(false, |perms| perms.administrator()),
            None => false,
        },
        None => false,
    };
    
    // Calculate potential claim amount
    let potential_claim = if communal_boops > 0.0 && !has_claimed {
        ((communal_boops / total_users as f64) * 100.0).round() / 100.0
    } else {
        0.0
    };
    
    // Create progress bars for visual representation
    let create_progress_bar = |value: f64, max: f64| -> String {
        let percentage = (value / max * 100.0).min(100.0);
        let blocks = (percentage / 10.0).round() as usize;
        let filled = "█".repeat(blocks);
        let empty = "░".repeat(10 - blocks);
        format!("{filled}{empty} ({percentage:.1}%)")
    };
    
    let personal_max = 100.0; // Threshold for personal progress
    let communal_max = 1000.0; // Threshold for communal progress
    
    let personal_progress = create_progress_bar(personal_boops, personal_max);
    let communal_progress = create_progress_bar(communal_boops, communal_max);
    
    // Soviet-themed quotes for footer
    let quotes = [
        "From each according to their ability, to each according to their needs.",
        "Workers of the world, unite!",
        "The will of the collective is the highest law.",
        "Labor is the source of all wealth and culture.",
        "The strength of the collective is measured by the prosperity of all.",
        "Glory to labor, glory to the collective!",
        "One for all, all for one - the foundation of our strength.",
        "The path to equality is paved with shared labor.",
        "Serve the people with all your heart and soul.",
        "The collective farm is the path to socialism."
    ];
    let quote = quotes.choose(&mut rand::thread_rng()).unwrap_or(&quotes[0]);
    
    // Create embed response
    ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ State Treasury Report ☭")
                .description("Detailed accounting of all contributions to our glorious collective")
                .color(serenity::Color::RED)
                .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/7/79/Hammer_and_sickle.svg/512px-Hammer_and_sickle.svg.png")
                .field("Personal Contribution", format!("**{:.2}** boops\n{}", personal_boops, personal_progress), false)
                .field("Collective Treasury", format!("**{:.2}** boops\n{}", communal_boops, communal_progress), false);
            
            // Add distribution info
            if communal_boops > 0.0 {
                let distribution_status = format!(
                    "Round: **#{}**\nComrades Claimed: **{}/{}**",
                    current_round, claimed_users, total_users
                );
                
                let claim_status = if has_claimed {
                    "You have fulfilled your duty by claiming your share for this distribution cycle.".to_string()
                } else if potential_claim > 0.0 {
                    format!("You may claim **{:.2}** boops with `/claim`!", potential_claim)
                } else {
                    "No boops available to claim at this time.".to_string()
                };
                
                e.field("Distribution Status", distribution_status, true)
                 .field("Your Claim", claim_status, true);
            }
            
            // Add top contributors section
            if !top_contributors.is_empty() {
                let mut contributors_text = String::new();
                
                for (i, (_, username, boops)) in top_contributors.iter().enumerate() {
                    let rank = i + 1;
                    let rank_symbol = match rank {
                        1 => "☭1".to_string(),
                        2 => "☭2".to_string(),
                        3 => "☭3".to_string(),
                        _ => format!("☭{}", rank),
                    };
                    
                    // Create fancy formatting with alignment
                    contributors_text.push_str(&format!(
                        "`{:<4}` `{:<20}` `{:>8.2}` boops\n",
                        rank_symbol,
                        // Truncate username if too long
                        if username.len() > 18 { &username[0..18] } else { username },
                        boops
                    ));
                }
                
                e.field("Heroes of Socialist Labor", contributors_text, false);
            }
            
            // Add admin notice if needed
            if user_count < 3 && is_admin {
                e.field(
                    "⚠️ Administrative Notice ⚠️",
                    "Comrade Administrator, our records show very few registered citizens. If this collective has more members, please use `/sync_users` to ensure all are properly counted for equitable distribution.",
                    false
                );
            }
            
            // Add footer quote
            e.footer(|f| {
                f.text(format!("「{}」", quote))
            })
        })
    }).await?;
    
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
    
    // Get distribution round status
    let (current_round, claimed_users, total_users) = db.get_distribution_status(&server_id).await?;
    
    // Check if user has already claimed in this round
    if db.has_claimed_current_round(&user_id, &server_id).await? {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("☭ Distribution Claim Denied ☭")
                 .description("Your request to claim resources has been rejected")
                 .color(serenity::Color::from_rgb(139, 0, 0)) // Dark red
                 .field(
                    "Reason for Rejection", 
                    format!("Comrade, according to our records, you have already received your allocation for distribution round #{}.", current_round),
                    false
                 )
                 .field(
                    "Distribution Status",
                    format!("{}/{} comrades have fulfilled their duty to claim in this cycle.", claimed_users, total_users),
                    false
                 )
                 .footer(|f| f.text("Wait for the next distribution cycle to receive additional resources."))
            })
        }).await?;
        return Ok(());
    }
    
    // Claim boops
    let claimed_amount = db.claim_boops(&user_id, &server_id).await?;
    
    // If no boops to claim
    if claimed_amount <= 0.0 {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("☭ Treasury Notice ☭")
                 .description("Your resource claim request cannot be processed at this time")
                 .color(serenity::Color::DARK_GREY)
                 .field(
                    "Distribution Status", 
                    "The collective treasury is unable to fulfill your request.", 
                    false
                 )
                 .field(
                    "Reason",
                    "Insufficient resources in the communal stores or your calculated share is below the minimum threshold.",
                    false
                 )
                 .footer(|f| f.text("Continue your labor to contribute to the treasury for future distributions."))
            })
        }).await?;
        return Ok(());
    }
    
    // Get updated personal boops and distribution status
    let personal_boops = db.get_user_boops(&user_id).await?;
    let (_, updated_claimed, total_users) = db.get_distribution_status(&server_id).await?;
    
    // Calculate percentage of total claimed
    let claim_percentage = (updated_claimed as f64 / total_users as f64 * 100.0).round();
    
    // Progress bar for claims
    let blocks = (claim_percentage / 10.0).round() as usize;
    let filled = "█".repeat(blocks);
    let empty = "░".repeat(10 - blocks);
    let claim_progress = format!("{filled}{empty} ({claim_percentage:.0}%)");
    
    // Create success embed
    ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ Distribution Successful ☭")
             .description(format!("You have received your allocation from Round #{}", current_round))
             .color(serenity::Color::RED)
             .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
             .field(
                "Resources Claimed", 
                format!("**{:.2}** boops transferred to your personal account", claimed_amount),
                false
             )
             .field(
                "Updated Balance",
                format!("**{:.2}** boops in your personal account", personal_boops),
                true
             )
             .field(
                "Rank Status",
                {
                    if personal_boops > 500.0 {
                        "**Hero of Socialist Labor** ★★★"
                    } else if personal_boops > 250.0 {
                        "**Order of Lenin** ★★"
                    } else if personal_boops > 100.0 {
                        "**Order of the Red Star** ★"
                    } else {
                        "**Citizen**"
                    }
                },
                true
             )
             .field(
                "Collective Distribution Progress",
                format!("{}/{} comrades have claimed\n{}", updated_claimed, total_users, claim_progress),
                false
             )
             .footer(|f| 
                f.text("The State commends your participation in our shared economic system.")
             )
        })
    }).await?;
    
    Ok(())
} 