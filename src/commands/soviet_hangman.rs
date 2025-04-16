use crate::CommandError;
use poise::serenity_prelude as serenity;
use poise::futures_util::StreamExt;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::time::Duration;

// Soviet Hangman - Word guessing game with button interactions
struct SovietHangman {
    target_word: String,
    category: String,
    guessed_letters: HashSet<char>,
    max_attempts: usize,
    attempts_left: usize,
    game_over: bool,
    won: bool,
}

impl SovietHangman {
    fn new() -> Self {
        // Pick a random category and word
        let (category, word) = Self::random_word_and_category();
        
        SovietHangman {
            target_word: word,
            category,
            guessed_letters: HashSet::new(),
            max_attempts: 6,
            attempts_left: 6,
            game_over: false,
            won: false,
        }
    }
    
    fn random_word_and_category() -> (String, String) {
        let categories = [
            ("Soviet Leaders", vec![
                "LENIN", "STALIN", "KHRUSHCHEV", "BREZHNEV", "GORBACHEV", 
                "ANDROPOV", "CHERNENKO", "MALENKOV", "BULGANIN", "KOSYGIN"
            ]),
            ("Revolutionary Concepts", vec![
                "COMMUNISM", "SOCIALISM", "PROLETARIAT", "BOURGEOISIE", "REVOLUTION",
                "COLLECTIVIZATION", "VANGUARD", "DIALECTIC", "MATERIALISM", "CLASSLESS"
            ]),
            ("Soviet Geography", vec![
                "MOSCOW", "LENINGRAD", "STALINGRAD", "VLADIVOSTOK", "MINSK",
                "SIBERIA", "URAL", "CRIMEA", "VOLGA", "BALTIC"
            ]),
            ("Soviet Achievements", vec![
                "SPUTNIK", "VOSTOK", "INDUSTRIALIZATION", "LITERACY", "EDUCATION",
                "ELECTRIFICATION", "MIR", "COLLECTIVE", "SUBWAY", "HEALTHCARE"
            ]),
            ("Soviet Military", vec![
                "KALASHNIKOV", "RED ARMY", "NAVY", "MISSILE", "PARTISAN",
                "DEFENSE", "PARADE", "GUARDS", "VICTORY", "MEDAL"
            ])
        ];
        
        let mut rng = rand::thread_rng();
        let (category, words) = categories.choose(&mut rng).unwrap();
        let word = words.choose(&mut rng).unwrap();
        
        (category.to_string(), word.to_string())
    }
    
    fn guess_letter(&mut self, letter: char) -> bool {
        let upper_letter = letter.to_ascii_uppercase();
        
        // Check if letter was already guessed
        if self.guessed_letters.contains(&upper_letter) {
            return false;
        }
        
        // Add the letter to guessed letters
        self.guessed_letters.insert(upper_letter);
        
        // Check if the letter is in the word
        let correct = self.target_word.contains(upper_letter);
        
        // If incorrect, reduce attempts
        if !correct {
            self.attempts_left -= 1;
            if self.attempts_left == 0 {
                self.game_over = true;
            }
        } else {
            // Check if the player has won
            self.won = self.target_word.chars().all(|c| {
                c == ' ' || self.guessed_letters.contains(&c)
            });
            
            if self.won {
                self.game_over = true;
            }
        }
        
        correct
    }
    
    fn display_word(&self) -> String {
        self.target_word.chars().map(|c| {
            if c == ' ' {
                ' '
            } else if self.guessed_letters.contains(&c) {
                c
            } else {
                '_'
            }
        }).collect()
    }
    
    fn display_guessed_letters(&self) -> String {
        let alphabet = ('A'..='Z').collect::<Vec<char>>();
        let mut result = String::new();
        
        for letter in alphabet {
            if self.guessed_letters.contains(&letter) {
                if self.target_word.contains(letter) {
                    result.push_str(&format!("✅ {} ", letter));
                } else {
                    result.push_str(&format!("❌ {} ", letter));
                }
            }
        }
        
        if result.is_empty() {
            String::from("No letters guessed yet.")
        } else {
            result
        }
    }
    
