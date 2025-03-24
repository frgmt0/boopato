use rusqlite::params;
use tokio_rusqlite::Connection as AsyncConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::Path;

// Define our own error type to simplify error handling
pub type DbResult<T> = std::result::Result<T, tokio_rusqlite::Error>;

// Job types for the user
#[derive(Debug, Clone, PartialEq)]
pub enum JobType {
    Miner,
    Farmer,
    Programmer,
    Teacher,
    Doctor,
    None,
}

impl JobType {
    pub fn from_string(s: &str) -> Self {
        match s {
            "miner" => Self::Miner,
            "farmer" => Self::Farmer,
            "programmer" => Self::Programmer,
            "teacher" => Self::Teacher,
            "doctor" => Self::Doctor,
            _ => Self::None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Self::Miner => "miner".to_string(),
            Self::Farmer => "farmer".to_string(),
            Self::Programmer => "programmer".to_string(),
            Self::Teacher => "teacher".to_string(),
            Self::Doctor => "doctor".to_string(),
            Self::None => "none".to_string(),
        }
    }
    
    pub fn get_description(&self) -> String {
        match self {
            Self::Miner => "Extract precious resources from the depths for the motherland".to_string(),
            Self::Farmer => "Grow crops to feed your fellow comrades".to_string(),
            Self::Programmer => "Develop software for the glory of the collective".to_string(),
            Self::Teacher => "Educate the youth in the ways of our society".to_string(),
            Self::Doctor => "Heal the sick and care for the injured workers".to_string(),
            Self::None => "Unemployed and bringing shame to your comrades".to_string(),
        }
    }
    
    pub fn get_boops_multiplier(&self) -> f64 {
        match self {
            Self::Miner => 1.5,
            Self::Farmer => 1.2,
            Self::Programmer => 1.8,
            Self::Teacher => 1.3,
            Self::Doctor => 2.0,
            Self::None => 1.0,
        }
    }
    
    pub fn list_all() -> Vec<Self> {
        vec![
            Self::Miner,
            Self::Farmer,
            Self::Programmer,
            Self::Teacher,
            Self::Doctor,
        ]
    }
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<AsyncConnection>>,
}

#[allow(dead_code)]
impl Database {
    pub async fn new(db_path: &str) -> DbResult<Self> {
        let is_new_db = !Path::new(db_path).exists();
        
        // Initialize the database
        let conn = AsyncConnection::open(db_path).await?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        // Initialize or verify tables
        db.init_tables().await?;
        
        // If this is a new database, perform additional setup
        if is_new_db {
            println!("Creating new database at {}", db_path);
        } else {
            // Verify and upgrade schema if needed
            db.verify_schema().await?;
        }
        
        Ok(db)
    }
    
