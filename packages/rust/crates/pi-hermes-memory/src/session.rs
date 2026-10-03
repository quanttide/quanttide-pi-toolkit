//! `pi-hermes-memory` 扩展的会话数据访问：列表、JSONL 导出与统计分析。
//!
//! 数据源是插件维护的派生索引 `sessions.db`；pi 主程序本身不写数据库。

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

#[cfg(test)]
use rusqlite::Connection;
use serde::Serialize;

use crate::db::open_db;

#[derive(Debug, Serialize)]
pub struct SessionMeta {
    pub id: String,
    pub project: String,
    pub cwd: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub message_count: i64,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<String>,
    pub timestamp: String,
}

fn row_to_session(row: &rusqlite::Row) -> rusqlite::Result<SessionMeta> {
    Ok(SessionMeta {
        id: row.get("id")?,
        project: row.get("project")?,
        cwd: row.get("cwd")?,
        started_at: row.get("started_at")?,
        ended_at: row.get("ended_at")?,
        message_count: row.get("message_count")?,
    })
}

fn get_session(db: &Path, session_id: &str) -> Result<SessionMeta, String> {
    let conn = open_db(db)?;
    conn.query_row(
        "SELECT id, project, cwd, started_at, ended_at, message_count
         FROM sessions WHERE id = ?1",
        [session_id],
        row_to_session,
    )
    .map_err(|e| format!("Session not found ({session_id}): {e}"))
}

fn get_messages(db: &Path, session_id: &str) -> Result<Vec<Message>, String> {
    let conn = open_db(db)?;
    let mut stmt = conn
        .prepare(
            "SELECT role, content, tool_calls, timestamp
             FROM messages
             WHERE session_id = ?1
             ORDER BY timestamp ASC",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let rows = stmt
        .query_map([session_id], |row| {
            Ok(Message {
                role: row.get("role")?,
                content: row.get("content")?,
                tool_calls: row.get("tool_calls")?,
                timestamp: row.get("timestamp")?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|e| format!("Row read error: {e}"))?);
    }
    Ok(messages)
}

/// List recent sessions (newest first).
pub fn list_sessions(db: &Path, limit: usize) -> Result<Vec<SessionMeta>, String> {
    let conn = open_db(db)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project, cwd, started_at, ended_at, message_count
             FROM sessions
             ORDER BY started_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let rows = stmt
        .query_map([limit as i64], row_to_session)
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|e| format!("Row read error: {e}"))?);
    }
    Ok(sessions)
}

/// Export sessions to JSONL files.
/// Returns `(session_id, path, message_count, size_kb)` per exported file.
pub fn export_sessions(
    db: &Path,
    out_dir: &Path,
    session_id: Option<&str>,
    limit: usize,
) -> Result<Vec<(String, String, usize, u64)>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| format!("Cannot create output dir: {e}"))?;

    let session_ids: Vec<String> = match session_id {
        Some(sid) => vec![sid.to_string()],
        None => list_sessions(db, limit)?
            .into_iter()
            .map(|s| s.id)
            .collect(),
    };

    let mut results = Vec::new();
    for sid in &session_ids {
        let meta = get_session(db, sid)?;
        let msgs = get_messages(db, sid)?;

        let file_path = out_dir.join(format!("{}_{}.jsonl", meta.id, sanitize(&meta.project)));
        let mut file = std::fs::File::create(&file_path)
            .map_err(|e| format!("Cannot create {file_path:?}: {e}"))?;

        let meta_line = serde_json::json!({
            "type": "session_meta",
            "session_id": meta.id,
            "project": meta.project,
            "cwd": meta.cwd,
            "started_at": meta.started_at,
            "ended_at": meta.ended_at,
            "message_count": meta.message_count,
        });
        let meta_line = serde_json::to_string(&meta_line)
            .map_err(|e| format!("Serialize session meta failed: {e}"))?;
        writeln!(file, "{meta_line}").map_err(|e| format!("Write failed ({sid}): {e}"))?;

        for m in &msgs {
            let mut msg = serde_json::Map::new();
            msg.insert("role".into(), serde_json::Value::String(m.role.clone()));
            msg.insert(
                "content".into(),
                serde_json::Value::String(m.content.clone()),
            );
            if let Some(tool_calls) = &m.tool_calls {
                msg.insert(
                    "tool_calls".into(),
                    serde_json::Value::String(tool_calls.clone()),
                );
            }
            msg.insert(
                "timestamp".into(),
                serde_json::Value::String(m.timestamp.clone()),
            );
            let line = serde_json::to_string(&msg)
                .map_err(|e| format!("Serialize message failed: {e}"))?;
            writeln!(file, "{line}").map_err(|e| format!("Write failed ({sid}): {e}"))?;
        }

        let size_kb = std::fs::metadata(&file_path)
            .map(|m| m.len() / 1024)
            .unwrap_or(0);

        results.push((
            meta.id,
            file_path.display().to_string(),
            msgs.len(),
            size_kb,
        ));
    }

    Ok(results)
}

