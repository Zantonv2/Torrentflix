# Torrentflix MCP Servers

This document describes the Model Context Protocol (MCP) servers configured for the Torrentflix workspace via `.kiro/settings/mcp.json`.

It is intended for **AI agents and tool authors** who want to:

- Understand what each MCP server does.
- Know how to install / run each server on a developer machine.
- See how each server is configured for this repo.
- Use the tools safely when acting on this project.

Config file:

- Path: `.kiro/settings/mcp.json`

---

## 1. Overview of Configured Servers

| Key         | Upstream project                             | Status in Torrentflix | Scope / Notes                                   |
|------------|-----------------------------------------------|------------------------|-------------------------------------------------|
| `filesystem` | `philgei/mcp_server_filesystem` (Python)    | **Enabled**            | Filesystem access limited to this repo          |
| `git`      | `MementoRC/mcp-git` (Python)                  | **Enabled**            | Git operations for this repo only              |
| `sqlite`   | `mcp-server-sqlite` from `modelcontextprotocol/servers` | **Enabled**  | Direct access to project SQLite DB            |
| `shell`    | `sonirico/mcp-shell` (Go)                     | **Disabled by default** | System shell access (high‑risk)               |
| `browser`  | `BrowserMCP/mcp` (TS + Chrome extension)      | **Disabled**           | Browser automation                             |
| `code-assistant` | `stippi/code-assistant` (Rust)          | **Disabled**           | Standalone coding agent / MCP server           |
| `svelte`   | `freema/mcp-design-system-extractor` (TS)     | **Enabled**            | Storybook design system analysis (if present)  |
| `typescript` | `@mizchi/lsmcp` (TS)                        | **Enabled**            | TypeScript/Svelte LSP via MCP                  |
| `rust`     | `Vaiz/rust-mcp-server` (Rust)                 | **Enabled**            | Rust dev commands (cargo, rustup, etc.)        |
| `context7` | `@upstash/context7-mcp` (TS, cloud API)       | **Enabled**            | Cloud docs for libraries/frameworks            |

> **Note**
> Some enabled servers require external services or runtimes (e.g. Storybook, Chrome extension, OpenAI key). See each section below.

---

## 2. Filesystem MCP Server (`filesystem`)

- Upstream: <https://github.com/philgei/mcp_server_filesystem>
- Purpose: Read/write files and list directories inside the Torrentflix repo.

### 2.1. Install / Run

Torrentflix uses **`uvx`** to run the CLI `mcp-server-filesystem`:

- Prerequisite: Install **uv** from <https://docs.astral.sh/uv/>.
- Test that the CLI is available (uvx will download it on first run):

```bash
uvx mcp-server-filesystem --help
```

You typically do **not** run this manually; your MCP‑capable client (e.g. Windsurf, Claude Desktop) will spawn it using the config below.

### 2.2. Torrentflix Configuration

```jsonc
"filesystem": {
  "command": "uvx",
  "args": [
    "mcp-server-filesystem",
    "/home/trash/Coding_Projects/Torrentflix"
  ],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": [
    "list_allowed_directories",
    "read_file",
    "write_file",
    "list_directory"
  ]
}
```

- Scope: Access is limited to `/home/trash/Coding_Projects/Torrentflix`.

### 2.3. Key Tools (conceptual)

Exact tool names may vary by version, but Torrentflix auto‑approves these:

- **`list_allowed_directories`** – Show which directories the server can access.
- **`list_directory`** – List files inside an allowed directory.
- **`read_file`** – Read a file (text or binary) from the repo.
- **`write_file`** – Write or overwrite a file.

### 2.4. Usage Guidelines for Agents

- **Prefer narrow paths**: When reading/writing, use specific file paths instead of whole-directory operations.
- **Avoid destructive writes**: Do not overwrite large files or config files unless explicitly asked.
- **Pair with git**: For non‑trivial edits, stage/commit changes using the `git` server so changes are traceable.

---

## 3. Git MCP Server (`git`)

