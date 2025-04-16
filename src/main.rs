mod commands;
mod db;

use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use std::env;
use rand::Rng;
use serde::{Deserialize, Serialize};
use reqwest;

// Define a type for the user data that will be passed to all command functions
type CommandError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, CommandError>;

// User data that is stored and accessible in all command functions
pub struct Data {
    db: db::Database,
    // voting_state: Arc<Mutex<commands::VotingState>>,
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Get the Discord token from environment variables
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set in .env file");
    
    // Initialize the database
    let db_path = "boopato.db";
    let database = db::Database::new(db_path).await.expect("Failed to initialize database");
    
    // Define the framework configuration with all commands
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::work(),
                commands::commit(),
                commands::boops(),
                commands::claim(),
                commands::jobs_list(),
                commands::jobs_apply(),
                commands::jobs_quit(),
                commands::prosper(),
                commands::reset_cooldowns(),
                commands::list_users(),
                commands::reset_server(),
                commands::distribute(),
                commands::sync_users(),
                commands::game(),
                commands::tictactoe(),
                commands::clicker(),
                commands::connect4(),
                commands::kremlin_secrets(),
                commands::soviet_hangman(),
                commands::redistribute(),
                commands::schedule_redistribution(),
                commands::about(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_MEMBERS)
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Bot is running! The glory of communal boops awaits!");
                
                let db = database.clone();
                
                // Register all existing guild members in the database
                for guild in &ready.guilds {
                    let server_id = guild.id.to_string();
                    let server_name = guild.id.name(&ctx.cache).unwrap_or_else(|| "Unknown Server".to_string());
                    
                    // Add server to database
                    if let Err(e) = db.add_server(&server_id, &server_name).await {
                        eprintln!("Failed to add server: {}", e);
                        continue;
                    }
                    
                    // Get all members of the guild
                    match guild.id.members(&ctx.http, None, None).await {
                        Ok(members) => {
                            println!("Initializing {} members for server {}", members.len(), server_name);
                            for member in members {
                                // Skip bots
                                if member.user.bot {
                                    continue;
                                }
                                
                                let user_id = member.user.id.to_string();
                                let username = member.user.name.clone();
                                
                                // Add user to database
                                if let Err(e) = db.add_user(&user_id, &server_id, &username).await {
                                    eprintln!("Failed to add user: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to get members for guild {}: {}", guild.id, e);
                        }
                    }
                }
                
                Ok(Data {
                    db: database
                })
            })
        });

    // Start the bot
    framework.run().await.unwrap();
}

