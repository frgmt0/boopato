use crate::CommandError;
use crate::db::JobType;
use rand::Rng;

/// Work in the mines and earn boops for the community
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn work(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Check if on cooldown (3 hours)
    if let Some(last_work) = db.get_last_work(&user_id).await? {
        let now = chrono::Utc::now().timestamp();
        let diff = now - last_work;
        
        if diff < 3 * 60 * 60 {
            let remaining = 3 * 60 * 60 - diff;
            let hours = remaining / 3600;
            let minutes = (remaining % 3600) / 60;
            
            return Err(format!("You must rest, comrade! Try again in {} hours and {} minutes.", hours, minutes).into());
        }
    }
    
    // Get user's job
    let job_name = db.get_user_job(&user_id).await?;
    let job = JobType::from_string(&job_name);
    let job_level = db.get_job_level(&user_id).await?;
    
    // Calculate boops to add
    let base_boops = 5.0;
    let mut bonus_multiplier = 1.0;
    
    // Apply job multiplier if user has a job
    if job != JobType::None {
        bonus_multiplier *= job.get_boops_multiplier();
        // Also apply job level bonus (5% per level)
        bonus_multiplier += (job_level as f64 - 1.0) * 0.05;
    }
    
    // Random chance for bonus (0 to 50% more)
    let random_bonus = rand::thread_rng().gen_range(0.0..=0.5);
    bonus_multiplier += random_bonus;
    
    // Apply total multiplier
    let earned_boops = (base_boops * bonus_multiplier * 100.0).round() / 100.0;
    
    // Add 90% to communal boops
    let communal_amount = (earned_boops * 0.9 * 100.0).round() / 100.0;
    let personal_bonus = earned_boops - communal_amount;
    
    // Update database (these operations involve awaits)
    db.distribute_boops(&server_id, communal_amount).await?;
    db.add_user_boops(&user_id, personal_bonus).await?;
    db.update_last_work(&user_id).await?;
    
    // Create response
    let mut response = String::new();
    
    match job {
        JobType::None => {
            response.push_str("You toil without purpose, contributing the bare minimum to society.\n");
        },
        JobType::Miner => {
            response.push_str("You descend into the mines and extract precious resources for the collective.\n");
        },
        JobType::Farmer => {
            response.push_str("You tend the fields, growing sustenance for your fellow comrades.\n");
        },
        JobType::Programmer => {
            response.push_str("You write efficient code to automate systems for the benefit of all.\n");
        },
        JobType::Teacher => {
            response.push_str("You educate the youth, preparing them for service to society.\n");
        },
        JobType::Doctor => {
            response.push_str("You heal the sick, ensuring the workforce remains productive.\n");
        },
    }
    
    response.push_str(&format!(
        "\nYou have earned {:.2} boops for your labor!\n",
        earned_boops
    ));
    
    if job != JobType::None {
        response.push_str(&format!(
            "Your job as a {} (Level {}) provided a multiplier of {:.2}x\n",
            job.to_string(), job_level, job.get_boops_multiplier() + (job_level as f64 - 1.0) * 0.05
        ));
    }
    
    response.push_str(&format!(
        "\n{:.2} boops added to the communal pool, and {:.2} boops added to your personal balance as a work bonus!\n",
        communal_amount, personal_bonus
    ));
    
    response.push_str("\nGlory to those who serve the collective!");
    
    ctx.say(response).await?;
    
    Ok(())
} 