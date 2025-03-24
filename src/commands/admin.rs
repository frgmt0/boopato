use crate::CommandError;
use poise::serenity_prelude as serenity;
use std::time::Duration;

const OWNER_ID: &str = "1151230208783945818";

/// Check if the user has admin permissions
async fn check_if_admin(ctx: crate::Context<'_>) -> Result<bool, CommandError> {
    let member = match ctx.guild_id() {
        Some(guild_id) => match ctx.author_member().await {
            Some(member) => member,
            None => {
                return Err("Failed to get member information".into());
            }
        },
        None => return Ok(false),
    };

    Ok(member.permissions.map_or(false, |perms| perms.administrator()))
}

/// Clear your cooldowns (Owner only)
/// 
/// This command is only usable by the bot owner for testing purposes
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn reset_cooldowns(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    
    // Check if the user is the owner
    if user_id != OWNER_ID {
        return Err("You are not authorized to use this command, comrade! Only the supreme leader may reset their own cooldowns.".into());
    }
    
    let db = &ctx.data().db;
    
    // Clear work cooldown
    db.clear_user_cooldowns(&user_id).await?;
    
    ctx.say("The Party has graciously reset all your cooldowns, Supreme Leader. You may now continue testing.").await?;
    
    Ok(())
}

/// List all users in the database (Owner only)
///
/// This command is only usable by the bot owner for debugging purposes
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn list_users(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    
    // Check if the user is the owner
    if user_id != OWNER_ID {
        return Err("You are not authorized to use this command, comrade! Only the supreme leader may view the citizen database.".into());
    }
    
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };
    
    let db = &ctx.data().db;
    
    // Get all users in the current server
    let users = db.get_all_users(&server_id).await?;
    
    if users.is_empty() {
        ctx.say("No users found in the database for this server!").await?;
        return Ok(());
    }
    
    let mut response = "**Users in Database**\n\n".to_string();
    
    for (i, (user_id, username, boops)) in users.iter().enumerate() {
        response.push_str(&format!("{}. **{}** (ID: {}) - {:.2} boops\n", 
            i + 1, username, user_id, boops));
        
        // Discord has a 2000 character limit, so break it up if needed
        if response.len() > 1900 {
            ctx.say(response).await?;
            response = "**Continued...**\n\n".to_string();
        }
    }
    
    if !response.is_empty() {
        ctx.say(response).await?;
    }
    
    Ok(())
}

/// Reset all server data (Owner only)
///
/// This command is only usable by the bot owner and will reset all data for the current server
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn reset_server(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    
    // Check if the user is the owner
    if user_id != OWNER_ID {
        return Err("You are not authorized to use this command, comrade! Only the supreme leader may reset server data.".into());
    }
    
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };
    
    let server_name = match ctx.guild() {
        Some(guild) => guild.name.clone(),
        None => "this server".to_string(),
    };
    
    // Create confirmation message with buttons
    let msg = ctx.send(|m| {
        m.content(format!("⚠️ **DANGER ZONE** ⚠️\n\nYou are about to reset ALL data for **{}**.\nThis will:\n- Delete all user accounts\n- Reset all boops to 0\n- Remove all job assignments\n- Reset all statistics\n\nThis action cannot be undone! Are you sure?", server_name))
         .components(|c| {
             c.create_action_row(|row| {
                 row.create_button(|b| {
                     b.custom_id("confirm_reset")
                      .label("Yes, Reset Everything")
                      .style(serenity::ButtonStyle::Danger)
                      .emoji("🗑️".parse::<serenity::ReactionType>().unwrap())
                 })
                 .create_button(|b| {
                     b.custom_id("cancel_reset")
                      .label("Cancel")
                      .style(serenity::ButtonStyle::Secondary)
                 })
             })
         })
    }).await?;
    
    // Wait for button interaction
    if let Some(interaction) = serenity::CollectComponentInteraction::new(ctx)
        .filter(move |interaction| {
            // Only accept interactions from the original user
            interaction.user.id.to_string() == user_id
        })
        .timeout(Duration::from_secs(60))
        .await
    {
        // Get the custom_id to determine which button was pressed
        let custom_id = &interaction.data.custom_id;
        
        if custom_id == "confirm_reset" {
            // Acknowledge the interaction
            interaction.create_interaction_response(ctx, |r| {
                r.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
            }).await?;
            
            // Perform the reset
            let db = &ctx.data().db;
            db.reset_server_data(&server_id).await?;
            
            // Update the message to indicate success
            msg.edit(ctx, |m| {
                m.content(format!("✅ **Reset Complete**\n\nAll data for **{}** has been reset. The society can start anew!", server_name))
                 .components(|c| c)  // Remove all components
            }).await?;
        } else {
            // Cancel was pressed
            interaction.create_interaction_response(ctx, |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                 .interaction_response_data(|d| {
                     d.content("Operation cancelled. The glorious data of the motherland remains intact.")
                      .components(|c| c)  // Remove all components
                 })
            }).await?;
        }
    } else {
        // Timeout occurred
        msg.edit(ctx, |m| {
            m.content("Time expired. Reset operation cancelled.")
             .components(|c| c)  // Remove all components
        }).await?;
    }
    
    Ok(())
}