/// Aggregate statistics over recent sessions, or a single session when `session_id` is given.
pub fn analyze(db: &Path, session_id: Option<&str>, limit: usize) -> Result<String, String> {
    let sessions = match session_id {
        Some(id) => vec![get_session(db, id)?],
        None => {
            let recent = list_sessions(db, limit)?;
            if recent.is_empty() {
                return Err("No sessions found".to_string());
            }
            recent
        }
    };

    let mut user_n = 0usize;
    let mut assistant_n = 0usize;
    let mut system_n = 0usize;
    let mut tool_call_msgs = 0usize;
    let mut total_chars = 0usize;
    let mut empty_sessions = 0usize;
    let mut projects: BTreeMap<&str, usize> = BTreeMap::new();

    for s in &sessions {
        *projects.entry(s.project.as_str()).or_default() += 1;
        let msgs = get_messages(db, &s.id)?;
        if msgs.is_empty() {
            empty_sessions += 1;
        }
        for m in &msgs {
            match m.role.as_str() {
                "user" => user_n += 1,
                "assistant" => assistant_n += 1,
                "system" => system_n += 1,
                _ => {}
            }
            if m.tool_calls.is_some() {
                tool_call_msgs += 1;
            }
            total_chars += m.content.chars().count();
        }
    }

    let message_total = user_n + assistant_n + system_n;
    let pct = |n: usize| {
        if message_total == 0 {
            0.0
        } else {
            n as f64 / message_total as f64 * 100.0
        }
    };
    let avg_len = if message_total == 0 {
        0.0
    } else {
        total_chars as f64 / message_total as f64
    };
    let range_from = sessions
        .iter()
        .map(|s| s.started_at.as_str())
        .min()
        .unwrap_or("-");
    let range_to = sessions
        .iter()
        .map(|s| s.started_at.as_str())
        .max()
        .unwrap_or("-");
    let projects = projects
        .iter()
        .map(|(name, count)| format!("{name}({count})"))
        .collect::<Vec<_>>()
        .join(", ");
    let scope = match session_id {
        Some(id) => format!("Session: {id}"),
        None => format!("last {} session(s)", sessions.len()),
    };

    Ok(format!(
        r#"📊 pi Session Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Scope: {scope}

Sessions
  Scanned: {scanned}
  Empty: {empty}
  Projects: {projects}

Messages
  User: {user} ({user_pct:.1}%)
  Assistant: {assistant} ({assistant_pct:.1}%)
  System: {system} ({system_pct:.1}%)
  Total: {total}
  With tool calls: {tool_msgs}
  Avg length: {avg_len:.0} chars

Range
  From: {range_from}
  To: {range_to}
"#,
        scanned = sessions.len(),
        empty = empty_sessions,
        user = user_n,
        user_pct = pct(user_n),
        assistant = assistant_n,
        assistant_pct = pct(assistant_n),
        system = system_n,
        system_pct = pct(system_n),
        total = message_total,
        tool_msgs = tool_call_msgs,
    ))
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Build a fixture DB with pi schema for tests (also used by `memory` tests).
#[cfg(test)]
pub(crate) fn seed_test_db(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        r#"CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            project TEXT NOT NULL,
            cwd TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            message_count INTEGER DEFAULT 0
        );
        CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            tool_calls TEXT
        );
        CREATE TABLE memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project TEXT,
            target TEXT NOT NULL CHECK (target IN ('memory', 'user', 'failure')),
            category TEXT CHECK (category IN ('failure', 'correction', 'insight', 'preference', 'convention', 'tool-quirk')),
            content TEXT NOT NULL,
            failure_reason TEXT,
            tool_state TEXT,
            corrected_to TEXT,
            created DATE NOT NULL,
            last_referenced DATE NOT NULL
        );
        INSERT INTO sessions (id, project, cwd, started_at, ended_at, message_count) VALUES
            ('sess-aaa', 'demo', '/tmp/demo', '2026-10-01T10:00:00.000Z', '2026-10-01T10:05:00.000Z', 3),
            ('sess-bbb', 'tmp', '/tmp', '2026-10-02T09:00:00.000Z', NULL, 1),
            ('sess-ccc', 'demo', '/tmp/demo', '2026-10-03T08:00:00.000Z', NULL, 0);
        INSERT INTO messages (id, session_id, role, content, timestamp, tool_calls) VALUES
            ('m1', 'sess-aaa', 'user', 'hello world', '2026-10-01T10:00:01.000Z', NULL),
            ('m2', 'sess-aaa', 'assistant', 'hi there', '2026-10-01T10:00:02.000Z', '[{"name":"read"}]'),
            ('m3', 'sess-aaa', 'system', 'sys prompt', '2026-10-01T10:00:03.000Z', NULL),
            ('m4', 'sess-bbb', 'user', 'second session', '2026-10-02T09:00:01.000Z', NULL);
        INSERT INTO memories (id, project, target, category, content, failure_reason, tool_state, corrected_to, created, last_referenced) VALUES
            (1, 'demo', 'user', 'preference', 'likes concise reports', NULL, NULL, NULL, '2026-09-01', '2026-09-02'),
            (2, 'demo', 'failure', 'failure', 'never run git push', 'push rejected', 'cli', 'git pull first', '2026-09-03', '2026-09-03'),
            (3, NULL, 'memory', NULL, 'global note', NULL, NULL, NULL, '2026-09-04', '2026-09-04');"#,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_db(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        seed_test_db(&path);
        (dir, path)
    }

    #[test]
    fn test_list_sessions_order_and_limit() {
        let (_guard, db) = fixture_db("sessions.db");

        let all = list_sessions(&db, 10).unwrap();
        let ids: Vec<&str> = all.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["sess-ccc", "sess-bbb", "sess-aaa"]);
        assert_eq!(all[0].message_count, 0);
        assert_eq!(all[2].project, "demo");
        assert_eq!(all[2].ended_at.as_deref(), Some("2026-10-01T10:05:00.000Z"));

        let limited = list_sessions(&db, 2).unwrap();
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].id, "sess-ccc");

        assert!(list_sessions(&db, 0).unwrap().is_empty());
    }

    #[test]
    fn test_list_sessions_missing_db() {
        let dir = tempfile::tempdir().unwrap();
        let err = list_sessions(&dir.path().join("nope.db"), 10).unwrap_err();
        assert!(err.contains("not found"), "unexpected error: {err}");
    }

    #[test]
    fn test_export_sessions_recent() {
        let (_guard, db) = fixture_db("sessions.db");
        let out = tempfile::tempdir().unwrap();

        let files = export_sessions(&db, out.path(), None, 2).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].0, "sess-ccc");
        assert_eq!(files[0].2, 0);

        let content = std::fs::read_to_string(&files[0].1).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        let meta: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(meta["type"], "session_meta");
        assert_eq!(meta["session_id"], "sess-ccc");
        assert_eq!(meta["project"], "demo");

        let content = std::fs::read_to_string(&files[1].1).unwrap();
        assert_eq!(content.lines().count(), 2);
    }

    #[test]
    fn test_export_single_session() {
        let (_guard, db) = fixture_db("sessions.db");
        let out = tempfile::tempdir().unwrap();

        let files = export_sessions(&db, out.path(), Some("sess-aaa"), 10).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].2, 3);

        let content = std::fs::read_to_string(&files[0].1).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 4);
        let msg: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(msg["role"], "user");
        assert_eq!(msg["content"], "hello world");
        let msg: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(msg["tool_calls"], "[{\"name\":\"read\"}]");
    }

    #[test]
    fn test_export_unknown_session() {
        let (_guard, db) = fixture_db("sessions.db");
        let out = tempfile::tempdir().unwrap();

        let err = export_sessions(&db, out.path(), Some("sess-missing"), 10).unwrap_err();
        assert!(err.contains("sess-missing"), "unexpected error: {err}");
    }

    #[test]
    fn test_analyze_recent_sessions() {
        let (_guard, db) = fixture_db("sessions.db");

        let report = analyze(&db, None, 3).unwrap();
        assert!(report.contains("pi Session Analysis"));
        assert!(report.contains("Scope: last 3 session(s)"));
        assert!(report.contains("Scanned: 3"));
        assert!(report.contains("Empty: 1"));
        assert!(report.contains("Projects: demo(2), tmp(1)"));
        assert!(report.contains("User: 2 (50.0%)"));
        assert!(report.contains("Assistant: 1 (25.0%)"));
        assert!(report.contains("System: 1 (25.0%)"));
        assert!(report.contains("Total: 4"));
        assert!(report.contains("With tool calls: 1"));
        assert!(report.contains("Avg length: 11 chars"));
        assert!(report.contains("From: 2026-10-01T10:00:00.000Z"));
        assert!(report.contains("To: 2026-10-03T08:00:00.000Z"));
    }

    #[test]
    fn test_analyze_single_session() {
        let (_guard, db) = fixture_db("sessions.db");

        let report = analyze(&db, Some("sess-aaa"), 10).unwrap();
        assert!(report.contains("Scope: Session: sess-aaa"));
        assert!(report.contains("Scanned: 1"));
        assert!(report.contains("Empty: 0"));
        assert!(report.contains("Total: 3"));
        assert!(report.contains("With tool calls: 1"));
        assert!(report.contains("From: 2026-10-01T10:00:00.000Z"));
        assert!(report.contains("To: 2026-10-01T10:00:00.000Z"));
    }

    #[test]
    fn test_analyze_errors() {
        let (_guard, db) = fixture_db("sessions.db");

        let err = analyze(&db, Some("sess-missing"), 10).unwrap_err();
        assert!(err.contains("sess-missing"), "unexpected error: {err}");

        let err = analyze(&db, None, 0).unwrap_err();
        assert_eq!(err, "No sessions found");
    }
}
