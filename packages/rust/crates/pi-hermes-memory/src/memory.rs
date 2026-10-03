//! `pi-hermes-memory` 扩展的记忆数据访问与导出（Markdown / JSONL）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::db::open_db;

/// One row of the pi `memories` table.
#[derive(Debug, Serialize)]
pub struct Memory {
    pub id: i64,
    pub project: Option<String>,
    pub target: String,
    pub category: Option<String>,
    pub content: String,
    pub failure_reason: Option<String>,
    pub tool_state: Option<String>,
    pub corrected_to: Option<String>,
    pub created: String,
    pub last_referenced: String,
}

fn row_to_memory(row: &rusqlite::Row) -> rusqlite::Result<Memory> {
    Ok(Memory {
        id: row.get("id")?,
        project: row.get("project")?,
        target: row.get("target")?,
        category: row.get("category")?,
        content: row.get("content")?,
        failure_reason: row.get("failure_reason")?,
        tool_state: row.get("tool_state")?,
        corrected_to: row.get("corrected_to")?,
        created: row.get("created")?,
        last_referenced: row.get("last_referenced")?,
    })
}

/// Load all memories, ordered by `target` then `id`.
pub fn load_memories(db: &Path) -> Result<Vec<Memory>, String> {
    let conn = open_db(db)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project, target, category, content, failure_reason,
                    tool_state, corrected_to, created, last_referenced
             FROM memories
             ORDER BY target, id",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let rows = stmt
        .query_map([], row_to_memory)
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut memories = Vec::new();
    for row in rows {
        memories.push(row.map_err(|e| format!("Row read error: {e}"))?);
    }
    Ok(memories)
}

/// Render memories as Markdown, grouped by `target`.
pub fn render_markdown(memories: &[Memory]) -> String {
    let mut groups: BTreeMap<&str, Vec<&Memory>> = BTreeMap::new();
    for m in memories {
        groups.entry(m.target.as_str()).or_default().push(m);
    }

    let exported = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let mut out = format!(
        "# pi Memories\n\nExported: {exported} — {} entries\n",
        memories.len()
    );
    for (target, items) in &groups {
        out.push_str(&format!("\n## {target} ({})\n", items.len()));
        for m in items {
            let project = m.project.as_deref().unwrap_or("-");
            let category = m.category.as_deref().unwrap_or("-");
            let id = m.id;
            out.push_str(&format!("\n### {id} · {project}\n\n"));
            out.push_str(&format!("- category: {category}\n"));
            out.push_str(&format!(
                "- created: {} → last referenced: {}\n",
                m.created, m.last_referenced
            ));
            if let Some(reason) = &m.failure_reason {
                out.push_str(&format!("- failure_reason: {reason}\n"));
            }
            if let Some(state) = &m.tool_state {
                out.push_str(&format!("- tool_state: {state}\n"));
            }
            if let Some(to) = &m.corrected_to {
                out.push_str(&format!("- corrected_to: {to}\n"));
            }
            out.push_str(&format!("\n{}\n", m.content));
        }
    }
    out
}