    fn get_gallows(&self) -> String {
        match self.attempts_left {
            6 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │      \n\
                 │      \n\
                 │      \n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            5 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │     O\n\
                 │      \n\
                 │      \n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            4 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │     O\n\
                 │     │\n\
                 │      \n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            3 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │     O\n\
                 │    /│\n\
                 │      \n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            2 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │     O\n\
                 │    /│\\\n\
                 │      \n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            1 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │     O\n\
                 │    /│\\\n\
                 │    / \n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            0 => String::from(
                "```\n\
                 ┌─────┐\n\
                 │     │\n\
                 │     O\n\
                 │    /│\\\n\
                 │    / \\\n\
                 │      \n\
                 └──────\n\
                 ```"
            ),
            _ => String::from("Invalid state")
        }
    }
}

// Create keyboard with letters A-X and a Quit button
fn create_keyboard_buttons(game: &SovietHangman) -> Vec<serenity::CreateActionRow> {
    // Discord allows maximum 5 components per action row and max 5 action rows per message
    let letters = vec![
        vec!['A', 'B', 'C', 'D'],       // Row 1 (with Quit button)
        vec!['E', 'F', 'G', 'H'],       // Row 2
        vec!['I', 'J', 'K', 'L'],       // Row 3
        vec!['M', 'N', 'O', 'P'],       // Row 4
    ];
    
    let mut result = Vec::with_capacity(5);
    
    // First row with quit button plus letters
    let mut first_row = serenity::CreateActionRow::default();
    
    // Add quit button
    first_row.create_button(|b| {
        b.style(serenity::ButtonStyle::Primary)
         .custom_id("soviet_hangman_quit")
         .label("Quit")
    });
    
    // Add the letters from the first group
    for &letter in &letters[0] {
        let already_guessed = game.guessed_letters.contains(&letter);
        first_row.create_button(|b| {
            b.style(if already_guessed {
                serenity::ButtonStyle::Secondary
            } else {
                serenity::ButtonStyle::Success
            })
             .custom_id(format!("soviet_hangman_{}", letter))
             .label(letter.to_string())
             .disabled(already_guessed)
        });
    }
    result.push(first_row);
    
    // Create the other 4 rows
    for letter_row in letters.iter().skip(1) {
        let mut action_row = serenity::CreateActionRow::default();
        for &letter in letter_row {
            let already_guessed = game.guessed_letters.contains(&letter);
            action_row.create_button(|b| {
                b.style(if already_guessed {
                    serenity::ButtonStyle::Secondary
                } else {
                    serenity::ButtonStyle::Success
                })
                 .custom_id(format!("soviet_hangman_{}", letter))
                 .label(letter.to_string())
                 .disabled(already_guessed)
            });
        }
        result.push(action_row);
    }
    
    result
}

// Create keyboard with the remaining letters Y-Z and a navigation button
fn create_second_keyboard(game: &SovietHangman) -> Vec<serenity::CreateActionRow> {
    // For the last two letters Y and Z
    let mut result = Vec::with_capacity(1);
    let mut row = serenity::CreateActionRow::default();
    
    // Add page navigation button
    row.create_button(|b| {
        b.style(serenity::ButtonStyle::Primary)
         .custom_id("soviet_hangman_page1")
         .label("Page 1 (A-X)")
    });
    
    // Add Y and Z buttons
    for &letter in &['Y', 'Z'] {
        let already_guessed = game.guessed_letters.contains(&letter);
        row.create_button(|b| {
            b.style(if already_guessed {
                serenity::ButtonStyle::Secondary
            } else {
                serenity::ButtonStyle::Success
            })
             .custom_id(format!("soviet_hangman_{}", letter))
             .label(letter.to_string())
             .disabled(already_guessed)
        });
    }
    
    result.push(row);
    result
}

