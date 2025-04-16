use crate::CommandError;
use poise::serenity_prelude as serenity;
use std::time::Duration;
use rand::seq::SliceRandom;
use std::collections::HashSet;

// Kremlin Secrets Game - Word Association Challenge
struct KremlinSecrets {
    target_word: String,
    category: String,
    difficulty: KremlinDifficulty,
    guesses: Vec<(String, TemperatureLevel)>,
    max_guesses: usize,
    game_over: bool,
    won: bool,
}

#[derive(Clone, Copy)]
enum KremlinDifficulty {
    Citizen,    // Easy
    Comrade,    // Medium
    Commissar,  // Hard
}

#[derive(Clone, Copy)]
enum TemperatureLevel {
    Freezing,
    Cold,
    Cool,
    Warm,
    Hot,
    Burning,
}

impl KremlinDifficulty {
    fn max_guesses(&self) -> usize {
        match self {
            KremlinDifficulty::Citizen => 12,
            KremlinDifficulty::Comrade => 10,
            KremlinDifficulty::Commissar => 8,
        }
    }
    
    fn display_name(&self) -> &str {
        match self {
            KremlinDifficulty::Citizen => "Citizen",
            KremlinDifficulty::Comrade => "Comrade",
            KremlinDifficulty::Commissar => "Party Commissar",
        }
    }
}

impl TemperatureLevel {
    fn from_similarity(similarity: f64) -> Self {
        if similarity >= 0.85 {
            TemperatureLevel::Burning
        } else if similarity >= 0.7 {
            TemperatureLevel::Hot
        } else if similarity >= 0.5 {
            TemperatureLevel::Warm
        } else if similarity >= 0.3 {
            TemperatureLevel::Cool
        } else if similarity >= 0.15 {
            TemperatureLevel::Cold
        } else {
            TemperatureLevel::Freezing
        }
    }
    
    fn to_emoji(&self) -> &str {
        match self {
            TemperatureLevel::Freezing => "🥶",
            TemperatureLevel::Cold => "❄️",
            TemperatureLevel::Cool => "🧊",
            TemperatureLevel::Warm => "🔥",
            TemperatureLevel::Hot => "♨️",
            TemperatureLevel::Burning => "💥",
        }
    }
    
    fn to_color(&self) -> serenity::Color {
        match self {
            TemperatureLevel::Freezing => serenity::Color::from_rgb(0, 0, 255),   // Blue
            TemperatureLevel::Cold => serenity::Color::from_rgb(0, 191, 255),     // Deep Sky Blue
            TemperatureLevel::Cool => serenity::Color::from_rgb(0, 255, 255),     // Cyan
            TemperatureLevel::Warm => serenity::Color::from_rgb(255, 165, 0),     // Orange
            TemperatureLevel::Hot => serenity::Color::from_rgb(255, 69, 0),       // Orange Red
            TemperatureLevel::Burning => serenity::Color::from_rgb(255, 0, 0),    // Red
        }
    }
    
    fn to_description(&self) -> &str {
        match self {
            TemperatureLevel::Freezing => "You're in Siberia territory, comrade.",
            TemperatureLevel::Cold => "That word feels like a Moscow winter.",
            TemperatureLevel::Cool => "Getting closer, but still quite distant.",
            TemperatureLevel::Warm => "The KGB is monitoring your progress with interest.",
            TemperatureLevel::Hot => "The Politburo is impressed with your thinking!",
            TemperatureLevel::Burning => "You're practically touching state secrets!",
        }
    }
}

impl KremlinSecrets {
    fn new(difficulty: KremlinDifficulty) -> Self {
        // Pick a random category and word
        let (category, word) = Self::random_word_and_category();
        
        KremlinSecrets {
            target_word: word,
            category,
            difficulty,
            guesses: Vec::new(),
            max_guesses: difficulty.max_guesses(),
            game_over: false,
            won: false,
        }
    }
    
    fn random_word_and_category() -> (String, String) {
        let categories = [
            ("Soviet Leaders", vec![
                "Lenin", "Stalin", "Khrushchev", "Brezhnev", "Gorbachev", 
                "Andropov", "Chernenko", "Malenkov", "Bulganin", "Kosygin"
            ]),
            ("Revolutionary Concepts", vec![
                "Communism", "Socialism", "Proletariat", "Bourgeoisie", "Revolution",
                "Collectivization", "Vanguard", "Dialectic", "Materialism", "Classless"
            ]),
            ("Soviet Geography", vec![
                "Moscow", "Leningrad", "Kiev", "Stalingrad", "Minsk",
                "Vladivostok", "Siberia", "Ural", "Crimea", "Volga"
            ]),
            ("Cold War Terms", vec![
                "Detente", "Containment", "Iron Curtain", "Berlin Wall", "Glasnost",
                "Perestroika", "Domino Theory", "Missiles", "Espionage", "Defector"
            ]),
            ("Soviet Achievements", vec![
                "Sputnik", "Gagarin", "Vostok", "Collective", "Industrialization",
                "Literacy", "Electrification", "Space Station", "Olympic", "Science"
            ])
        ];
        
        let mut rng = rand::thread_rng();
        let (category, words) = categories.choose(&mut rng).unwrap();
        let word = words.choose(&mut rng).unwrap();
        
        (category.to_string(), word.to_string())
    }
    