- Upstream: <https://github.com/MementoRC/mcp-git>
- Purpose: Inspect and manipulate the Git repo.

### 3.1. Install / Run

Using **uvx** (recommended by upstream):

```bash
# uvx will download and run the CLI
uvx mcp-server-git --help
```

Alternative (if needed):

```bash
pip install mcp-server-git
python -m mcp_server_git --help
```

### 3.2. Torrentflix Configuration

```jsonc
"git": {
  "command": "uvx",
  "args": [
    "mcp-server-git",
    "--repository",
    "/home/trash/Coding_Projects/Torrentflix"
  ],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": [
    "get_repository_status",
    "get_commit_history",
    "get_diff"
  ]
}
```

- Scope: Restricted to the Torrentflix repo.

### 3.3. Important Tools (from upstream README)

Names may differ slightly by client, but core tools include:

- **`git_status`** – Show working tree status.
- **`git_diff_unstaged`** – Show diffs for unstaged changes.
- **`git_diff_staged`** – Show diffs for staged changes.
- **`git_diff`** – Compare current state with a target branch/commit.
- **`git_log`** – Show recent commits (config usually limits count).
- **`git_add`** – Stage specific files.
- **`git_reset`** – Unstage changes.
- **`git_commit`** – Create a commit with a given message.
- **`git_create_branch`**, **`git_checkout`**, **`git_show`**, **`git_init`** – Branch, history, and repo initialization operations.

### 3.4. Usage Guidelines for Agents

- **Read‑only by default**: Prefer `git_status`, `git_diff_*`, and `git_log` unless explicitly asked to modify history.
- **Commit policy**:
  - Only call `git_add` / `git_commit` when the user has asked to save changes.
  - Use clear commit messages reflecting what changed.
- **Never rewrite history** (no forced pushes or history‑editing tools) unless clearly requested.

---

## 4. SQLite MCP Server (`sqlite`)

- Upstream: `mcp-server-sqlite` in <https://github.com/modelcontextprotocol/servers>
- Purpose: Query and analyze the Torrentflix SQLite database.

### 4.1. Install / Run

Using **uvx** (recommended pattern from official examples):

```bash
uvx mcp-server-sqlite --db-path ./test.db --help
```

Your MCP client will pass the real DB path from the config; you normally dont run this by hand.

### 4.2. Torrentflix Configuration

```jsonc
"sqlite": {
  "command": "uvx",
  "args": [
    "mcp-server-sqlite",
    "--db-path",
    "/home/trash/Coding_Projects/Torrentflix/sqlite_mcp_server.db"
  ],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": [
    "query",
    "analyze"
  ]
}
```

- DB file: `/home/trash/Coding_Projects/Torrentflix/sqlite_mcp_server.db`

### 4.3. Likely Tools

Based on the official SQLite MCP server, you can expect tools along the lines of:

- **`query`** – Run SQL queries and return rows.
- **`analyze`** – Inspect schema or query patterns to provide analysis/insights.

> Exact tool signatures depend on the current released version; your MCP client will advertise the detailed JSON schema.

### 4.4. Usage Guidelines for Agents

- **Favor read‑only queries**: Use `SELECT` and introspection queries; avoid `INSERT`/`UPDATE`/`DELETE` unless the user explicitly asks for mutations.
- **Be explicit**: Include fully qualified table names and clear `WHERE` clauses.
- **No schema changes**: Dont drop/create/alter tables without explicit, high‑confidence instructions.

---

## 5. Shell MCP Server (`shell`)

- Upstream: <https://github.com/sonirico/mcp-shell>
- Purpose: Execute shell commands via MCP in a controlled, audited way.
- **Status in Torrentflix: disabled by default** (`disabled: true`).

### 5.1. Install / Run

Upstream supports running as a standalone binary or via Docker. In this project the config assumes **`uvx`**:

```bash
uvx sonirico/mcp-shell@latest --help
```

Refer to the upstream README for Docker and security‑config options.

### 5.2. Torrentflix Configuration

