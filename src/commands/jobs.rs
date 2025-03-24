use crate::db::JobType;
use crate::Data;
use crate::CommandError;
use poise::serenity_prelude as serenity;
use rand::Rng;

/// View the available jobs in our collective
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn jobs_list(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Get current user job
    let current_job_str = db.get_user_job(&user_id).await?;
    let current_job = JobType::from_string(&current_job_str);
    let job_level = db.get_job_level(&user_id).await?;
    
    let mut response = "**Jobs Available in the Collective**\n\n".to_string();
    
    if current_job != JobType::None {
        response.push_str(&format!("**Your Current Position: {}** (Level {})\n", 
            current_job.to_string(), job_level));
        response.push_str(&format!("*{}*\n", current_job.get_description()));
        response.push_str(&format!("Work Efficiency: {}x boops multiplier\n\n", 
            current_job.get_boops_multiplier()));
    } else {
        response.push_str("**You are currently unemployed.**\n");
        response.push_str("The Party is disappointed in your lack of contribution to society.\n\n");
    }
    
    response.push_str("**Available Positions:**\n");
    
    for job in JobType::list_all() {
        let status = if current_job == job { " (Your current job)" } else { "" };
        response.push_str(&format!("- **{}**{}: {}\n", 
            job.to_string(), status, job.get_description()));
        response.push_str(&format!("  Work Efficiency: {}x boops multiplier\n\n", 
            job.get_boops_multiplier()));
    }
    
    response.push_str("\nApply for a job with `/jobs-apply <job>` to show your dedication to the collective!");
    
    ctx.say(response).await?;
    
    Ok(())
}

/// Apply for a job in our glorious collective (50% success rate)
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn jobs_apply(
    ctx: crate::Context<'_>,
    #[description = "The job you wish to apply for"] job_name: String,
) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Check if job exists
    let job = JobType::from_string(&job_name.to_lowercase());
    if job == JobType::None {
        let valid_jobs = JobType::list_all()
            .iter()
            .map(|j| j.to_string())
            .collect::<Vec<String>>()
            .join(", ");
            
        return Err(format!("That is not a valid job. Available jobs are: {}", valid_jobs).into());
    }
    
    // Get current job
    let current_job_str = db.get_user_job(&user_id).await?;
    let current_job = JobType::from_string(&current_job_str);
    
    // Check if already in this job
    if current_job == job {
        return Err(format!("You are already working as a {}!", job.to_string()).into());
    }
    
    // 50% chance of success
    let success = rand::thread_rng().gen_ratio(1, 2);
    
    if success {
        // Update job in database
        db.set_user_job(&user_id, &job.to_string()).await?;
        
        // Success message
        let messages = [
            "Your application has been accepted! Welcome to your new role as a {}!",
            "The Party recognizes your skills and has assigned you to work as a {}!",
            "Congratulations, comrade! You are now officially a {}!",
            "Your transition to {} has been approved by the central committee!",
            "The collective welcomes you to your new position as a {}!",
        ];
        
        let idx = rand::thread_rng().gen_range(0..messages.len());
        let message_template = messages[idx];
        let message = message_template.replace("{}", &job.to_string());
        
        ctx.say(format!(
            "{}\n\nYou will now earn {}x boops when working for the collective.",
            message,
            job.get_boops_multiplier()
        )).await?;
    } else {
        // Failure messages
        let messages = [
            "Your application to become a {} has been denied. The current holder of this position is too valuable to replace.",
            "The Party has reviewed your application for {} and found it lacking in revolutionary spirit.",
            "Your request to work as a {} cannot be fulfilled at this time. Perhaps you need more experience?",
            "The Committee has decided that you are not yet ready for the responsibility of being a {}.",
            "Application rejected. The collective needs you in your current role more than as a {}.",
        ];
        
        let idx = rand::thread_rng().gen_range(0..messages.len());
        let message_template = messages[idx];
        let message = message_template.replace("{}", &job.to_string());
        
        ctx.say(format!(
            "{}\n\nTry applying again later, comrade.",
            message
        )).await?;
    }
    
    Ok(())
}