/// Distribute communal boops directly to all users
#[poise::command(slash_command, prefix_command, track_edits, check = "check_if_admin")]
pub async fn distribute(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };
    
    let db = &ctx.data().db;
    
    // Get current communal boops before distribution
    let communal_boops = db.get_communal_boops(&server_id).await?;
    
    if communal_boops <= 0.0 {
        ctx.say("There are no communal boops to distribute!").await?;
        return Ok(());
    }
    
    // Check how many users are in the database before distribution
    let user_count = db.get_server_user_count(&server_id).await?;
    
    // If the server seems to have few users, check the guild member count
    if user_count < 3 {
        let guild_id = ctx.guild_id().unwrap();
        
        match ctx.guild() {
            Some(guild) => {
                let member_count = guild.member_count;
                
                // If server actually has many users but few in database, suggest sync
                if member_count > user_count as u64 + 2 { // +2 to account for bots
                    let response = format!(
                        "⚠️ **Warning!** ⚠️\n\n\
                        I see only **{}** users in my database, but this server has **{}** members.\n\
                        If you distribute now, only the registered users will receive boops.\n\n\
                        Would you like to:\n\
                        1. Run `/sync_users` first to ensure all members are counted\n\
                        2. Continue with distribution anyway\n\
                        \n(Please type your choice number)",
                        user_count, member_count
                    );
                    
                    ctx.say(response).await?;
                    
                    // Wait for a response
                    let reply = poise::serenity_prelude::CollectReply::new(ctx)
                        .timeout(std::time::Duration::from_secs(30))
                        .await;
                    
                    if let Some(msg) = reply {
                        let content = msg.content.trim();
                        
                        if content == "1" {
                            // Run sync_users command manually
                            ctx.say("Running user sync first...").await?;
                            
                            // Code copied from sync_users command
                            let guild_id = ctx.guild_id().unwrap();
                            let server_id = guild_id.to_string();
                            let server_name = match ctx.guild() {
                                Some(guild) => guild.name.clone(),
                                None => "Unknown Server".to_string(),
                            };
                            
                            // Start syncing message
                            let sync_msg = ctx.say("☭ **Syncing server members** ☭\nPlease wait while I gather information about all comrades...").await?;
                            
                            // Get all members of the guild
                            let members = match guild_id.members(&ctx.serenity_context().http, None, None).await {
                                Ok(members) => members,
                                Err(e) => return Err(format!("Failed to get server members: {}", e).into()),
                            };
                            
                            // First ensure server exists
                            db.add_server(&server_id, &server_name).await?;
                            
                            // Counter for users added
                            let mut added_count = 0;
                            let mut updated_count = 0;
                            let total_users = members.iter().filter(|m| !m.user.bot).count();
                            
                            // Add each non-bot user to database
                            for member in members {
                                // Skip bots
                                if member.user.bot {
                                    continue;
                                }
                                
                                let user_id = member.user.id.to_string();
                                let username = member.user.name.clone();
                                
                                // Check if user exists
                                let user_exists = db.ensure_user_exists(&user_id, &server_id, &username).await?;
                                
                                if !user_exists {
                                    added_count += 1;
                                } else {
                                    updated_count += 1;
                                }
                            }
                            
                            // Update the message with results
                            sync_msg.edit(ctx, |m| {
                                m.content(format!(
                                    "☭ **Server Sync Complete** ☭\n\n\
                                    Found **{}** total non-bot users in **{}**\n\
                                    • **{}** new users added to database\n\
                                    • **{}** existing users verified\n\n\
                                    The database is now in sync with the server! Now proceeding with distribution.",
                                    total_users, server_name, added_count, updated_count
                                ))
                            }).await?;
                            
                            // Continue with distribution after sync
                            ctx.say("Now continuing with distribution...").await?;
                            
                            // Get updated user count
                            let new_user_count = db.get_server_user_count(&server_id).await?;
                            
                            // Get current distribution status before distribution
                            let (old_round, _, _) = db.get_distribution_status(&server_id).await?;
                            
                            // Distribute boops to all users
                            let (updated_user_count, share_per_user) = db.distribute_to_all_users(&server_id).await?;
                            
                            if updated_user_count == 0 {
                                ctx.say("Still no users to distribute boops to, or share per user is too small.").await?;
                                return Ok(());
                            }
                            
                            // Get new distribution status after distribution
                            let (new_round, _, _) = db.get_distribution_status(&server_id).await?;
                            
                            // Create response with confirmation
                            let response = format!(
                                "**☭ Boops Distributed After Sync! ☭**\n\n\
                                You have distributed **{:.2} boops** from the communal pool!\n\n\
                                • Each of the **{}** comrades received **{:.2} boops** directly to their personal account\n\
                                • The communal pool is now empty\n\
                                • Distribution round #{} is now complete\n\
                                • New distribution round #{} has begun\n\n\
                                The party acknowledges your generosity to the people, comrade!",
                                communal_boops, updated_user_count, share_per_user, old_round, new_round
                            );
                            
                            // Send confirmation
                            ctx.say(response).await?;
                            
                            return Ok(());
                        }
                        // If user typed anything else, continue with regular distribution
                    }
                }
            },
            None => {
                // Can't check member count, continue with distribution
            }
        }
    }
    
    // Get current distribution status before distribution
    let (old_round, _, _) = db.get_distribution_status(&server_id).await?;
    
    // Distribute boops to all users
    let (user_count, share_per_user) = db.distribute_to_all_users(&server_id).await?;
    
    if user_count == 0 {
        ctx.say("No users to distribute boops to, or share per user is too small.").await?;
        return Ok(());
    }
    
    // Get new distribution status after distribution
    let (new_round, _, _) = db.get_distribution_status(&server_id).await?;
    
    // Create response with confirmation
    let response = format!(
        "**☭ Boops Distributed! ☭**\n\n\
        You have distributed **{:.2} boops** from the communal pool!\n\n\
        • Each of the **{}** comrades received **{:.2} boops** directly to their personal account\n\
        • The communal pool is now empty\n\
        • Distribution round #{} is now complete\n\
        • New distribution round #{} has begun\n\n\
        The party acknowledges your generosity to the people, comrade!",
        communal_boops, user_count, share_per_user, old_round, new_round
    );
    
    // Send confirmation
    ctx.say(response).await?;
    
    Ok(())
}