    fn make_guess(&mut self, guess: &str) -> TemperatureLevel {
        // Convert guess to lowercase
        let guess = guess.trim().to_lowercase();
        let target = self.target_word.to_lowercase();
        
        // Check if the guess is correct
        if guess == target {
            self.game_over = true;
            self.won = true;
            return TemperatureLevel::Burning;
        }
        
        // Calculate similarity score (simple version)
        let similarity = self.calculate_similarity(&guess, &target);
        let temperature = TemperatureLevel::from_similarity(similarity);
        
        // Store the guess and temperature
        self.guesses.push((guess, temperature));
        
        // Check if the game is over due to max guesses
        if self.guesses.len() >= self.max_guesses {
            self.game_over = true;
        }
        
        temperature
    }
    
    // A simple method to estimate word similarity without external libraries
    fn calculate_similarity(&self, word1: &str, word2: &str) -> f64 {
        // If either word is empty, return 0
        if word1.is_empty() || word2.is_empty() {
            return 0.0;
        }
        
        // If words are identical, return 1.0
        if word1 == word2 {
            return 1.0;
        }
        
        // Check for exact prefixes
        let min_len = word1.len().min(word2.len());
        
        // Compare character by character from the start
        let mut prefix_match = 0;
        for (c1, c2) in word1.chars().zip(word2.chars()) {
            if c1 == c2 {
                prefix_match += 1;
            } else {
                break;
            }
        }
        
        let prefix_sim = prefix_match as f64 / min_len as f64;
        
        // Character set similarity
        let chars1: HashSet<char> = word1.chars().collect();
        let chars2: HashSet<char> = word2.chars().collect();
        
        let common_chars = chars1.intersection(&chars2).count();
        let total_chars = chars1.union(&chars2).count();
        
        let set_sim = common_chars as f64 / total_chars as f64;
        
        // Length similarity
        let length_ratio = if word1.len() > word2.len() {
            word2.len() as f64 / word1.len() as f64
        } else {
            word1.len() as f64 / word2.len() as f64
        };
        
        // Calculate final similarity score (weighted)
        let similarity = (prefix_sim * 0.5) + (set_sim * 0.3) + (length_ratio * 0.2);
        
        // Apply semantic hints based on the category
        let semantic_boost = if word1.contains(word2) || word2.contains(word1) {
            0.2 // Boost if one is substring of the other
        } else {
            0.0
        };
        
        (similarity + semantic_boost).min(0.95) // Cap at 0.95 to require exact match for 1.0
    }
    
    fn render_status(&self) -> String {
        let guess_count = self.guesses.len();
        let guesses_left = self.max_guesses.checked_sub(guess_count).unwrap_or(0);
        
        let status = if self.game_over {
            if self.won {
                format!("**Congratulations!** You have uncovered the state secret!")
            } else {
                format!("**GAME OVER.** The state secret was **{}**.", self.target_word)
            }
        } else {
            format!("**Attempts remaining:** {} out of {}", guesses_left, self.max_guesses)
        };
        
        format!(
            "**Category: {}**\n\
            **Difficulty: {}**\n\
            {}\n",
            self.category,
            self.difficulty.display_name(),
            status
        )
    }
    
    fn render_guesses(&self) -> String {
        if self.guesses.is_empty() {
            return String::from("No guesses yet. Enter your first guess...");
        }
        
        let mut result = String::from("**Previous Guesses:**\n\n");
        
        for (i, (guess, temp)) in self.guesses.iter().enumerate() {
            result.push_str(&format!(
                "{:2}. `{}` {} {}\n",
                i+1, 
                guess,
                temp.to_emoji(),
                temp.to_description()
            ));
        }
        
        result
    }
    
    fn get_hint(&self) -> String {
        let mut rng = rand::thread_rng();
        
        let hints = [
            format!("The word has {} letters.", self.target_word.len()),
            format!("The word starts with '{}'.", self.target_word.chars().next().unwrap()),
            format!("The word is related to {}.", self.category),
            format!("The word contains the letter '{}'.", 
                self.target_word.chars().nth(self.target_word.len() / 2).unwrap()),
        ];
        
        hints.choose(&mut rng).unwrap().clone()
    }
}

