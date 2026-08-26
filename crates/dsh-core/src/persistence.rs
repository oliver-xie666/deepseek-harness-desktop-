use crate::Session;
use dsh_common::Result;
use std::fs;
use std::path::Path;
use tracing::info;

pub struct SessionPersistence;

impl SessionPersistence {
    pub fn save_session(storage_dir: &Path, session: &Session) -> Result<()> {
        let sessions_dir = storage_dir.join("sessions");
        if !sessions_dir.exists() {
            fs::create_dir_all(&sessions_dir)?;
        }

        let session_file = sessions_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)?;
        fs::write(&session_file, json)?;

        info!(
            "Persisted session {} to {}",
            session.id,
            session_file.display()
        );
        Ok(())
    }

    pub fn load_all_sessions(storage_dir: &Path) -> Result<Vec<Session>> {
        let sessions_dir = storage_dir.join("sessions");
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        if let Ok(entries) = fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(session) = serde_json::from_str::<Session>(&content) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    pub fn delete_session(storage_dir: &Path, session_id: &str) -> Result<()> {
        let session_file = storage_dir
            .join("sessions")
            .join(format!("{session_id}.json"));
        if session_file.exists() {
            fs::remove_file(session_file)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_save_and_load_session() {
        let temp_dir = env::temp_dir().join(format!("dsh_persist_{}", uuid::Uuid::new_v4()));

        let session = Session {
            id: "test-sess-1".into(),
            title: "Test Persistence".into(),
            workspace_path: "/tmp/workspace".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            messages: Vec::new(),
            diffs: std::collections::HashMap::new(),
            terminal_logs: vec!["Log 1".into()],
            agent_state: None,
            plan_state: None,
            pending_question: None,
            goal: None,
            jobs: Vec::new(),
        };

        SessionPersistence::save_session(&temp_dir, &session).unwrap();

        let loaded = SessionPersistence::load_all_sessions(&temp_dir).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-sess-1");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn delete_session_removes_only_requested_file() {
        let temp_dir = env::temp_dir().join(format!("dsh_persist_{}", uuid::Uuid::new_v4()));
        let make_session = |id: &str| Session {
            id: id.into(),
            title: id.into(),
            workspace_path: "/tmp/workspace".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            messages: Vec::new(),
            diffs: std::collections::HashMap::new(),
            terminal_logs: Vec::new(),
            agent_state: None,
            plan_state: None,
            pending_question: None,
            goal: None,
            jobs: Vec::new(),
        };

        SessionPersistence::save_session(&temp_dir, &make_session("one")).unwrap();
        SessionPersistence::save_session(&temp_dir, &make_session("two")).unwrap();
        SessionPersistence::delete_session(&temp_dir, "one").unwrap();

        let loaded = SessionPersistence::load_all_sessions(&temp_dir).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "two");
        let _ = fs::remove_dir_all(temp_dir);
    }
}
