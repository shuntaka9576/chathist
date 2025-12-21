# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                      # Build
cargo test                       # Run all tests
cargo test <test_name>           # Run single test
cargo test -- --nocapture        # Run tests with output
cargo clippy                     # Run linter
cargo run -- list                # Run list command
cargo run -- pick <id>           # Run pick command (opens in editor)
cargo run -- pick --stdout <id>  # Run pick command (output to stdout)
cargo run -- pick -p <id>        # Run pick command for plan file
cargo run -- config              # Open config in editor

# Stdin input is also supported
echo "<id>" | cargo run -- pick
```

### cargo-make (Development)

```bash
cargo make check                 # Run all checks (clippy, test, build)
cargo make pick                  # Interactive pick with fzf (includes preview)
```

Environment variable `CHATHIST_CONFIG_FILE_PATH="examples/config/config.lua"` is automatically set.

## Architecture

- **Agent trait** (`src/agent/mod.rs`): Abstraction for multi-agent support (`get_log_dir`, `list`, `pick`). `DisplayEntry` struct defines data flow.
- **ClaudeAgent** (`src/agent/claude/`): Claude Code implementation
  - `config.rs`: Log directory detection via `CLAUDE_CONFIG_DIR` or `~/.claude`
  - `parser.rs`: JSONL parsing with session/summary handling
  - `actions/`: `list.rs` and `pick.rs` implement Agent trait methods
- **Config** (`src/config/`): Lua config loader using mlua. Defaults to `~/.config/chathist/config.lua`
- **Templates** (`src/config/templates/`): Pick output via minijinja. Template receives `sessions[]` with `id`, `messages[]` (role/content)
- **Commands** (`src/commands/`): CLI handlers for `list`, `pick`, `config` subcommands

## Config Structure

```lua
local chathist = require("chathist")
local experimental = require("chathist.experimental")

return {
    editor = "vim",  -- Falls back to $EDITOR, then vim
    commands = {
        list = {
            template = "$session_id\t$title:50\t$relative_time:>15\t$message_count:>5",
        },
        pick = {
            template = {
                preset = {
                    standard = chathist.template.pick.standard,
                    collapsible = experimental.template.pick.collapsible,
                },
                default = "standard",
            },
        },
    },
}
```

Built-in templates: `chathist.template.pick.standard`, `experimental.template.pick.collapsible`

### List Template Variables
- `$var` / `$var:N` (left-aligned) / `$var:>N` (right-aligned)
- Available: `$session_id`, `$title`, `$time`, `$relative_time`, `$message_count`, `$branch`

### Pick Template Variables
- `sessions[]` with `id`, `messages[]` (role/content), `plan`

## Claude Code Log Format

Claude Code logs are stored in `$CLAUDE_CONFIG_DIR/projects/<encoded-cwd>/` (default: `~/.claude/projects/<encoded-cwd>/`).
- Session files: `<UUID>.jsonl`
- Agent files: `agent-<ID>.jsonl` (excluded)

JSONL line structure:
```jsonc
{
  "type": "user" | "assistant" | "system" | "summary",
  "sessionId": "UUID",
  "message": { "role": "user" | "assistant", "content": "..." },
  "summary": "...",           // when type=summary
  "leafUuid": "message UUID"  // links summary to message
}
```