/// Sync all server members to the database
#[poise::command(slash_command, prefix_command, track_edits, check = "check_if_admin")]
pub async fn sync_users(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Err("This command can only be used in a server!".into()),
    };
    
    let server_id = guild_id.to_string();
    let server_name = match ctx.guild() {
        Some(guild) => guild.name.clone(),
        None => "Unknown Server".to_string(),
    };
    
    // Start syncing message
    let msg = ctx.say("☭ **Syncing server members** ☭\nPlease wait while I gather information about all comrades...").await?;
    
    // Get all members of the guild
    let members = match guild_id.members(&ctx.serenity_context().http, None, None).await {
        Ok(members) => members,
        Err(e) => return Err(format!("Failed to get server members: {}", e).into()),
    };
    
    let db = &ctx.data().db;
    
    // First ensure server exists
    db.add_server(&server_id, &server_name).await?;
    
    // Counter for users added
    let mut added_count = 0;
    let mut updated_count = 0;
    let total_users = members.iter().filter(|m| !m.user.bot).count();
    
    // Add each non-bot user to database
    for member in members {
        // Skip bots
        if member.user.bot {
            continue;
        }
        
        let user_id = member.user.id.to_string();
        let username = member.user.name.clone();
        
        // Check if user exists
        let user_exists = db.ensure_user_exists(&user_id, &server_id, &username).await?;
        
        if !user_exists {
            added_count += 1;
        } else {
            updated_count += 1;
        }
    }
    
    // Update the message with results
    msg.edit(ctx, |m| {
        m.content(format!(
            "☭ **Server Sync Complete** ☭\n\n\
            Found **{}** total non-bot users in **{}**\n\
            • **{}** new users added to database\n\
            • **{}** existing users verified\n\n\
            The database is now in sync with the server! Future distribution commands will include all comrades.",
            total_users, server_name, added_count, updated_count
        ))
    }).await?;
    
    Ok(())
} 