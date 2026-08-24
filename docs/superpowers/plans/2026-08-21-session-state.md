# 会话与工作区真实状态 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the sidebar's hard-coded sessions with durable `AppState` session and workspace operations.

**Architecture:** `AppState` owns all lifecycle changes and delegates file changes to `SessionPersistence`. `Sidebar` takes an ID-based snapshot of core state and keeps only presentation state locally. Server events save their mutated session so the sidebar and a restarted app observe the same data.

**Tech Stack:** Rust, Tokio `RwLock`, Serde JSON persistence, Chrono, UUID, GPUI.

---

## File structure

- `crates/dsh-core/src/lib.rs` — session metadata, lifecycle methods, workspace mutation and event persistence.
- `crates/dsh-core/src/persistence.rs` — remove one persisted session and test persistence behavior.
- `crates/dsh-ui/src/sidebar.rs` — derive sidebar items from `AppState`, call ID-based operations, rescan the selected workspace.
- `docs/superpowers/specs/2026-08-21-session-state-design.md` — accepted design boundary.

### Task 1: Test and implement core session lifecycle

**Files:**
- Modify: `crates/dsh-core/src/lib.rs`
- Test: `crates/dsh-core/src/lib.rs`

- [ ] **Step 1: Add failing async tests for ID-based rename, duplicate, delete and active-session fallback.**

```rust
#[tokio::test]
async fn session_lifecycle_uses_ids_and_updates_active_session() {
    let (state, _) = AppState::new(DaemonConfig::default());
    let first = state.create_session("相同标题", "/tmp/a").await;
    let second = state.create_session("相同标题", "/tmp/b").await;

    assert!(state.rename_session(&first, "已重命名").await.unwrap());
    let copy = state.duplicate_session(&first).await.unwrap().unwrap();
    assert_ne!(copy, first);
    assert!(state.delete_session(&second).await.unwrap());
    assert_eq!(*state.active_session_id.read().await, Some(copy));
}
```

- [ ] **Step 2: Run the focused test and confirm it fails because lifecycle methods do not exist.**

Run: `cargo test -p dsh-core session_lifecycle_uses_ids_and_updates_active_session -- --exact`

Expected: compile failure mentioning `rename_session`, `duplicate_session` and `delete_session`.

- [ ] **Step 3: Add `created_at` and `updated_at` to `Session`, then add lifecycle methods.**

```rust
pub async fn select_session(&self, session_id: &str) -> bool;
pub async fn rename_session(&self, session_id: &str, title: &str) -> Result<bool>;
pub async fn duplicate_session(&self, session_id: &str) -> Result<Option<String>>;
pub async fn delete_session(&self, session_id: &str) -> Result<bool>;
pub async fn session_snapshot(&self) -> Vec<Session>;
pub async fn set_workspace_path(&self, workspace: PathBuf);
```

`duplicate_session` clones only user-visible session content, assigns a new UUID and uses `"{title} 副本"`. Every mutating method updates `updated_at`, saves the resulting session, and only updates the active ID after the mutation succeeds.

- [ ] **Step 4: Run focused tests and full core tests.**

Run: `cargo test -p dsh-core`

Expected: all `dsh-core` tests pass.

- [ ] **Step 5: Commit the isolated core lifecycle change.**

```bash
git add crates/dsh-core/src/lib.rs
git commit -m "feat(core): add durable session lifecycle operations"
```

### Task 2: Test and implement persisted deletion plus event saves

**Files:**
- Modify: `crates/dsh-core/src/persistence.rs`
- Modify: `crates/dsh-core/src/lib.rs`
- Test: `crates/dsh-core/src/persistence.rs`

- [ ] **Step 1: Add a failing persistence test for removing one session file.**

```rust
#[test]
fn delete_session_removes_only_requested_file() {
    let temp_dir = env::temp_dir().join(format!("dsh_persist_{}", uuid::Uuid::new_v4()));
    SessionPersistence::save_session(&temp_dir, &session("one")).unwrap();
    SessionPersistence::save_session(&temp_dir, &session("two")).unwrap();
    SessionPersistence::delete_session(&temp_dir, "one").unwrap();
    assert_eq!(SessionPersistence::load_all_sessions(&temp_dir).unwrap()[0].id, "two");
    let _ = fs::remove_dir_all(temp_dir);
}
```