/// Word association challenge in Kremlin style
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn kremlin_secrets(
    ctx: crate::Context<'_>,
    #[description = "Difficulty level (citizen=easy, comrade=medium, commissar=hard)"] 
    difficulty: Option<String>,
) -> Result<(), CommandError> {
    // Parse difficulty
    let difficulty_level = match difficulty.as_deref().unwrap_or("comrade").to_lowercase().as_str() {
        "citizen" | "easy" => KremlinDifficulty::Citizen,
        "commissar" | "hard" => KremlinDifficulty::Commissar,
        _ => KremlinDifficulty::Comrade, // default to medium
    };
    
    // Create a new game
    let mut game = KremlinSecrets::new(difficulty_level);
    
    // Initial message
    let msg = ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ Kremlin Secrets ☭")
             .description("State Security Word Association Test")
             .color(serenity::Color::RED)
             .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/5/58/Coat_of_arms_of_the_KGB.svg/240px-Coat_of_arms_of_the_KGB.svg.png")
             .field("Instructions", 
                    "I'm thinking of a word related to Soviet history and ideology.\n\
                     Guess the word, and I'll tell you how close you are.\n\
                     The temperature indicates your proximity to the secret term.", false)
             .field("Status", game.render_status(), false)
             .field("Guesses", "No guesses yet. Enter your first guess...", false)
             .field("Hint", game.get_hint(), false)
             .footer(|f| f.text("Type your guess in chat. The KGB is watching your progress."))
        })
    }).await?;
    
    // Set up message collection
    let author = ctx.author().id;
    let channel_id = ctx.channel_id();
    
    // Process guesses
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    let mut time_left = Duration::from_secs(180); // 3 minute time limit
    let start_time = tokio::time::Instant::now();
    
    while !game.game_over && time_left.as_secs() > 0 {
        tokio::select! {
            _ = interval.tick() => {
                // Get the message ID to search after
                let message_id = match msg.message().await {
                    Ok(message) => message.id,
                    Err(_) => continue,
                };
                
                // Check for new messages that could be guesses
                if let Ok(messages) = channel_id.messages(&ctx.serenity_context().http, |retriever| {
                    retriever.after(message_id).limit(10)
                }).await {
                    for potential_guess in messages.iter() {
                        // Only process messages from the command author
                        if potential_guess.author.id == author {
                            // Process the guess
                            let guess = potential_guess.content.clone();
                            let temperature = game.make_guess(&guess);
                            
                            // Try to delete the user's message to keep the channel clean
                            let _ = potential_guess.delete(&ctx.serenity_context().http).await;
                            
                            // Update the message with the new game state
                            msg.edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title("☭ Kremlin Secrets ☭")
                                     .description("State Security Word Association Test")
                                     .color(temperature.to_color())
                                     .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/5/58/Coat_of_arms_of_the_KGB.svg/240px-Coat_of_arms_of_the_KGB.svg.png")
                                     .field("Instructions", 
                                            "I'm thinking of a word related to Soviet history and ideology.\n\
                                             Guess the word, and I'll tell you how close you are.\n\
                                             The temperature indicates your proximity to the secret term.", false)
                                     .field("Status", game.render_status(), false)
                                     .field("Guesses", game.render_guesses(), false);
                                    
                                    if !game.game_over {
                                        e.field("Hint", game.get_hint(), false);
                                    } else if game.won {
                                        e.field("Result", 
                                               format!("Correct! The word was **{}**.\nYou've proven yourself a worthy member of the Party!", 
                                                      game.target_word), false);
                                    } else {
                                        e.field("Result", 
                                               format!("The state secret was **{}**.\nBetter luck next time, comrade.", 
                                                      game.target_word), false);
                                    }
                                    
                                    e.footer(|f| {
                                        if game.game_over {
                                            f.text("Game over. Type /kremlin_secrets to play again.")
                                        } else {
                                            let remaining = time_left.as_secs();
                                            f.text(format!("Type your next guess in chat. Time remaining: {}m {}s", remaining / 60, remaining % 60))
                                        }
                                    })
                                })
                            }).await?;
                            
                            if game.game_over {
                                break;
                            }
                        }
                    }
                }
                
                // Update time left
                let elapsed = start_time.elapsed();
                if elapsed < Duration::from_secs(180) {
                    time_left = Duration::from_secs(180) - elapsed;
                } else {
                    time_left = Duration::from_secs(0);
                }
            }
        }
    }
    
    // Handle case where user didn't respond in time
    if !game.game_over {
        msg.edit(ctx, |m| {
            m.embed(|e| {
                e.title("☭ Kremlin Secrets ☭")
                 .description("Operation Terminated")
                 .color(serenity::Color::DARK_GREY)
                 .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/5/58/Coat_of_arms_of_the_KGB.svg/240px-Coat_of_arms_of_the_KGB.svg.png")
                 .field("Status", 
                        format!("Time expired. The secret word was **{}**.\nThe KGB has noted your lack of participation.", 
                               game.target_word), false)
                 .footer(|f| f.text("Game abandoned. Type /kremlin_secrets to try again."))
            })
        }).await?;
    }
    
    Ok(())
}