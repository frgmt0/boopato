use crate::CommandError;
use rand::Rng;

const COMMIT_COOLDOWN_SECS: u64 = 1800; // 30 minutes cooldown

/// Commit crimes against the nation (or acts of communism)
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn commit(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    // Get server name for flavor text
    let server_name = match ctx.guild() {
        Some(guild) => guild.name.clone(),
        None => "The State".to_string(),
    };

    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Check cooldown
    if let Ok(Some(last_commit)) = db.get_last_commit(&user_id).await {
        let now = chrono::Utc::now().timestamp();
        let elapsed = now - last_commit;
        
        if elapsed < COMMIT_COOLDOWN_SECS as i64 {
            // Calculate when cooldown ends
            let cooldown_end = last_commit + COMMIT_COOLDOWN_SECS as i64;
            
            ctx.say(format!(
                "Comrade, you must lay low until <t:{cooldown_end}:R> before committing any more acts!",
            )).await?;
            return Ok(());
        }
    }
    
    // Update last commit time
    db.update_last_commit(&user_id).await?;
    
    // Determine if it's a crime or communism (20% chance for communism)
    let is_communism = rand::thread_rng().gen_ratio(1, 5);
    
    if is_communism {
        // Commit an act of communism - calculate boops
        let total_boops = 5.0;
        
        // Determine how much goes to communal vs. personal
        let communal_amount = (((total_boops * 0.8) * 100.0) as f64).round() / 100.0;
        let personal_bonus = total_boops - communal_amount;
        
        // Select a random act before any awaits
        let communism_acts = [
            "established a community garden",
            "organized a neighborhood cleanup",
            "started a mutual aid network",
            "distributed bread to the hungry",
            "founded a workers' cooperative",
            "created a free library",
            "organized a clothing swap",
            "set up a tool sharing program",
        ];
        
        let act_index = rand::thread_rng().gen_range(0..communism_acts.len());
        let act = communism_acts[act_index];
        
        // Add to communal pool and personal balance (awaits here)
        db.distribute_boops(&server_id, communal_amount).await?;
        db.add_user_boops(&user_id, personal_bonus).await?;
        
        let response = format!(
            "**You've committed an act of COMMUNISM!** 🌟\n\n\
            You {act}! Your service to the community has been recognized.\n\n\
            {:.2} boops have been added to the communal pool, and you've received {:.2} boops as a personal bonus for your initiative!",
            communal_amount, personal_bonus
        );
        
        ctx.say(response).await?;
    } else {
        // Commit a crime - risk getting caught
        let crime_outcomes = [
            ("stole a loaf of bread", -5.0, false),
            ("skipped mandatory party meeting", -10.0, false),
            ("distributed unauthorized literature", -15.0, true),
            ("hoarded potatoes", -8.0, false),
            ("vandalized party propaganda", -12.0, true),
            ("listened to capitalist radio", -7.0, false),
            ("wore blue jeans", -5.0, false),
            ("spoke out against the leadership", -20.0, true),
        ];
        
        // Select crime and determine outcomes before awaits
        let crime_index = rand::thread_rng().gen_range(0..crime_outcomes.len());
        let (crime, penalty, caught) = crime_outcomes[crime_index];
        
        // 60% chance of getting caught if the crime is marked as high risk
        let got_caught = caught && rand::thread_rng().gen_ratio(3, 5);
        
        if got_caught {
            // Get boops before await
            let current_boops = db.get_user_boops(&user_id).await?;
            
            // Update boops (await here)
            db.update_user_boops(&user_id, current_boops + penalty).await?;
            
            let response = format!(
                "**CRIMINAL ALERT!** 🚨\n\nYou {crime} in {} and got caught!\n\nThe secret police have fined you {:.2} boops for your crimes against the state.",
                server_name, -penalty
            );
            
            ctx.say(response).await?;
        } else {
            // Got away with it!
            let response = format!(
                "**You've committed a crime!** 🤫\n\nYou {crime} in {} and got away with it!\n\nWhile your actions are capitalist in nature, your cunning is commendable. Be more careful next time, comrade.",
                server_name
            );
            
            ctx.say(response).await?;
        }
    }
    
    Ok(())
} 