/// Play Soviet-themed Hangman
#[poise::command(slash_command, prefix_command)]
pub async fn soviet_hangman(
    ctx: crate::Context<'_>,
) -> Result<(), CommandError> {
    let mut game = SovietHangman::new();
    
    // Initial message with keyboard (page 1: A-X)
    let mut show_page_2 = false; // Track which page we're showing
    
    let msg = ctx.send(|m| {
        m.embed(|e| {
            e.title("☭ Comrade Hangman ☭")
             .description("State-Approved Word Guessing - Page 1 (A-P)")
             .color(serenity::Color::RED)
             .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
             .field("Category", &game.category, true)
             .field("Attempts", format!("{}/{}", game.attempts_left, game.max_attempts), true)
             .field("Word", format!("```{}```", game.display_word()), false)
             .field("Gallows", game.get_gallows(), false)
             .field("Guessed Letters", game.display_guessed_letters(), false)
             .footer(|f| f.text("Select a letter on the keyboard. Use page navigation for Q-Z."))
        })
        .components(|c| {
            // Add a page 2 navigation button
            let mut page_row = serenity::CreateActionRow::default();
            page_row.create_button(|b| {
                b.style(serenity::ButtonStyle::Primary)
                 .custom_id("soviet_hangman_page2")
                 .label("Page 2 (Y-Z)")
            });
            c.add_action_row(page_row);
            
            // Add actual letter buttons (just the first 4 rows)
            for row in create_keyboard_buttons(&game).into_iter().take(4) {
                c.add_action_row(row);
            }
            c
        })
    }).await?;
    
    // Button interaction loop
    let author_id = ctx.author().id;
    
    // Create a collector for button interactions
    let mut collector = msg.message().await?.await_component_interactions(ctx)
        .timeout(Duration::from_secs(180))
        .filter(move |press| press.user.id == author_id)
        .build();
    
    // Process interactions
    while let Some(press) = collector.next().await {
        let custom_id = &press.data.custom_id;
        
        // Handle quit
        if custom_id == "soviet_hangman_quit" {
            press.create_interaction_response(ctx, |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                 .interaction_response_data(|d| {
                     d.embed(|e| {
                         e.title("☭ Comrade Hangman ☭")
                          .description("Game Terminated by User")
                          .color(serenity::Color::DARK_GREY)
                          .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
                          .field("Word", &game.target_word, false)
                          .footer(|f| f.text("Your defection has been noted in your permanent record."))
                     })
                     .components(|c| c) // Clear components
                 })
            }).await?;
            
            break;
        }
        
        // Handle page switching - to page 1
        if custom_id == "soviet_hangman_page1" {
            show_page_2 = false;
            press.create_interaction_response(ctx, |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                 .interaction_response_data(|d| {
                     d.embed(|e| {
                         e.title("☭ Comrade Hangman ☭")
                          .description("State-Approved Word Guessing - Page 1 (A-P)")
                          .color(serenity::Color::RED)
                          .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
                          .field("Category", &game.category, true)
                          .field("Attempts", format!("{}/{}", game.attempts_left, game.max_attempts), true)
                          .field("Word", format!("```{}```", game.display_word()), false)
                          .field("Gallows", game.get_gallows(), false)
                          .field("Guessed Letters", game.display_guessed_letters(), false)
                          .footer(|f| f.text("Select a letter on the keyboard. Use page navigation for Q-Z."))
                     })
                     .components(|c| {
                        // Add a page 2 navigation button
                        let mut page_row = serenity::CreateActionRow::default();
                        page_row.create_button(|b| {
                            b.style(serenity::ButtonStyle::Primary)
                             .custom_id("soviet_hangman_page2")
                             .label("Page 2 (Y-Z)")
                        });
                        c.add_action_row(page_row);
                        
                        // Add actual letter buttons (just the first 4 rows)
                        for row in create_keyboard_buttons(&game).into_iter().take(4) {
                            c.add_action_row(row);
                        }
                        c
                     })
                 })
            }).await?;
            continue;
        }
        
        // Handle page switching - to page 2
        if custom_id == "soviet_hangman_page2" {
            show_page_2 = true;
            press.create_interaction_response(ctx, |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                 .interaction_response_data(|d| {
                     d.embed(|e| {
                         e.title("☭ Comrade Hangman ☭")
                          .description("State-Approved Word Guessing - Page 2 (Q-Z)")
                          .color(serenity::Color::RED)
                          .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
                          .field("Category", &game.category, true)
                          .field("Attempts", format!("{}/{}", game.attempts_left, game.max_attempts), true)
                          .field("Word", format!("```{}```", game.display_word()), false)
                          .field("Gallows", game.get_gallows(), false)
                          .field("Guessed Letters", game.display_guessed_letters(), false)
                          .footer(|f| f.text("Select a letter on the keyboard. Use page navigation to return to A-P."))
                     })
                     .components(|c| {
                         // Page 1 navigation button
                         let mut page_row = serenity::CreateActionRow::default();
                         page_row.create_button(|b| {
                             b.style(serenity::ButtonStyle::Primary)
                              .custom_id("soviet_hangman_page1")
                              .label("Page 1 (A-P)")
                         });
                         c.add_action_row(page_row);
                         
                         // Second half of alphabet (Q-Z)
                         // Row with Q R S T
                         let mut row1 = serenity::CreateActionRow::default();
                         for &letter in &['Q', 'R', 'S', 'T'] {
                             let already_guessed = game.guessed_letters.contains(&letter);
                             row1.create_button(|b| {
                                 b.style(if already_guessed {
                                     serenity::ButtonStyle::Secondary
                                 } else {
                                     serenity::ButtonStyle::Success
                                 })
                                  .custom_id(format!("soviet_hangman_{}", letter))
                                  .label(letter.to_string())
                                  .disabled(already_guessed)
                             });
                         }
                         c.add_action_row(row1);
                         
                         // Row with U V W X
                         let mut row2 = serenity::CreateActionRow::default();
                         for &letter in &['U', 'V', 'W', 'X'] {
                             let already_guessed = game.guessed_letters.contains(&letter);
                             row2.create_button(|b| {
                                 b.style(if already_guessed {
                                     serenity::ButtonStyle::Secondary
                                 } else {
                                     serenity::ButtonStyle::Success
                                 })
                                  .custom_id(format!("soviet_hangman_{}", letter))
                                  .label(letter.to_string())
                                  .disabled(already_guessed)
                             });
                         }
                         c.add_action_row(row2);
                         
                         // Row with Y Z
                         let mut row3 = serenity::CreateActionRow::default();
                         for &letter in &['Y', 'Z'] {
                             let already_guessed = game.guessed_letters.contains(&letter);
                             row3.create_button(|b| {
                                 b.style(if already_guessed {
                                     serenity::ButtonStyle::Secondary
                                 } else {
                                     serenity::ButtonStyle::Success
                                 })
                                  .custom_id(format!("soviet_hangman_{}", letter))
                                  .label(letter.to_string())
                                  .disabled(already_guessed)
                             });
                         }
                         c.add_action_row(row3);
                         
                         c
                     })
                 })
            }).await?;
            continue;
        }
        
        // Process a letter guess
        if custom_id.starts_with("soviet_hangman_") && custom_id.len() == 16 {
            let letter = custom_id.chars().last().unwrap();
            let correct = game.guess_letter(letter);
            
            // Update the message
            press.create_interaction_response(ctx, |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                 .interaction_response_data(|d| {
                     d.embed(|e| {
                         e.title("☭ Comrade Hangman ☭")
                          .description(if correct {
                              "Correct! The letter is present in the word."
                          } else {
                              "Incorrect! You lose an attempt."
                          })
                          .color(if game.game_over {
                              if game.won {
                                  serenity::Color::DARK_GREEN
                              } else {
                                  serenity::Color::DARK_RED
                              }
                          } else {
                              serenity::Color::RED
                          })
                          .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
                          .field("Category", &game.category, true)
                          .field("Attempts", format!("{}/{}", game.attempts_left, game.max_attempts), true)
                          .field("Word", format!("```{}```", game.display_word()), false)
                          .field("Gallows", game.get_gallows(), false)
                          .field("Guessed Letters", game.display_guessed_letters(), false);
                          
                         if game.game_over {
                             if game.won {
                                 e.field("Result", "Victory! The state commends your lexical knowledge!", false);
                             } else {
                                 e.field("Result", format!("Failure! The correct word was **{}**", game.target_word), false);
                             }
                             e.footer(|f| f.text("Game over. Use /soviet_hangman to play again."));
                         } else {
                             if show_page_2 {
                                 e.footer(|f| f.text("Select a letter on the keyboard. Use page navigation to return to A-P."));
                             } else {
                                 e.footer(|f| f.text("Select a letter on the keyboard. Use page navigation for Q-Z."));
                             }
                         }
                         
                         e
                     });
                     
                     // Only keep buttons if game is still active
                     if !game.game_over {
                         d.components(|c| {
                             if show_page_2 {
                                 // Page 1 navigation button
                                 let mut page_row = serenity::CreateActionRow::default();
                                 page_row.create_button(|b| {
                                     b.style(serenity::ButtonStyle::Primary)
                                      .custom_id("soviet_hangman_page1")
                                      .label("Page 1 (A-P)")
                                 });
                                 c.add_action_row(page_row);
                                 
                                 // Second half of alphabet (Q-Z)
                                 // Row with Q R S T
                                 let mut row1 = serenity::CreateActionRow::default();
                                 for &letter in &['Q', 'R', 'S', 'T'] {
                                     let already_guessed = game.guessed_letters.contains(&letter);
                                     row1.create_button(|b| {
                                         b.style(if already_guessed {
                                             serenity::ButtonStyle::Secondary
                                         } else {
                                             serenity::ButtonStyle::Success
                                         })
                                          .custom_id(format!("soviet_hangman_{}", letter))
                                          .label(letter.to_string())
                                          .disabled(already_guessed)
                                     });
                                 }
                                 c.add_action_row(row1);
                                 
                                 // Row with U V W X
                                 let mut row2 = serenity::CreateActionRow::default();
                                 for &letter in &['U', 'V', 'W', 'X'] {
                                     let already_guessed = game.guessed_letters.contains(&letter);
                                     row2.create_button(|b| {
                                         b.style(if already_guessed {
                                             serenity::ButtonStyle::Secondary
                                         } else {
                                             serenity::ButtonStyle::Success
                                         })
                                          .custom_id(format!("soviet_hangman_{}", letter))
                                          .label(letter.to_string())
                                          .disabled(already_guessed)
                                     });
                                 }
                                 c.add_action_row(row2);
                                 
                                 // Row with Y Z
                                 let mut row3 = serenity::CreateActionRow::default();
                                 for &letter in &['Y', 'Z'] {
                                     let already_guessed = game.guessed_letters.contains(&letter);
                                     row3.create_button(|b| {
                                         b.style(if already_guessed {
                                             serenity::ButtonStyle::Secondary
                                         } else {
                                             serenity::ButtonStyle::Success
                                         })
                                          .custom_id(format!("soviet_hangman_{}", letter))
                                          .label(letter.to_string())
                                          .disabled(already_guessed)
                                     });
                                 }
                                 c.add_action_row(row3);
                             } else {
                                 // Page 2 navigation button
                                 let mut page_row = serenity::CreateActionRow::default();
                                 page_row.create_button(|b| {
                                     b.style(serenity::ButtonStyle::Primary)
                                      .custom_id("soviet_hangman_page2")
                                      .label("Page 2 (Q-Z)")
                                 });
                                 c.add_action_row(page_row);
                                 
                                 // Letter buttons A-P
                                 for row in create_keyboard_buttons(&game) {
                                     c.add_action_row(row);
                                 }
                             }
                             c
                         });
                     }
                     
                     d
                 })
            }).await?;
            
            if game.game_over {
                break;
            }
        }
    }
    
    // If we exit the loop without the game being over, the collector timed out
    if !game.game_over {
        msg.edit(ctx, |m| {
            m.embed(|e| {
                e.title("☭ Comrade Hangman ☭")
                 .description("Session Expired")
                 .color(serenity::Color::DARK_GREY)
                 .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/Soviet_Union_state_emblem.svg/240px-Soviet_Union_state_emblem.svg.png")
                 .field("Word", &game.target_word, false)
                 .footer(|f| f.text("Your inactivity has been reported to the authorities."))
            })
            .components(|c| c) // Clear components
        }).await?;
    }
    
    Ok(())
}