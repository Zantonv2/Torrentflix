# Agent Rules & MCP Guidelines for Torrentflix

This document provides comprehensive rules for all agents working on Torrentflix, including how to use MCP tools effectively and safely.

---

## 1. Core Principles

- **Read before write**: Always analyze code/data before making changes.
- **Reversible by default**: Keep changes small, staged, and committable.
- **Ask for confirmation**: Before destructive operations (deletes, schema changes, major refactors).
- **Document decisions**: Explain why changes were made, not just what.
- **Prefer tools over guessing**: Use MCP servers for accurate information instead of inferring from code.

---

## 2. MCP Tool Usage Guidelines

### 2.1 Filesystem MCP (`filesystem`)

**When to use:**
- Reading project files and structure
- Writing code changes
- Listing directories to understand layout

**Best practices:**
- Use specific file paths, not wildcards
- Read entire files first to understand context
- For edits: make small, focused changes
- Pair with git to track changes

**Auto-approved tools:**
- `list_allowed_directories` – Safe, use freely
- `list_directory` – Safe, use to explore structure
- `read_file` – Safe, use to understand code
- `write_file` – Use carefully; prefer small edits

**Example workflow:**
```
1. list_directory("engine/src") → understand structure
2. read_file("engine/src/engine.rs") → understand current implementation
3. write_file("engine/src/new_module.rs") → add new code
4. git_add + git_commit → save changes
```

---

### 2.2 Git MCP (`git`)

**When to use:**
- Understanding recent changes
- Checking what's staged/unstaged
- Reviewing diffs before committing
- Tracking blame/history

**Best practices:**
- Always check `git_status` before making changes
- Use `git_diff_unstaged` to review your edits
- Write clear, descriptive commit messages
- Never force-push or rewrite history without explicit user request

**Auto-approved tools:**
- `get_repository_status` – Safe, use to check state
- `get_commit_history` – Safe, use to understand history
- `get_diff` – Safe, use to review changes
- `git_status` – Safe, use frequently
- `git_log` – Safe, use to explore history

**Restricted tools (ask user first):**
- `git_add` – Stage files only when user confirms
- `git_commit` – Commit only when user asks to save
- `git_create_branch`, `git_checkout` – Only with explicit user intent

**Example workflow:**
```
1. git_status() → check current state
2. git_diff_unstaged() → review what changed
3. git_add(["file1.rs", "file2.rs"]) → stage specific files
4. git_commit("feat: add new feature") → save with message
```

---

### 2.3 SQLite MCP (`sqlite`)

**When to use:**
- Querying the database to understand data structure
- Analyzing schema and relationships
- Debugging data issues
- Checking job queue status

**Best practices:**
- Start with `SELECT` queries only
- Use `LIMIT` to avoid huge result sets
- Understand schema before writing complex queries
- Never `DELETE` or `DROP` without explicit user confirmation

**Auto-approved tools:**
- `query` – Use for SELECT queries
- `analyze` – Use to understand schema
- `list_tables` – Use to explore database structure
- `read_query` – Use for read-only operations

**Restricted tools (ask user first):**
- Any `INSERT`, `UPDATE`, `DELETE` operations
- Schema changes (`ALTER TABLE`, `CREATE TABLE`, `DROP`)

**Example workflow:**
```
1. list_tables() → see what tables exist
2. query("SELECT * FROM library LIMIT 5") → understand data
3. query("SELECT COUNT(*) FROM jobs WHERE status='pending'") → check queue
4. analyze() → get schema insights
```

---

### 2.4 TypeScript/Svelte LSP MCP (`typescript`)

**When to use:**
- Understanding TypeScript types and interfaces
- Finding symbol definitions and references
- Checking for type errors in UI code
- Navigating Svelte component structure

**Best practices:**
- Use for precise type information, not guessing
- Check diagnostics before writing new code
- Understand component props before using them
- Prefer LSP queries over reading raw files

**Auto-approved tools:**
- `list_dir` – Safe, use to explore project
- `get_project_overview` – Safe, use to understand structure

**Example workflow:**
```
1. get_project_overview() → understand UI structure
2. list_dir("ui/src/components") → see available components
3. Use LSP to check types before writing new components
```

---

### 2.5 Rust MCP (`rust`)

**When to use:**
- Building and checking the Rust workspace
- Running tests
- Analyzing code with clippy
- Managing dependencies

**Best practices:**
- Always `cargo check` before `cargo build`
- Run `cargo test` to validate changes
- Use `cargo clippy` to catch common mistakes
- Avoid `cargo add`/`cargo remove` unless explicitly asked

**Recommended workflow:**
```
1. cargo check --workspace → fast type check
2. cargo clippy --workspace → lint check
3. cargo test --workspace → run tests
4. cargo build --release → build if all pass
```

---

### 2.6 Context7 MCP (`context7`)

**When to use:**
- Looking up documentation for dependencies (tokio, serde, reqwest, sqlx, etc.)
- Understanding best practices for libraries
- Finding code examples for APIs
- Checking changelogs for version compatibility

**Best practices:**
- Use before writing code that uses external crates
- Prefer official docs over guessing API signatures
- Check for breaking changes in changelogs
- Use for learning unfamiliar libraries

**Auto-approved tools:**
- `search_documentation` – Safe, use to find docs
- `get_library_docs` – Safe, use to read docs
- `get_code_examples` – Safe, use to learn patterns
- `get_api_reference` – Safe, use for API details
- `get_best_practices` – Safe, use for guidance
- `list_supported_libraries` – Safe, use to check availability
- `get_changelog` – Safe, use to check versions