/// Quit your current job (not recommended for loyal comrades)
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn jobs_quit(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Get current job
    let current_job_str = db.get_user_job(&user_id).await?;
    let current_job = JobType::from_string(&current_job_str);
    
    // Check if already unemployed
    if current_job == JobType::None {
        return Err("You cannot quit a job you don't have! The Party encourages you to seek employment.".into());
    }
    
    // Set to unemployed
    db.set_user_job(&user_id, "none").await?;
    
    // Responses
    let messages = [
        "You have abandoned your post as a {}. The Party is disappointed in your lack of commitment.",
        "Your resignation as a {} has been noted in your permanent record. This will not be forgotten.",
        "You are no longer a {}. Your comrades will remember this betrayal.",
        "The collective acknowledges your departure from the {} position. We expected better from you.",
        "Your service as a {} has been terminated. The People had higher hopes for you.",
    ];
    
    let idx = rand::thread_rng().gen_range(0..messages.len());
    let message_template = messages[idx];
    let message = message_template.replace("{}", &current_job.to_string());
    
    ctx.say(format!(
        "{}\n\nYou are now unemployed. Your work efficiency has been reduced to 1x. The Party encourages you to find new employment quickly.",
        message
    )).await?;
    
    Ok(())
}

/// Prosper through your contributions to the collective!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn prosper(ctx: crate::Context<'_>) -> Result<(), CommandError> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => return Err("This command can only be used in a server!".into()),
    };

    let db = &ctx.data().db;
    
    // Ensure user exists in database before proceeding
    db.ensure_user_exists(&user_id, &server_id, &username).await?;
    
    // Get current job
    let current_job_str = db.get_user_job(&user_id).await?;
    let current_job = JobType::from_string(&current_job_str);
    
    // Check if unemployed
    if current_job == JobType::None {
        return Err("You must first find employment with `/jobs-apply` before you can prosper!".into());
    }
    
    // Get current job level
    let current_level = db.get_job_level(&user_id).await?;
    let max_level = 10;
    
    if current_level >= max_level {
        return Err(format!("You have already reached the maximum level ({}) as a {}! The Party applauds your dedication.", max_level, current_job.to_string()).into());
    }
    
    // 33% chance to level up
    let success = rand::thread_rng().gen_ratio(1, 3);
    
    if success {
        // Increment job level
        db.increment_job_level(&user_id).await?;
        let new_level = current_level + 1;
        
        // Success messages
        let messages = [
            "Your proposal has been accepted by the Party leadership! Your {} skills have improved.",
            "The central committee acknowledges your contribution as a {}! Your efficiency has increased.",
            "Your business proposal to improve {} operations has yielded great results!",
            "The collective has recognized your dedication as a {}. You have been promoted!",
            "Your innovative approach to {} duties has not gone unnoticed by the Party!",
        ];
        
        let idx = rand::thread_rng().gen_range(0..messages.len());
        let message_template = messages[idx];
        let message = message_template.replace("{}", &current_job.to_string());
        
        // Calculate new multiplier (base multiplier + 0.1 per level)
        let base_multiplier = current_job.get_boops_multiplier();
        let level_bonus = (new_level - 1) as f64 * 0.1;
        let new_multiplier = base_multiplier + level_bonus;
        
        ctx.say(format!(
            "{}\n\nYou have advanced to level {} as a {}!\nYour new work efficiency is {:.1}x boops multiplier.",
            message,
            new_level,
            current_job.to_string(),
            new_multiplier
        )).await?;
    } else {
        // Failure messages
        let messages = [
            "Your proposal to improve {} operations has been rejected for insufficient revolutionary zeal.",
            "The committee has reviewed your business plan for the {} department and found it lacking.",
            "Your request for additional resources in the {} sector cannot be fulfilled at this time.",
            "The Party leadership does not see merit in your {} improvement suggestions.",
            "Your efficiency report as a {} shows you are not yet ready for advancement.",
        ];
        
        let idx = rand::thread_rng().gen_range(0..messages.len());
        let message_template = messages[idx];
        let message = message_template.replace("{}", &current_job.to_string());
        
        ctx.say(format!(
            "{}\n\nYou remain at level {} as a {}. Try again after contributing more to the collective.",
            message,
            current_level,
            current_job.to_string()
        )).await?;
    }
    
    Ok(())
} 