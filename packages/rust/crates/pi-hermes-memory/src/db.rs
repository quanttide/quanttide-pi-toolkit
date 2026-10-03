//! 扩展数据访问的共享打开逻辑。

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

/// 以只读用途打开插件 SQLite 数据库：校验存在性并设置 busy timeout。
///
/// 锁只约束插件写方，只读查询以超时兜底。
pub fn open_db(path: &Path) -> Result<Connection, String> {
    if !path.exists() {
        return Err(format!("database not found at {}", path.display()));
    }
    let conn = Connection::open(path)
        .map_err(|e| format!("Cannot open database at {}: {e}", path.display()))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|e| format!("Cannot set busy timeout: {e}"))?;
    Ok(conn)
}