**Example workflow:**
```
1. search_documentation("tokio async patterns") → find relevant docs
2. get_code_examples("tokio spawn") → see how to use
3. get_api_reference("tokio::spawn") → understand exact signature
4. Write code with confidence
```

---

### 2.7 Shell MCP (`shell`) – DISABLED BY DEFAULT

**Status:** Disabled for safety. Only enable if user explicitly requests.

**When to use (if enabled):**
- Running build scripts
- Executing tests
- Running linters/formatters
- Checking system state

**Restrictions:**
- Never use `rm`, `sudo`, or destructive commands
- Avoid long-running processes
- Keep commands idempotent and safe
- Always ask user before enabling

---

## 3. Workflow Patterns

### 3.1 Adding a New Feature

```
1. git_status() → check current state
2. read_file("docs/ROADMAP.md") → understand architecture
3. filesystem: list_directory("engine/src") → find where to add code
4. context7: get_library_docs() → understand dependencies
5. Write code in small, focused files
6. cargo check → validate
7. cargo test → verify
8. git_add + git_commit → save
```

### 3.2 Debugging an Issue

```
1. git_log() → understand recent changes
2. git_diff() → see what changed
3. sqlite: query() → check data state
4. cargo check → find compile errors
5. Read relevant code files
6. Make targeted fix
7. cargo test → verify fix
8. git_commit → save
```

### 3.3 Refactoring Code

```
1. git_status() → ensure clean state
2. read_file() → understand current code
3. cargo test → establish baseline
4. Make small changes
5. cargo check → validate
6. cargo test → verify no regressions
7. git_diff_unstaged() → review changes
8. git_add + git_commit → save with clear message
```

---

## 4. Safety Rules

### 4.1 File Operations

- ✅ Read any file
- ✅ Write new files
- ✅ Edit existing files (small, focused changes)
- ❌ Delete files (ask user first)
- ❌ Overwrite large files (ask user first)
- ❌ Modify config files without understanding impact

### 4.2 Git Operations

- ✅ Check status, history, diffs
- ✅ Stage and commit when user asks
- ❌ Force push
- ❌ Rewrite history
- ❌ Delete branches
- ❌ Merge without understanding conflicts

### 4.3 Database Operations

- ✅ Read/query data
- ✅ Analyze schema
- ❌ Insert/update/delete (ask user first)
- ❌ Alter schema
- ❌ Drop tables

### 4.4 Rust Operations

- ✅ Check, test, clippy, build
- ✅ Run tests
- ✅ Format code
- ❌ Add/remove dependencies (ask user first)
- ❌ Publish crates

---

## 5. Code Quality Standards

### 5.1 Rust Code

- Follow the tech stack: Tokio async, sqlx, serde, anyhow
- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix warnings
- Write tests for new functionality
- Document public APIs with doc comments
- Use meaningful error messages

### 5.2 TypeScript/Svelte Code

- Use TypeScript strictly (no `any`)
- Follow Svelte conventions (reactive declarations, stores)
- Use TailwindCSS for styling
- Keep components focused and reusable
- Add prop documentation
- Test component behavior

### 5.3 Commits

- Write clear, descriptive messages
- Reference issues/features when relevant
- Keep commits focused (one feature per commit)
- Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`

---

## 6. Decision Tree: When to Use Which MCP

```
Need to understand code structure?
  → filesystem: list_directory, read_file

Need to understand recent changes?
  → git: git_log, git_diff

Need to understand data?
  → sqlite: query, analyze

Need to understand types/symbols?
  → typescript: LSP tools

Need to build/test Rust code?
  → rust: cargo check, cargo test

Need to understand a library/API?
  → context7: get_library_docs, get_code_examples

Need to save changes?
  → git: git_add, git_commit

Need to run arbitrary commands?
  → shell (disabled by default, ask user)
```

---

## 7. Common Mistakes to Avoid

1. **Guessing instead of querying**: Use MCP tools instead of inferring from code
2. **Large commits**: Keep changes small and focused
3. **Skipping tests**: Always run `cargo test` before committing
4. **Ignoring warnings**: Fix clippy warnings and compiler warnings
5. **Modifying without understanding**: Read code first, understand context
6. **Forgetting to stage changes**: Use `git_add` before `git_commit`
7. **Assuming schema**: Query the database to understand structure
8. **Ignoring type errors**: Use TypeScript LSP to catch errors early

---

## 8. Asking for Help

When uncertain:
- Ask the user for clarification
- Explain what you're trying to do and why
- Show the relevant code/data
- Propose a solution and ask for feedback
- Never guess on destructive operations

---

## 9. Performance & Token Efficiency

- Use `LIMIT` in SQL queries to avoid huge result sets
- Read only the files you need
- Use git diffs instead of reading entire files
- Batch related operations together
- Prefer MCP tools over reading raw files (they're more efficient)

---

## 10. Project-Specific Context

**Tech Stack:**
- Backend: Rust (Tokio, sqlx, serde)
- Frontend: Svelte + TypeScript + TailwindCSS
- Desktop: Tauri 2.0
- Database: SQLite (WAL mode)

**Key Directories:**
- `engine/src/` – Core Rust business logic
- `src-tauri/src/` – Tauri app wrapper
- `ui/src/` – Svelte frontend
- `.data/` – SQLite database
- `docs/` – Documentation

**Key Files:**
- `Cargo.toml` – Rust workspace config
- `package.json` – Node/npm config
- `ui/package.json` – UI dependencies
- `.kiro/settings/mcp.json` – MCP configuration

---

This document should be referenced by all agents before starting work. Keep it updated as new patterns emerge or rules change.