// Event handler to track messages for the yappers command
async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, CommandError>,
    data: &Data,
) -> Result<(), CommandError> {
    match event {
        poise::Event::Message { new_message } => {
            // Ignore bot messages
            if new_message.author.bot {
                return Ok(());
            }
            
            // Get user and server info
            let user_id = new_message.author.id.to_string();
            let username = new_message.author.name.clone();
            
            if let Some(guild_id) = new_message.guild_id {
                let server_id = guild_id.to_string();
                
                // Get server name
                let server_name = match guild_id.to_guild_cached(&ctx.cache) {
                    Some(guild) => guild.name.clone(),
                    None => "Unknown Server".to_string(),
                };
                
                // Add server if not exists
                if let Err(e) = data.db.add_server(&server_id, &server_name).await {
                    eprintln!("Failed to add server: {}", e);
                    return Ok(());
                }
                
                // Add user if not exists
                if let Err(e) = data.db.add_user(&user_id, &server_id, &username).await {
                    eprintln!("Failed to add user: {}", e);
                    return Ok(());
                }
                
                // Increment message count
                if let Err(e) = data.db.add_message_count(&user_id).await {
                    eprintln!("Failed to increment message count: {}", e);
                }
                
                // Check for collective words and react with a suitable standard emoji
                let content = new_message.content.to_lowercase();
                let collective_words = ["we", "our", "together", "comrade", "collective", "unity"];
                
                let should_react = collective_words.iter().any(|&word| {
                    content.split(|c: char| !c.is_alphanumeric()).any(|part| part == word)
                });
                
                // Random chance (10%) to react even if there's a match to avoid being too spammy
                let should_react_random = {
                    let mut rng = rand::thread_rng();
                    should_react && rng.gen_bool(1.0)
                };
                
                if should_react_random {
                    // React with the custom USSR Hammer emoji
                    let custom_emoji = serenity::ReactionType::Custom {
                        animated: false,
                        id: serenity::EmojiId(1353614767545389097),
                        name: Some("USSR_Hammer".to_string()),
                    };
                    
                    if let Err(e) = new_message.react(ctx, custom_emoji).await {
                        eprintln!("Failed to react with emoji: {}", e);
                    }
                }
                
                // Implement KGB listener functionality - random chance of the bot "overhearing" conversations
                // Only trigger on longer messages (>20 chars) with very low probability (0.5%)
                let should_trigger_kgb = {
                    let mut rng = rand::thread_rng();
                    content.len() > 20 && rng.gen_bool(1.0)
                };
                
                if should_trigger_kgb {
                    // Trigger KGB response
                    if let Err(e) = kgb_listener(ctx, new_message, &content).await {
                        eprintln!("Failed to process KGB listener: {}", e);
                    }
                }
            }
        },
        poise::Event::GuildMemberAddition { new_member } => {
            // Skip bots
            if new_member.user.bot {
                return Ok(());
            }
            
            let user_id = new_member.user.id.to_string();
            let username = new_member.user.name.clone();
            let server_id = new_member.guild_id.to_string();
            
            // Get server name
            let server_name = match new_member.guild_id.to_guild_cached(&ctx.cache) {
                Some(guild) => guild.name.clone(),
                None => "Unknown Server".to_string(),
            };
            
            // Add server if not exists
            if let Err(e) = data.db.add_server(&server_id, &server_name).await {
                eprintln!("Failed to add server: {}", e);
                return Ok(());
            }
            
            // Add user if not exists
            if let Err(e) = data.db.add_user(&user_id, &server_id, &username).await {
                eprintln!("Failed to add user: {}", e);
                return Ok(());
            }
            
            println!("New member added to database: {} in server {}", username, server_name);
        },
        poise::Event::GuildCreate { guild, is_new } => {
            let guild_id = guild.id;
            let server_id = guild_id.to_string();
            let server_name = guild.name.clone();
            
            // Add server to database
            if let Err(e) = data.db.add_server(&server_id, &server_name).await {
                eprintln!("Failed to add server: {}", e);
                return Ok(());
            }
            
            println!("Bot added to guild: {} ({})", server_name, server_id);
            
            // Only show welcome message if this is a brand new join, not a reconnect after restart
            if *is_new {
                println!("Bot is joining guild {} for the first time", server_name);
                
                if let Some(channel_id) = guild.system_channel_id {
                    let _ = channel_id.say(
                        &ctx.http, 
                        "☭ **Greetings, comrades!** ☭\n\
                        I am Boopato, the glorious redistributor of boops!\n\
                        I'm currently in the process of gathering information about all comrades in this server...\n\
                        This may take a while for larger servers."
                    ).await;
                }
            }
            
            // Fetch all members
            match guild_id.members(&ctx.http, None, None).await {
                Ok(members) => {
                    println!("Syncing {} members for server {}", members.len(), server_name);
                    let mut added_count = 0;
                    
                    // Count non-bot members before consuming the collection
                    let non_bot_count = members.iter().filter(|m| !m.user.bot).count();
                    
                    for member in members {
                        // Skip bots
                        if member.user.bot {
                            continue;
                        }
                        
                        let user_id = member.user.id.to_string();
                        let username = member.user.name.clone();
                        
                        // Add user to database
                        match data.db.ensure_user_exists(&user_id, &server_id, &username).await {
                            Ok(already_existed) => {
                                if !already_existed {
                                    added_count += 1;
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to add user: {}", e);
                            }
                        }
                    }
                    
                    println!("Added {} new users to database for server {}", added_count, server_name);
                    
                    // Only show completion message if this is a brand new join
                    if *is_new {
                        if let Some(channel_id) = guild.system_channel_id {
                            let _ = channel_id.say(
                                &ctx.http, 
                                format!(
                                    "☭ **Setup complete!** ☭\n\
                                    I have registered **{}** comrades in this server.\n\
                                    Use `/help` to see all available commands.\n\
                                    Glory to the collective!",
                                    non_bot_count
                                )
                            ).await;
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to get members for guild {}: {}", guild_id, e);
                    
                    // Only show error message if this is a brand new join
                    if *is_new {
                        if let Some(channel_id) = guild.system_channel_id {
                            let _ = channel_id.say(
                                &ctx.http, 
                                "⚠️ **Comrade, there was an issue!** ⚠️\n\
                                I was unable to register all server members. Some features may not work correctly.\n\
                                An admin should use the `/sync_users` command to manually sync all users."
                            ).await;
                        }
                    }
                }
            }
        },
        _ => {}
    }
    
    Ok(())
}

// KGB listener feature - bot "overhears" conversations and comments
async fn kgb_listener(
    ctx: &serenity::Context, 
    message: &serenity::Message,
    content: &str
) -> Result<(), CommandError> {
    // Check for specific trigger phrases with higher priority
    let triggers = [
        ("revolution", "The KGB is watching your revolutionary activities with great interest, comrade..."),
        ("overthrow", "Plotting against the state, are we? The KGB has noted your... enthusiasm."),
        ("capitalism", "Ah, discussing the enemy's economic system? The KGB approves of your educational pursuits."),
        ("freedom", "Freedom? The KGB reminds you that true freedom comes through service to the state."),
        ("western", "The KGB advises caution when discussing western influences, comrade."),
    ];
    
    // Try to fetch Groq API key from environment variables
    let groq_api_key = env::var("GROQ_API_KEY").ok();
    
    if let Some(api_key) = groq_api_key {
        // We have an API key, so let's use Groq to generate a response
        // Create a typing indicator to show the bot is thinking
        let _ = message.channel_id.broadcast_typing(&ctx.http).await;
        
        // Check if any high-priority triggers match first
        for (trigger, _) in triggers.iter() {
            if content.contains(trigger) {
                // For triggered words, use Groq API to generate a themed response
                if let Ok(response) = generate_kgb_response(&api_key, content, trigger).await {
                    // If we get a valid response, use it
                    let _ = message.reply(ctx, response).await;
                    return Ok(());
                }
            }
        }
        
        // If no specific triggers or API call failed, fall back to generic responses
        if let Ok(response) = generate_kgb_response(&api_key, content, "").await {
            let _ = message.reply(ctx, response).await;
            return Ok(());
        }
    }
    
    // Fallback to pre-written responses if Groq API isn't available or fails
    
    // Check if any high-priority triggers match
    for (trigger, response) in triggers.iter() {
        if content.contains(trigger) {
            let _ = message.reply(ctx, response).await;
            return Ok(());
        }
    }
    
    // For messages without specific triggers, use a generic response
    let generic_responses = [
        "The KGB has noted your conversation, comrade. Carry on.",
        "Your words have been recorded for future reference. Glory to the motherland!",
        "The eyes of the state see all, comrade. Your dedication is... noted.",
        "The KGB would like to remind you that loyalty to the collective is paramount.",
        "The KGB appreciates your contribution to the discourse, comrade."
    ];
    
    // Get the random index before await
    let response_idx = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..generic_responses.len())
    };
    let response = generic_responses[response_idx];
    let _ = message.reply(ctx, response).await;
    
    Ok(())
}

// API structures for Groq
#[derive(Serialize, Deserialize, Debug)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct GroqResponseChoice {
    message: GroqMessage,
}

