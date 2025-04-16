use crate::CommandError;
use crate::db::JobType;
use poise::serenity_prelude as serenity;
use rand::{Rng, seq::SliceRandom};

/// Work and contribute to the glory of the state
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
            // Calculate when the cooldown will end
            let cooldown_end = last_work + (3 * 60 * 60);
            
            ctx.send(|m| {
                m.embed(|e| {
                    e.title("☭ Labor Assignment Rejected ☭")
                     .description("Rest period enforced by the Department of Worker Welfare")
                     .color(serenity::Color::from_rgb(139, 0, 0)) // Dark red
                     .field(
                        "Notice from Labor Committee",
                        "Comrade, you must observe mandatory rest periods between work shifts.",
                        false
                     )
                     .field(
                        "Remaining Rest Time",
                        format!("Your next work shift is permitted <t:{}:R>", cooldown_end),
                        false
                     )
                     .footer(|f| f.text("The State values your wellbeing as much as your productivity."))
                })
            }).await?;
            
            return Ok(());
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
    
    // Job-specific assets and descriptions
    let (job_title, job_description, job_thumbnail) = match job {
        JobType::None => (
            "Unskilled Laborer",
            "You toil without specialized skills, performing basic tasks for the collective. Your contributions, while modest, still fulfill the fundamental needs of society.",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/8/80/Farm-Fresh_user.png/240px-Farm-Fresh_user.png"
        ),
        JobType::Miner => (
            "Proletarian Miner",
            "You descend bravely into the depths of the earth, extracting vital resources that fuel our industrial might. Your labor in the darkness brings light to the collective's future.",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/7/7a/Emojione_1F528.svg/240px-Emojione_1F528.svg.png"
        ),
        JobType::Farmer => (
            "Agricultural Specialist",
            "You cultivate the fertile soil of the motherland, providing nourishment for all citizens. Your connection to the land embodies the pure spirit of collective prosperity.",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/f/f7/Emojione_1F33D.svg/240px-Emojione_1F33D.svg.png"
        ),
        JobType::Programmer => (
            "Technical Engineer",
            "You harness the power of computation to automate and enhance our society's systems. Your mastery of logic and code advances our technological revolution.",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/0/0f/Emojione_1F4BB.svg/240px-Emojione_1F4BB.svg.png"
        ),
        JobType::Teacher => (
            "Education Commissar",
            "You enlighten the minds of young comrades, molding them into productive citizens faithful to our collective principles. Your wisdom shapes the future of our society.",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/2/24/Emojione_1F4DA.svg/240px-Emojione_1F4DA.svg.png"
        ),
        JobType::Doctor => (
            "Healthcare Provider",
            "You maintain the health and vigor of our workforce, ensuring optimal productivity through medical science. Your healing touch protects our most valuable resource: our citizens.",
            "https://upload.wikimedia.org/wikipedia/commons/thumb/0/04/Emojione_1FA7A.svg/240px-Emojione_1FA7A.svg.png"
        ),
    };
    
    // Job level titles
    let level_title = match job_level {
        1 => "Apprentice",
        2..=3 => "Practitioner",
        4..=6 => "Expert",
        7..=9 => "Master",
        _ => "Grandmaster"
    };
    
    // Random labor achievement descriptions
    let labor_achievements = [
        "Your productivity exceeds expectations",
        "You have fulfilled your quota with remarkable efficiency",
        "The state acknowledges your extraordinary effort",
        "Your dedication to labor is an example to all comrades",
        "The central committee recognizes your exceptional contribution",
        "Your work ethic brings honor to your labor brigade",
        "You have displayed ideological correctness in your labor approach",
        "The quality of your work surpasses established standards",
        "Your innovative methods have benefited the collective",
        "You demonstrate unwavering commitment to productive labor"
    ];
    
    let achievement = labor_achievements.choose(&mut rand::thread_rng()).unwrap_or(&labor_achievements[0]);
    
    // Progress to next level calculation
    // Using random bonus for "progress" - next_level_requirement is used for display
    let level_progress = ((random_bonus / 0.5) * 100.0).round() as i32;
    
    // Progress bar for level advancement
    let blocks = (level_progress / 10) as usize;
    let filled = "█".repeat(blocks);
    let empty = "░".repeat(10 - blocks);
    let progress_bar = format!("{filled}{empty} ({level_progress}%)");
    
    // Create embed response
    ctx.send(|m| {
        m.embed(|e| {
            // Basic embed structure
            e.title("☭ Labor Contribution Report ☭")
             .description(format!("Work Assignment: {}", job_title))
             .color(serenity::Color::RED)
             .thumbnail(job_thumbnail);
            
            // Job status and description
            if job != JobType::None {
                e.field(
                    format!("{} {} (Level {})", level_title, job.to_string(), job_level),
                    job_description,
                    false
                );
            } else {
                e.field(
                    "Unspecialized Labor",
                    job_description,
                    false
                );
            }
            
            // Labor results
            e.field(
                "Labor Output",
                format!("**{:.2}** boops produced through your labor", earned_boops),
                true
            );
            
            // Multiplier breakdown
            if job != JobType::None {
                let job_multiplier = job.get_boops_multiplier();
                let level_bonus = (job_level as f64 - 1.0) * 0.05;
                let display_job_bonus = if job_multiplier > 1.0 { format!("+{:.0}%", (job_multiplier - 1.0) * 100.0) } else { "0%".to_string() };
                let display_level_bonus = if level_bonus > 0.0 { format!("+{:.0}%", level_bonus * 100.0) } else { "0%".to_string() };
                let display_random_bonus = format!("+{:.0}%", random_bonus * 100.0);
                
                e.field(
                    "Productivity Factors",
                    format!(
                        "Job Specialty: **{}**\nExperience Bonus: **{}**\nExceptional Effort: **{}**",
                        display_job_bonus, display_level_bonus, display_random_bonus
                    ),
                    true
                );
            } else {
                e.field(
                    "Productivity Factors",
                    format!("Exceptional Effort: **+{:.0}%**", random_bonus * 100.0),
                    true
                );
            }
            
            // Distribution information
            e.field(
                "Resource Distribution",
                format!(
                    "Communal Fund: **{:.2}** boops (90%)\nPersonal Bonus: **{:.2}** boops (10%)",
                    communal_amount, personal_bonus
                ),
                false
            );
            
            // Advancement progress
            if job != JobType::None {
                e.field(
                    "Skill Advancement",
                    format!(
                        "Progress to Level {}:\n{}",
                        job_level + 1, progress_bar
                    ),
                    false
                );
            }
            
            // Achievement and footer
            e.field(
                "Labor Achievement",
                format!("**{achievement}**"),
                false
            )
            .footer(|f| f.text("Glory to labor! Your contribution strengthens the collective."))
        })
    }).await?;
    
    Ok(())
} 