    async fn init_tables(&self) -> DbResult<()> {
        let conn = self.conn.lock().await;
        
        conn.call(|conn| {
            // Create users table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    user_id TEXT PRIMARY KEY,
                    server_id TEXT NOT NULL,
                    username TEXT NOT NULL,
                    boops REAL DEFAULT 0.0,
                    messages_count INTEGER DEFAULT 0,
                    last_work TIMESTAMP,
                    last_commit TIMESTAMP,
                    last_leader TIMESTAMP,
                    job TEXT DEFAULT 'none',
                    job_level INTEGER DEFAULT 1
                )",
                [],
            )?;
            
            // Create server table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS servers (
                    server_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    communal_boops REAL DEFAULT 0.0,
                    current_distribution_round INTEGER DEFAULT 1
                )",
                [],
            )?;
            
            // Create distribution claims table to track who has claimed in each round
            conn.execute(
                "CREATE TABLE IF NOT EXISTS distribution_claims (
                    user_id TEXT NOT NULL,
                    server_id TEXT NOT NULL,
                    distribution_round INTEGER NOT NULL,
                    claimed_at TIMESTAMP NOT NULL,
                    PRIMARY KEY (user_id, server_id, distribution_round)
                )",
                [],
            )?;
            
            // Create game_scores table if it doesn't exist
            conn.execute(
                "CREATE TABLE IF NOT EXISTS game_scores (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id TEXT NOT NULL,
                    server_id TEXT NOT NULL,
                    username TEXT NOT NULL,
                    game_type TEXT NOT NULL,
                    score REAL NOT NULL,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    async fn verify_schema(&self) -> DbResult<()> {
        let conn = self.conn.lock().await;
        
        conn.call(|conn| {
            // Check if last_leader column exists in users table
            let has_last_leader = match conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name='last_leader'",
                [],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(count) => count > 0,
                Err(_) => false,
            };
            
            if !has_last_leader {
                // Add last_leader column if it doesn't exist
                conn.execute("ALTER TABLE users ADD COLUMN last_leader TIMESTAMP", [])?;
            }
            
            // Check if job column exists in users table
            let has_job = match conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name='job'",
                [],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(count) => count > 0,
                Err(_) => false,
            };
            
            if !has_job {
                // Add job column if it doesn't exist
                conn.execute("ALTER TABLE users ADD COLUMN job TEXT DEFAULT 'none'", [])?;
            }
            
            // Check if job_level column exists
            let has_job_level = match conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name='job_level'",
                [],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(count) => count > 0,
                Err(_) => false,
            };
            
            if !has_job_level {
                // Add job_level column if it doesn't exist
                conn.execute("ALTER TABLE users ADD COLUMN job_level INTEGER DEFAULT 1", [])?;
            }
            
            // Check if boops column is REAL type
            let boops_type: String = match conn.query_row(
                "SELECT type FROM pragma_table_info('users') WHERE name='boops'",
                [],
                |row| row.get(0)
            ) {
                Ok(t) => t,
                Err(_) => "INTEGER".to_string(),
            };
            
            // If boops is not REAL, create a backup and update the schema
            if boops_type != "REAL" {
                // Back up the original data
                conn.execute("CREATE TABLE users_backup AS SELECT * FROM users", [])?;
                
                // Drop the original table and recreate with REAL type
                conn.execute("DROP TABLE users", [])?;
                conn.execute(
                    "CREATE TABLE users (
                        user_id TEXT PRIMARY KEY,
                        server_id TEXT NOT NULL,
                        username TEXT NOT NULL,
                        boops REAL DEFAULT 0.0,
                        messages_count INTEGER DEFAULT 0,
                        last_work TIMESTAMP,
                        last_commit TIMESTAMP,
                        last_leader TIMESTAMP,
                        job TEXT DEFAULT 'none',
                        job_level INTEGER DEFAULT 1
                    )",
                    [],
                )?;
                
                // Restore data, converting INTEGER to REAL for boops
                conn.execute(
                    "INSERT INTO users SELECT 
                        user_id, server_id, username, 
                        CAST(boops AS REAL) AS boops, 
                        messages_count, last_work, last_commit, last_leader, 
                        job, job_level 
                    FROM users_backup",
                    [],
                )?;
                
                // Drop backup table
                conn.execute("DROP TABLE users_backup", [])?;
                
                println!("Updated boops column to REAL type");
            }
            
            // Do the same for communal_boops in servers table
            let communal_boops_type: String = match conn.query_row(
                "SELECT type FROM pragma_table_info('servers') WHERE name='communal_boops'",
                [],
                |row| row.get(0)
            ) {
                Ok(t) => t,
                Err(_) => "INTEGER".to_string(),
            };
            
            if communal_boops_type != "REAL" {
                // Back up the original data
                conn.execute("CREATE TABLE servers_backup AS SELECT * FROM servers", [])?;
                
                // Drop the original table and recreate with REAL type
                conn.execute("DROP TABLE servers", [])?;
                conn.execute(
                    "CREATE TABLE servers (
                        server_id TEXT PRIMARY KEY,
                        name TEXT NOT NULL,
                        communal_boops REAL DEFAULT 0.0
                    )",
                    [],
                )?;
                
                // Restore data, converting INTEGER to REAL for communal_boops
                conn.execute(
                    "INSERT INTO servers SELECT 
                        server_id, name, 
                        CAST(communal_boops AS REAL) AS communal_boops
                    FROM servers_backup",
                    [],
                )?;
                
                // Drop backup table
                conn.execute("DROP TABLE servers_backup", [])?;
                
                println!("Updated communal_boops column to REAL type");
            }
            
            // Check if current_distribution_round column exists in servers table
            let has_distribution_round = match conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('servers') WHERE name='current_distribution_round'",
                [],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(count) => count > 0,
                Err(_) => false,
            };
            
            if !has_distribution_round {
                // Add current_distribution_round column if it doesn't exist
                conn.execute("ALTER TABLE servers ADD COLUMN current_distribution_round INTEGER DEFAULT 1", [])?;
                println!("Added current_distribution_round column to servers table");
            }
            
            // Check if distribution_claims table exists
            let has_claims_table = match conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='distribution_claims'",
                [],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(count) => count > 0,
                Err(_) => false,
            };
            
            if !has_claims_table {
                // Create distribution claims table
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS distribution_claims (
                        user_id TEXT NOT NULL,
                        server_id TEXT NOT NULL,
                        distribution_round INTEGER NOT NULL,
                        claimed_at TIMESTAMP NOT NULL,
                        PRIMARY KEY (user_id, server_id, distribution_round)
                    )",
                    [],
                )?;
                println!("Created distribution_claims table");
            }
            
            Ok::<_, rusqlite::Error>(())
        }).await
    }

    pub async fn add_user(&self, user_id: &str, server_id: &str, username: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let server_id = server_id.to_string();
        let username = username.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "INSERT OR IGNORE INTO users (user_id, server_id, username) VALUES (?, ?, ?)",
                params![user_id, server_id, username],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn add_server(&self, server_id: &str, name: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        let name = name.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "INSERT OR IGNORE INTO servers (server_id, name) VALUES (?, ?)",
                params![server_id, name],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_user_boops(&self, user_id: &str) -> DbResult<f64> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT boops FROM users WHERE user_id = ?")?;
            let boops = stmt.query_row(params![user_id], |row| row.get(0))?;
            Ok::<f64, rusqlite::Error>(boops)
        }).await
    }
    
    pub async fn update_user_boops(&self, user_id: &str, boops: f64) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET boops = ? WHERE user_id = ?",
                params![boops, user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_communal_boops(&self, server_id: &str) -> DbResult<f64> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT communal_boops FROM servers WHERE server_id = ?")?;
            let boops = stmt.query_row(params![server_id], |row| row.get(0))?;
            Ok::<f64, rusqlite::Error>(boops)
        }).await
    }
    
    pub async fn update_communal_boops(&self, server_id: &str, boops: f64) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE servers SET communal_boops = ? WHERE server_id = ?",
                params![boops, server_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn add_message_count(&self, user_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET messages_count = messages_count + 1 WHERE user_id = ?",
                params![user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_top_talkers(&self, server_id: &str, limit: u32) -> DbResult<Vec<(String, String, i64)>> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT user_id, username, messages_count FROM users 
                 WHERE server_id = ? ORDER BY messages_count DESC LIMIT ?"
            )?;
            
            let rows = stmt.query_map(params![server_id, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;
            
            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            
            Ok::<Vec<(String, String, i64)>, rusqlite::Error>(result)
        }).await
    }
    
    pub async fn get_top_contributors(&self, server_id: &str, limit: u32) -> DbResult<Vec<(String, String, f64)>> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT user_id, username, boops FROM users 
                 WHERE server_id = ? ORDER BY boops DESC LIMIT ?"
            )?;
            
            let rows = stmt.query_map(params![server_id, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            })?;
            
            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            
            Ok::<Vec<(String, String, f64)>, rusqlite::Error>(result)
        }).await
    }
    
    pub async fn update_last_work(&self, user_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let now = chrono::Utc::now().timestamp();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET last_work = ? WHERE user_id = ?",
                params![now, user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_last_work(&self, user_id: &str) -> DbResult<Option<i64>> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            let result = conn.query_row(
                "SELECT COALESCE(last_work, 0) FROM users WHERE user_id = ?", 
                params![user_id], 
                |row| row.get(0)
            );
            
            match result {
                Ok(timestamp) => Ok(Some(timestamp)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
    }
    
    pub async fn update_last_commit(&self, user_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let now = chrono::Utc::now().timestamp();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET last_commit = ? WHERE user_id = ?",
                params![now, user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_last_commit(&self, user_id: &str) -> DbResult<Option<i64>> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            let result = conn.query_row(
                "SELECT COALESCE(last_commit, 0) FROM users WHERE user_id = ?", 
                params![user_id], 
                |row| row.get(0)
            );
            
            match result {
                Ok(timestamp) => Ok(Some(timestamp)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
    }
    
    pub async fn distribute_boops(&self, server_id: &str, amount: f64) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Start a transaction
            conn.execute("BEGIN TRANSACTION", [])?;
            
            // Update communal boops first
            conn.execute(
                "UPDATE servers SET communal_boops = communal_boops + ? WHERE server_id = ?",
                params![amount, server_id],
            )?;
            
            // Increment the distribution round to start a new round
            // This ensures everyone gets a fair chance to claim from the new boops
            conn.execute(
                "UPDATE servers SET current_distribution_round = current_distribution_round + 1 WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Get the new round number for logging
            let new_round: i64 = conn.query_row(
                "SELECT current_distribution_round FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Commit the transaction
            conn.execute("COMMIT", [])?;
            
            println!("Added {:.2} boops to communal pool for server {} and started new distribution round #{}", 
                    amount, server_id, new_round);
            
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_last_leader(&self, user_id: &str) -> DbResult<Option<i64>> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            let result = conn.query_row(
                "SELECT COALESCE(last_leader, 0) FROM users WHERE user_id = ?", 
                params![user_id], 
                |row| row.get(0)
            );
            
            match result {
                Ok(timestamp) => Ok(Some(timestamp)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
    }
    
    pub async fn update_last_leader(&self, user_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let now = chrono::Utc::now().timestamp();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET last_leader = ? WHERE user_id = ?",
                params![now, user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_user_job(&self, user_id: &str) -> DbResult<String> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT job FROM users WHERE user_id = ?")?;
            let job = stmt.query_row(params![user_id], |row| row.get(0))?;
            Ok::<String, rusqlite::Error>(job)
        }).await
    }
    
    pub async fn set_user_job(&self, user_id: &str, job: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let job = job.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET job = ? WHERE user_id = ?",
                params![job, user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_job_level(&self, user_id: &str) -> DbResult<i64> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT job_level FROM users WHERE user_id = ?")?;
            let level = stmt.query_row(params![user_id], |row| row.get(0))?;
            Ok::<i64, rusqlite::Error>(level)
        }).await
    }
    
    pub async fn increment_job_level(&self, user_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET job_level = job_level + 1 WHERE user_id = ?",
                params![user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn add_user_boops(&self, user_id: &str, amount: f64) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "UPDATE users SET boops = boops + ? WHERE user_id = ?",
                params![amount, user_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await
    }

    pub async fn ensure_user_exists(&self, user_id: &str, server_id: &str, username: &str) -> DbResult<bool> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let server_id = server_id.to_string();
        let username = username.to_string();
        
        conn.call(move |conn| {
            // Check if user exists
            let user_exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE user_id = ?",
                params![user_id],
                |row| row.get(0)
            ).unwrap_or(0);
            
            let user_already_existed = user_exists > 0;
            
            if !user_already_existed {
                // Insert the user if they don't exist
                conn.execute(
                    "INSERT INTO users (user_id, server_id, username, boops, messages_count, job, job_level) VALUES (?, ?, ?, 0, 0, 'none', 1)",
                    params![user_id, server_id, username],
                )?;
                
                println!("Added new user to database: {}", username);
            }
            
            // Also ensure server exists
            let server_exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0)
            ).unwrap_or(0);
            
            if server_exists == 0 {
                // Insert the server if it doesn't exist
                conn.execute(
                    "INSERT INTO servers (server_id, name, communal_boops) VALUES (?, ?, 0)",
                    params![server_id, "Server"],
                )?;
                
                println!("Added new server to database: {}", server_id);
            }
            
            Ok::<bool, rusqlite::Error>(user_already_existed)
        }).await
    }

    pub async fn clear_user_cooldowns(&self, user_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        
        conn.call(move |conn| {
            // Set all cooldown timestamps to NULL
            conn.execute(
                "UPDATE users SET last_work = NULL, last_commit = NULL, last_leader = NULL WHERE user_id = ?",
                params![user_id],
            )?;
            
            println!("Cleared cooldowns for user: {}", user_id);
            Ok::<_, rusqlite::Error>(())
        }).await
    }

    pub async fn get_all_users(&self, server_id: &str) -> DbResult<Vec<(String, String, f64)>> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT user_id, username, boops FROM users WHERE server_id = ? ORDER BY boops DESC")?;
            let rows = stmt.query_map(params![server_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            })?;
            
            let mut users = Vec::new();
            for user in rows {
                users.push(user?);
            }
            
            Ok::<Vec<(String, String, f64)>, rusqlite::Error>(users)
        }).await
    }

    pub async fn reset_server_data(&self, server_id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Start a transaction to ensure atomicity
            conn.execute("BEGIN TRANSACTION", [])?;
            
            // First, delete all user data for the server
            let deleted_users = conn.execute(
                "DELETE FROM users WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Reset the server's communal boops
            conn.execute(
                "UPDATE servers SET communal_boops = 0.0 WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Commit the transaction
            conn.execute("COMMIT", [])?;
            
            println!("Reset server data for {}: {} users removed", server_id, deleted_users);
            Ok::<_, rusqlite::Error>(())
        }).await
    }

    pub async fn get_server_user_count(&self, server_id: &str) -> DbResult<i64> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Count all non-bot users in the server
            let user_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            Ok::<i64, rusqlite::Error>(user_count)
        }).await
    }

    pub async fn claim_boops(&self, user_id: &str, server_id: &str) -> DbResult<f64> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Start a transaction
            conn.execute("BEGIN TRANSACTION", [])?;
            
            // Get current distribution round
            let current_round: i64 = conn.query_row(
                "SELECT current_distribution_round FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Check if user has already claimed in this round
            let already_claimed: i64 = conn.query_row(
                "SELECT COUNT(*) FROM distribution_claims 
                 WHERE user_id = ? AND server_id = ? AND distribution_round = ?",
                params![user_id, server_id, current_round],
                |row| row.get(0),
            )?;
            
            if already_claimed > 0 {
                // User already claimed in this round
                conn.execute("ROLLBACK", [])?;
                return Ok(0.0);
            }
            
            // Get current communal boops
            let communal_boops: f64 = conn.query_row(
                "SELECT communal_boops FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Get ALL users in the server 
            let total_users: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Get number of users who have already claimed in this round
            let claimed_users: i64 = conn.query_row(
                "SELECT COUNT(*) FROM distribution_claims 
                 WHERE server_id = ? AND distribution_round = ?",
                params![server_id, current_round],
                |row| row.get(0),
            )?;
            
            // Calculate remaining eligible users
            let remaining_users = total_users - claimed_users;
            
            if remaining_users <= 0 || communal_boops <= 0.0 {
                // No more eligible users or no communal boops
                conn.execute("ROLLBACK", [])?;
                return Ok(0.0);
            }
            
            // Calculate share based on total_users (not remaining_users)
            // This ensures each user gets the same amount regardless of when they claim
            let share_per_user = ((communal_boops / total_users as f64) * 100.0).round() / 100.0;
            
            if share_per_user <= 0.0 {
                // Share too small
                conn.execute("ROLLBACK", [])?;
                return Ok(0.0);
            }
            
            // Add share to user's boops
            conn.execute(
                "UPDATE users SET boops = boops + ? WHERE user_id = ?",
                params![share_per_user, user_id],
            )?;
            
            // Remove claimed amount from communal pool
            conn.execute(
                "UPDATE servers SET communal_boops = communal_boops - ? WHERE server_id = ?",
                params![share_per_user, server_id],
            )?;
            
            // Record that this user has claimed in this round
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "INSERT INTO distribution_claims (user_id, server_id, distribution_round, claimed_at)
                 VALUES (?, ?, ?, ?)",
                params![user_id, server_id, current_round, now],
            )?;
            
            // Check if all users have claimed in this round
            let total_claimed: i64 = conn.query_row(
                "SELECT COUNT(*) FROM distribution_claims 
                 WHERE server_id = ? AND distribution_round = ?",
                params![server_id, current_round],
                |row| row.get(0),
            )?;
            
            // If all users have claimed, advance to the next distribution round
            if total_claimed >= total_users {
                conn.execute(
                    "UPDATE servers SET current_distribution_round = current_distribution_round + 1 
                     WHERE server_id = ?",
                    params![server_id],
                )?;
                println!("All users claimed in round {}. Advancing to next round.", current_round);
            }
            
            // Commit transaction
            conn.execute("COMMIT", [])?;
            
            println!("User {} claimed {:.2} boops from communal pool in round {}", user_id, share_per_user, current_round);
            Ok::<f64, rusqlite::Error>(share_per_user)
        }).await
    }

    // New method to get distribution round status
    pub async fn get_distribution_status(&self, server_id: &str) -> DbResult<(i64, i64, i64)> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Get current round
            let current_round: i64 = conn.query_row(
                "SELECT current_distribution_round FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Get total users
            let total_users: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Get claimed users
            let claimed_users: i64 = conn.query_row(
                "SELECT COUNT(*) FROM distribution_claims 
                 WHERE server_id = ? AND distribution_round = ?",
                params![server_id, current_round],
                |row| row.get(0),
            )?;
            
            Ok::<(i64, i64, i64), rusqlite::Error>((current_round, claimed_users, total_users))
        }).await
    }

    // Method to check if a user has claimed in the current round
    pub async fn has_claimed_current_round(&self, user_id: &str, server_id: &str) -> DbResult<bool> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Get current round
            let current_round: i64 = conn.query_row(
                "SELECT current_distribution_round FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Check if user has claimed
            let claimed: i64 = conn.query_row(
                "SELECT COUNT(*) FROM distribution_claims 
                 WHERE user_id = ? AND server_id = ? AND distribution_round = ?",
                params![user_id, server_id, current_round],
                |row| row.get(0),
            )?;
            
            Ok::<bool, rusqlite::Error>(claimed > 0)
        }).await
    }

    // Method for admins to force a new distribution round
    pub async fn start_new_distribution_round(&self, server_id: &str) -> DbResult<i64> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Increment the distribution round
            conn.execute(
                "UPDATE servers SET current_distribution_round = current_distribution_round + 1 
                 WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Get the new round number
            let new_round: i64 = conn.query_row(
                "SELECT current_distribution_round FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            println!("Started new distribution round {} for server {}", new_round, server_id);
            Ok::<i64, rusqlite::Error>(new_round)
        }).await
    }

    // Method for admins to distribute communal boops to all users directly
    pub async fn distribute_to_all_users(&self, server_id: &str) -> DbResult<(i64, f64)> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        
        conn.call(move |conn| {
            // Start a transaction
            conn.execute("BEGIN TRANSACTION", [])?;
            
            // Get current communal boops
            let communal_boops: f64 = conn.query_row(
                "SELECT communal_boops FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            if communal_boops <= 0.0 {
                // No boops to distribute
                conn.execute("ROLLBACK", [])?;
                return Ok((0, 0.0));
            }
            
            // Get all users in the server
            let total_users: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            if total_users <= 0 {
                // No users to distribute to
                conn.execute("ROLLBACK", [])?;
                return Ok((0, 0.0));
            }
            
            // Calculate share per user (rounded to 2 decimal places)
            let share_per_user = ((communal_boops / total_users as f64) * 100.0).round() / 100.0;
            
            if share_per_user <= 0.0 {
                // Share too small to distribute
                conn.execute("ROLLBACK", [])?;
                return Ok((0, 0.0));
            }
            
            // Add share to all users' boops
            let updated_users = conn.execute(
                "UPDATE users SET boops = boops + ? WHERE server_id = ?",
                params![share_per_user, server_id],
            )? as i64;
            
            // Set communal boops to 0
            conn.execute(
                "UPDATE servers SET communal_boops = 0.0 WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Increment the distribution round
            conn.execute(
                "UPDATE servers SET current_distribution_round = current_distribution_round + 1 WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Get the new round number
            let new_round: i64 = conn.query_row(
                "SELECT current_distribution_round FROM servers WHERE server_id = ?",
                params![server_id],
                |row| row.get(0),
            )?;
            
            // Clear all previous distribution claims for this server to start fresh
            conn.execute(
                "DELETE FROM distribution_claims WHERE server_id = ?",
                params![server_id],
            )?;
            
            // Commit transaction
            conn.execute("COMMIT", [])?;
            
            println!("Distributed {:.2} boops to each of {} users in server {}", 
                    share_per_user, updated_users, server_id);
                
            Ok::<(i64, f64), rusqlite::Error>((updated_users, share_per_user))
        }).await
    }

    // Game score functions
    pub async fn save_game_score(&self, user_id: &str, server_id: &str, username: &str, game_type: &str, score: f64) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let server_id = server_id.to_string();
        let username = username.to_string();
        let game_type = game_type.to_string();
        
        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO game_scores (user_id, server_id, username, game_type, score) VALUES (?, ?, ?, ?, ?)",
                params![user_id, server_id, username, game_type, score],
            )?;
            
            println!("Saved score {} for {} in game {}", score, username, game_type);
            Ok::<_, rusqlite::Error>(())
        }).await
    }
    
    pub async fn get_user_best_score(&self, user_id: &str, server_id: &str, game_type: &str) -> DbResult<Option<f64>> {
        let conn = self.conn.lock().await;
        let user_id = user_id.to_string();
        let server_id = server_id.to_string();
        let game_type = game_type.to_string();
        
        conn.call(move |conn| {
            let result = conn.query_row(
                "SELECT MIN(score) FROM game_scores WHERE user_id = ? AND server_id = ? AND game_type = ?",
                params![user_id, server_id, game_type],
                |row| row.get(0),
            );
            
            match result {
                Ok(score) => Ok(Some(score)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        }).await
    }
    
    pub async fn get_server_leaderboard(&self, server_id: &str, game_type: &str, limit: usize) -> DbResult<Vec<(String, f64)>> {
        let conn = self.conn.lock().await;
        let server_id = server_id.to_string();
        let game_type = game_type.to_string();
        
        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT username, MIN(score) as best_score FROM game_scores 
                WHERE server_id = ? AND game_type = ? 
                GROUP BY user_id 
                ORDER BY best_score ASC 
                LIMIT ?"
            )?;
            
            let rows = stmt.query_map(params![server_id, game_type, limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?;
            
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            
            Ok(results)
        }).await
    }
} 