/// Render memories as JSONL, one object per line.
pub fn render_jsonl(memories: &[Memory]) -> Result<String, String> {
    let mut out = String::new();
    for m in memories {
        let id = m.id;
        let line = serde_json::to_string(m).map_err(|e| format!("Serialize memory {id}: {e}"))?;
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Export memories to `out_dir/memories.md` or `out_dir/memories.jsonl`.
/// Returns `(path, memory_count)`.
pub fn export_memories(
    db: &Path,
    out_dir: &Path,
    format: &str,
) -> Result<(PathBuf, usize), String> {
    if !matches!(format, "md" | "jsonl") {
        return Err(format!(
            "Unsupported format ({format}), expected md or jsonl"
        ));
    }

    let memories = load_memories(db)?;
    let count = memories.len();
    let (content, filename) = if format == "jsonl" {
        (render_jsonl(&memories)?, "memories.jsonl")
    } else {
        (render_markdown(&memories), "memories.md")
    };

    std::fs::create_dir_all(out_dir).map_err(|e| format!("Cannot create output dir: {e}"))?;
    let path = out_dir.join(filename);
    std::fs::write(&path, content).map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    Ok((path, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::seed_test_db;

    fn fixture_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sessions.db");
        seed_test_db(&path);
        (dir, path)
    }

    #[test]
    fn test_load_memories() {
        let (_guard, db) = fixture_db();

        let memories = load_memories(&db).unwrap();
        assert_eq!(memories.len(), 3);
        let targets: Vec<&str> = memories.iter().map(|m| m.target.as_str()).collect();
        assert_eq!(targets, vec!["failure", "memory", "user"]);
        assert_eq!(memories[0].failure_reason.as_deref(), Some("push rejected"));
        assert_eq!(memories[0].corrected_to.as_deref(), Some("git pull first"));
        assert_eq!(memories[1].project, None);
        assert_eq!(memories[1].category, None);
        assert_eq!(memories[2].target, "user");
        assert_eq!(memories[2].category.as_deref(), Some("preference"));
    }

    #[test]
    fn test_load_memories_missing_db() {
        let dir = tempfile::tempdir().unwrap();
        let err = load_memories(&dir.path().join("nope.db")).unwrap_err();
        assert!(err.contains("not found"), "unexpected error: {err}");
    }

    #[test]
    fn test_render_markdown_groups_by_target() {
        let (_guard, db) = fixture_db();
        let memories = load_memories(&db).unwrap();

        let md = render_markdown(&memories);
        assert!(md.starts_with("# pi Memories\n"));
        assert!(md.contains("— 3 entries"));
        assert!(md.contains("## failure (1)"));
        assert!(md.contains("## memory (1)"));
        assert!(md.contains("## user (1)"));

        let failure = md.find("## failure").unwrap();
        let memory = md.find("## memory").unwrap();
        let user = md.find("## user").unwrap();
        assert!(failure < memory && memory < user);

        let content = md.find("never run git push").unwrap();
        assert!(content > failure && content < memory);
        assert!(md.contains("- failure_reason: push rejected"));
        assert!(md.contains("- tool_state: cli"));
        assert!(md.contains("- category: preference"));
        assert!(md.contains("\nlikes concise reports\n"));
    }

    #[test]
    fn test_render_markdown_empty() {
        let md = render_markdown(&[]);
        assert!(md.starts_with("# pi Memories\n"));
        assert!(md.contains("— 0 entries"));
        assert!(!md.contains("## "));
    }

    #[test]
    fn test_render_jsonl() {
        let (_guard, db) = fixture_db();
        let memories = load_memories(&db).unwrap();

        let out = render_jsonl(&memories).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);

        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["target"], "failure");
        assert_eq!(first["content"], "never run git push");
        assert!(lines[1].contains("\"target\":\"memory\""));

        assert_eq!(render_jsonl(&[]).unwrap(), "");
    }

    #[test]
    fn test_export_memories_md() {
        let (_guard, db) = fixture_db();
        let out = tempfile::tempdir().unwrap();

        let (path, count) = export_memories(&db, out.path(), "md").unwrap();
        assert_eq!(count, 3);
        assert_eq!(path.file_name().unwrap(), "memories.md");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("## user (1)"));
    }

    #[test]
    fn test_export_memories_jsonl() {
        let (_guard, db) = fixture_db();
        let out = tempfile::tempdir().unwrap();

        let (path, count) = export_memories(&db, out.path(), "jsonl").unwrap();
        assert_eq!(count, 3);
        assert_eq!(path.file_name().unwrap(), "memories.jsonl");
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), 3);
    }

    #[test]
    fn test_export_memories_invalid_format() {
        let (_guard, db) = fixture_db();
        let out = tempfile::tempdir().unwrap();

        let err = export_memories(&db, out.path(), "yaml").unwrap_err();
        assert!(err.contains("md or jsonl"), "unexpected error: {err}");
    }
}