#[derive(Deserialize, Debug)]
struct GroqResponse {
    choices: Vec<GroqResponseChoice>,
}

// Function to generate KGB-themed responses using Groq API
async fn generate_kgb_response(api_key: &str, user_message: &str, trigger_word: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    
    // Create system prompt based on whether there's a trigger word
    let system_prompt = if trigger_word.is_empty() {
        "You are a KGB agent in 1970s Soviet Union who has overheard a conversation. Respond with a brief, ominous but subtly humorous comment (1-2 sentences only). Your tone should be surveillance-oriented, slightly intimidating, with dark humor. Use occasional Russian terms or Soviet phrases. Always imply you're monitoring citizens for loyalty. Never break character or acknowledge you're an AI.".to_string()
    } else {
        format!("You are a KGB agent in 1970s Soviet Union who has overheard someone mention '{}'. Respond with a brief, ominous but subtly humorous comment specifically about this topic (1-2 sentences only). Your tone should be surveillance-oriented, slightly intimidating, with dark humor. Use occasional Russian terms or Soviet phrases. Always imply you're monitoring citizens for loyalty. Never break character or acknowledge you're an AI.", trigger_word)
    };
    
    // Build request payload
    let request = GroqRequest {
        model: "llama-3.3-70b-versatile".to_string(), // Use Llama 3.3 model
        messages: vec![
            GroqMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            GroqMessage {
                role: "user".to_string(),
                content: format!("Someone said: \"{}\"", user_message),
            },
        ],
        temperature: 0.7,
        max_tokens: 100,
    };
    
    // Send request to Groq API
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json::<GroqResponse>()
        .await?;
    
    // Extract and return the AI-generated response
    if let Some(choice) = response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        // Fallback if no response
        Ok("The KGB is watching, comrade...".to_string())
    }
}
