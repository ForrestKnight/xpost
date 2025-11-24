use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Draft {
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        Self {
            id: now.timestamp_millis().to_string(),
            content,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    pub fn preview(&self) -> String {
        let first_line = self.content.lines().next().unwrap_or("");
        let preview = if first_line.len() > 60 {
            format!("{}...", &first_line[..60])
        } else {
            first_line.to_string()
        };
        
        let date = self.updated_at.format("%Y-%m-%d %H:%M").to_string();
        format!("{} | {}", date, preview)
    }
}

fn drafts_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let config_dir = Path::new(&home).join(".config").join("xpost").join("drafts");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .context("Failed to create drafts directory")?;
    }
    
    Ok(config_dir)
}

pub fn save_draft(draft: &Draft) -> Result<()> {
    let dir = drafts_dir()?;
    let file_path = dir.join(format!("{}.json", draft.id));
    
    let json = serde_json::to_string_pretty(draft)
        .context("Failed to serialize draft")?;
    
    fs::write(&file_path, json)
        .context("Failed to write draft file")?;
    
    Ok(())
}

pub fn load_drafts() -> Result<Vec<Draft>> {
    let dir = drafts_dir()?;
    
    if !dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut drafts = Vec::new();
    
    for entry in fs::read_dir(dir).context("Failed to read drafts directory")? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(draft) = serde_json::from_str::<Draft>(&content) {
                    drafts.push(draft);
                }
            }
        }
    }
    
    // Sort by updated_at descending (most recent first)
    drafts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    
    Ok(drafts)
}

pub fn delete_draft(draft_id: &str) -> Result<()> {
    let dir = drafts_dir()?;
    let file_path = dir.join(format!("{}.json", draft_id));
    
    if file_path.exists() {
        fs::remove_file(&file_path)
            .context("Failed to delete draft file")?;
    }
    
    Ok(())
}