```jsonc
"shell": {
  "command": "uvx",
  "args": ["sonirico/mcp-shell@latest"],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": true,
  "autoApprove": []
}
```

### 5.3. Tool Shape (from README)

The shell tool accepts parameters like:

- **`command`** (string, required): Shell command to execute.
- **`base64`** (boolean, optional): Whether to base64‑encode stdout/stderr.

Responses look like:

```jsonc
{
  "status": "success | error",
  "exit_code": 0,
  "stdout": "command output",
  "stderr": "error output",
  "command": "executed command",
  "execution_time": "100ms",
  "security_info": {
    "security_enabled": true,
    "working_dir": "/tmp/mcp-workspace",
    "timeout_applied": true
  }
}
```

### 5.4. Usage Guidelines for Agents

- **Only use if explicitly enabled** (`disabled` must be set to `false` by the user).
- Prefer **read‑only** and **idempotent** commands (`ls`, `cat`, `grep`) over anything that mutates the system.
- Avoid `rm`, `sudo`, package managers, or long‑running daemons unless there is clear user intent and constraints.

---

## 6. Browser MCP Server (`browser`)

- Upstream: <https://github.com/BrowserMCP/mcp>
- Purpose: Control a local Chrome browser (navigate, click, fill forms, etc.).
- **Status in Torrentflix: disabled**.

### 6.1. Notes for Agents

- This server is **off by default**; treat it as unavailable unless the user enables it.
- When enabled, it provides tools such as navigation and interaction commands (e.g. navigate, click, type, select); see upstream README for exact names and parameters.
- Use cautiously and keep actions reproducible (log URLs and form fields you act on).

---

## 7. Code Assistant MCP Server (`code-assistant`)

- Upstream: <https://github.com/stippi/code-assistant>
- Purpose: A full autonomous coding assistant that itself exposes tools like file edits and command execution.
- **Status in Torrentflix: disabled**.

### 7.1. Notes for Agents

- When enabled, this server effectively gives you another coding agent as a tool.
- Use this only in environments designed for nested agents; in Torrentflix it remains off to avoid conflicts with the primary assistant.

For installation and detailed tools, see the upstream README; it supports a GUI, terminal mode, and MCP server mode.

---

## 8. Svelte / Design System MCP Server (`svelte`)

- Upstream: <https://github.com/freema/mcp-design-system-extractor>
- Purpose: Let an AI assistant analyze a Storybook‑based design system.
- **Status in Torrentflix: enabled**, but only useful if you have a Storybook instance running.

### 8.1. Install / Run

From upstream quick start:

```bash
npm install
npm run build
npm run setup   # interactive Claude/MCP setup
# or set STORYBOOK_URL=http://localhost:6006
```

Torrentflix runs it via `uvx freema/mcp-design-system-extractor@latest`; see upstream docs for full setup.

### 8.2. Torrentflix Configuration

```jsonc
"svelte": {
  "command": "uvx",
  "args": ["freema/mcp-design-system-extractor@latest"],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": []
}
```

### 8.3. Key Tools (from README)

Core tools include (summarized):

- **`list_components`** – List all Storybook components with categories; supports pagination.
- **`search_components`** – Search components by name/title/category.
- **`get_component_html`** – Fetch rendered HTML for a specific story (`component-name--story-name`).
- **`get_component_variants`** – List all story variants for a component.
- **`get_component_props`** – Extract props / argTypes for a given story.
- **`get_component_dependencies`** – Analyze which other components a component uses.
- **`get_layout_components`** – Get layout components plus usage examples.
- **`get_theme_info`** – Extract theme tokens (colors, spacing, typography, breakpoints).

### 8.4. Usage Guidelines for Agents

- Use this server **only** when the user is working with a Storybook design system.
- Start with `list_components` / `search_components`, then drill into `get_component_html`, `get_component_props`, and `get_theme_info`.
- Dont assume Storybook is running; handle connection errors gracefully.

---

## 9. TypeScript / Svelte LSP MCP Server (`typescript`)

- Upstream: <https://github.com/mizchi/lsmcp>
- Purpose: Expose the TypeScript (and related) language server via MCP so the agent can query symbols, diagnostics, references, etc.