- [ ] **Step 2: Run it and confirm it fails because `delete_session` is absent.**

Run: `cargo test -p dsh-core delete_session_removes_only_requested_file -- --exact`

Expected: compile failure mentioning `SessionPersistence::delete_session`.

- [ ] **Step 3: Add `SessionPersistence::delete_session` and save every changed session after server events.**

```rust
pub fn delete_session(storage_dir: &Path, session_id: &str) -> Result<()> {
    let session_file = storage_dir.join("sessions").join(format!("{session_id}.json"));
    if session_file.exists() { fs::remove_file(session_file)?; }
    Ok(())
}
```

In `handle_server_event`, save the session after `TokenChunk`, `ToolCallStart`, `ToolCallEnd`, `FileDiffReady`, `AgentStateChange`, and `TerminalLog` mutations.

- [ ] **Step 4: Run the full core suite.**

Run: `cargo test -p dsh-core`

Expected: all tests pass.

- [ ] **Step 5: Commit persistence changes.**

```bash
git add crates/dsh-core/src/lib.rs crates/dsh-core/src/persistence.rs
git commit -m "fix(core): persist session lifecycle and server updates"
```

### Task 3: Bind sidebar to real session and workspace state

**Files:**
- Modify: `crates/dsh-ui/src/sidebar.rs`
- Modify: `crates/dsh-ui/src/workspace.rs` only if an explicit sidebar refresh handle is needed

- [ ] **Step 1: Add a pure helper that converts and orders a session snapshot.**

```rust
fn visible_session_items(sessions: Vec<Session>, query: &str, sort_by_name: bool) -> Vec<SessionItemView> {
    // filter title case-insensitively; sort by title or descending updated_at
}
```

- [ ] **Step 2: Replace the hard-coded `Sidebar::sessions` initialization with an empty projection and synchronize it from `AppState::session_snapshot`.**

```rust
Self { sessions: Vec::new(), active_workspace: ".".into(), /* existing transient fields */ }
```

Use a short background refresh loop matching `ChatView`'s existing state-sync pattern. Mark the item active by comparing each `Session.id` with `active_session_id`.

- [ ] **Step 3: Route selection and menu actions through the ID-based core methods.**

```rust
state.select_session(&id).await;
state.rename_session(&id, &title).await?;
state.duplicate_session(&id).await?;
state.delete_session(&id).await?;
```

Do not create a new session when selection fails. After each action, clear the transient menu/rename state and refresh from the returned core snapshot.

- [ ] **Step 4: Replace hard-coded workspace switching with path mutation and scan the selected path.**

```rust
let workspace_path = PathBuf::from(name);
state.set_workspace_path(workspace_path.clone()).await;
self.file_tree = WorkspaceScanner::scan_dir(&workspace_path, 2).ok();
```

- [ ] **Step 5: Check compilation and formatting.**

Run: `cargo fmt --check && cargo check -p dsh-ui`

Expected: formatter and UI crate checks pass.

- [ ] **Step 6: Commit the sidebar binding.**

```bash
git add crates/dsh-ui/src/sidebar.rs crates/dsh-ui/src/workspace.rs
git commit -m "feat(ui): bind sidebar actions to session state"
```

### Task 4: Verify the feature branch

**Files:**
- Modify: no source files unless a verification failure requires the smallest correction

- [ ] **Step 1: Run the complete workspace test suite.**

Run: `cargo test --workspace`

Expected: all workspace tests pass.

- [ ] **Step 2: Inspect the branch diff and status.**

Run: `git diff main...HEAD --check && git status --short`

Expected: no whitespace errors and no uncommitted tracked feature files.

- [ ] **Step 3: Build a Windows release package for manual validation.**

Run: `powershell -ExecutionPolicy Bypass -File scripts/package-windows.ps1`

Expected: `target/dist/DeepSeek-Harness-Desktop-Windows-x64.zip` is recreated.