### 9.1. Install / Run

From upstream Quick Start (simplified):

```bash
# As a dev dependency (optional)
npm add -D @mizchi/lsmcp @typescript/native-preview

# MCP client will then run something like:
npx -y @mizchi/lsmcp -p tsgo
```

Torrentflix uses the `npx` pattern directly in the MCP config.

### 9.2. Torrentflix Configuration

```jsonc
"typescript": {
  "command": "npx",
  "args": [
    "-y",
    "@mizchi/lsmcp",
    "-p",
    "tsgo"
  ],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": []
}
```

### 9.3. Capabilities

The exact tool names are defined by `lsmcp`, but they typically wrap the Language Server Protocol (LSP) capabilities:

- Listing diagnostics for files.
- Finding symbol definitions and references.
- Navigating project structure.
- Getting hover info / type info for expressions.

### 9.4. Usage Guidelines for Agents

- Use this server whenever you need **precise** information about the Svelte/TypeScript UI code (types, errors, references).
- Prefer LSP queries over guessing types or project structure from raw text.

---

## 10. Rust MCP Server (`rust`)

- Upstream: <https://github.com/Vaiz/rust-mcp-server>
- Purpose: Give the agent controlled access to `cargo`, `rustup`, and related tools for the Rust workspace.

### 10.1. Install / Run

Install via Cargo:

```bash
cargo install rust-mcp-server
rust-mcp-server --help
```

Ensure `$HOME/.cargo/bin` (or equivalent) is on your `PATH` so the CLI is discoverable.

### 10.2. Torrentflix Configuration

```jsonc
"rust": {
  "command": "rust-mcp-server",
  "args": [
    "--workspace",
    "/home/trash/Coding_Projects/Torrentflix"
  ],
  "env": {
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": []
}
```

### 10.3. Key Tools (from upstream `tools.md`)

**Core Cargo commands**

- `cargo-build` – Build the workspace.
- `cargo-check` – Type‑check without building.
- `cargo-test` – Run tests.
- `cargo-doc` – Build docs (usually with `--no-deps`).
- `cargo-fmt` – Format code.
- `cargo-clippy` – Run clippy lints.
- `cargo-clean` – Clean `target/`.

**Project management**

- `cargo-new`, `cargo-generate_lockfile`, `cargo-package`, `cargo-list`.

**Dependency management**

- `cargo-add`, `cargo-remove`, `cargo-update`, `cargo-metadata`, `cargo-search`, `cargo-info`.

**Code quality & security**

- `cargo-deny-*`, `cargo-machete*`, `cargo-hack*` helpers.

**Toolchain**

- `rustc-explain`, `rustup-show`, `rustup-toolchain-add`, `rustup-update`.

### 10.4. Usage Guidelines for Agents

- Prefer **read‑only / analysis** first: `cargo-check`, `cargo-test`, `cargo-clippy` before `cargo-build`.
- Avoid creating new crates or changing dependencies (`cargo-new`, `cargo-add`, etc.) unless explicitly instructed.
- Keep invocations scoped to this workspace (Torrentflix config enforces `--workspace`).

---

## 11. Context7 MCP Server (`context7`)

- Upstream: <https://github.com/upstash/context7>
- Purpose: Give the agent **up-to-date documentation and examples for popular libraries/frameworks** (React, Svelte, Node/TS libs, etc.) via Context7s cloud API.

### 11.1. Install / Run

Torrentflix runs Context7 using `npx`:

```bash
npx -y @upstash/context7-mcp --help
```

Your MCP client (Kiro) will invoke this using the `command`/`args` from the config; you don't need to run it manually.

### 11.2. Torrentflix Configuration

```jsonc
"context7": {
  "command": "npx",
  "args": [
    "-y",
    "@upstash/context7-mcp"
  ],
  "env": {
    "CONTEXT7_API_KEY": "ctx7sk-dff56983-6957-4778-89af-a2dfcab18120",
    "FASTMCP_LOG_LEVEL": "ERROR"
  },
  "disabled": false,
  "autoApprove": [
    "search_documentation",
    "get_library_docs",
    "get_code_examples",
    "get_api_reference",
    "get_best_practices",
    "list_supported_libraries",
    "get_changelog"
  ]
}
```

> This project is local and `.kiro` is intended to be gitignored, so the API key is stored directly in the config for convenience.

### 11.3. Key Tools (from upstream)

- **`search_documentation`**
  - Search docs across supported libraries.
  - Params: `query`, optional `language`, `framework`, `limit`.

- **`get_library_docs`**
  - Fetch docs for a specific library and optional `version` / `topic`.

- **`get_code_examples`**
  - Code examples for a given `library` and `use_case`.

- **`explain_code`**
  - Explain a code snippet (`code`) with optional `language` and `context`.

- **`get_api_reference`**
  - Detailed API reference for `library` + `api_name` (and optional `version`).

- **`compare_libraries`**
  - Compare multiple libraries for a `use_case`.

- **`get_migration_guide`**
  - Migration guide between libraries/versions.

- **`get_best_practices`**
  - Best practices for a given `library` (and optional `topic`).

- **`troubleshoot_error`**
  - Help with `error_message` plus optional `library` and `code_context`.

- **`list_supported_libraries`**
  - List libraries, filterable by `language` and `category`.

- **`get_changelog`**
  - Changelogs for a `library` between `from_version` and `to_version`.

### 11.4. Usage Guidelines for Agents

- Use Context7 for **library/framework questions**, not for Torrentflix-specific business logic.
- Prefer it over guessing when you need authoritative info on:
  - React/Svelte/TS/Node/Rust libraries.
  - Breaking changes, migration paths, or best practices.
- Combine it with `rust-mcp-server` / `lsmcp`:
  - Ask Context7 for docs or examples.
  - Use `rust` / `typescript` servers for workspace-aware checks and edits.

---
## 12. Optional: Rust Docs MCP Server (Not Yet in Config)

> This server is **not** currently configured in `.kiro/settings/mcp.json`, but it is highly recommended for Rust work.

- Upstream: <https://github.com/Govcraft/rust-docs-mcp-server>
- Purpose: Answer questions about **specific Rust crates** using up‑to‑date docs (e.g. `serde`, `tokio`, `reqwest`, `sqlx`).

### 11.1. Install / Requirements

- Requires an **OpenAI API key** in `OPENAI_API_KEY`.
- Install from GitHub releases (prebuilt binaries) or build from source.

### 11.2. Main Tool (from README)

- **`query_rust_docs`**
  - Input JSON schema:

    ```jsonc
    {
      "type": "object",
      "properties": {
        "question": {
          "type": "string",
          "description": "The specific question about the crate's API or usage."
        }
      },
      "required": ["question"]
    }
    ```

  - Output: Answer text like:

    ```text
    From <crate_name> docs: ...
    ```

- **Resources**
  - `crate://<crate_name>` – metadata about which crate this server instance covers (e.g. `crate://reqwest`).

### 11.3. Usage Guidelines for Agents

- Call `query_rust_docs` **before** writing code for crates like `reqwest`, `tokio`, `sqlx`, etc., to avoid using outdated APIs.
- Treat responses as authoritative for the targeted crate.

---

## 13. How Agents Should Approach MCP in Torrentflix

1. **Discover capabilities**
   - Use your MCP clients built‑in discovery to list servers and tools.
   - Cross‑check with this document to understand scope and safety.

2. **Prefer read‑only operations first**
   - Use `filesystem` + `git` + `sqlite` + `typescript` + `rust` for analysis before making changes.

3. **Be explicit and reversible**
   - Keep file edits small and organized.
   - Use `git` to stage/commit when the user requests persistent changes.

4. **Ask before dangerous actions**
   - Shell commands, schema changes, large refactors, and dependency updates should always be confirmed by the user.

This document should be kept in sync with `.kiro/settings/mcp.json` whenever the MCP configuration changes.
