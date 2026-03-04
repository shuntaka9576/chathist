# chathist

A lightweight CLI tool to view and export your AI coding agent's chat history—currently optimized for Claude Code.

Built for speed and flexibility, chathist hooks into fzf / fzf-tmux to let you breeze through past sessions with instant previews. It’s fully customizable via Lua configuration and Jinja2 templates, so you can tailor the output to your exact workflow.

![img](asset/chathist-demo.gif)

## Installation

### Brew (MacOS)

```bash
brew install shuntaka9576/tap/chathist
```

### Cargo (git)
```bash
git clone https://github.com/shuntaka9576/chathist
cd chathist
cargo install --path .
```

## Getting Started

`chathist` is designed to pair perfectly with `fzf`.

`chathist list` appends normalized conversation text as an extra tab-separated field, so you can search by message body with `fzf` while keeping the session ID in the first column for previews and selection.

### Zsh Integration

Add this function to your .zshrc for a powerful session picker with live previews.

* Tab: Multi-select sessions
* Shift + Up/Down (or Scroll): Scroll through the preview window

```bash
chp() {
  while true; do
    selection=$(chathist list | fzf --multi \
      --delimiter=$'\t' \
      --with-nth=2.. \
      --preview 'chathist pick {1} --stdout' \
      --preview-window 'right:45%:wrap' | cut -f1)

    [ -z "$selection" ] && break

    template=$(chathist pick --list-templates | fzf --prompt="Select template: ")
    [ -z "$template" ] && continue

    echo "$selection" | chathist pick -t "$template"
  done
}
```

Template-first selection
```diff
+ template=$(chathist pick --list-templates | fzf --prompt="Select template: ")
+ [ -z "$template" ] && return
  while true; do
    ...
-   template=$(chathist pick --list-templates | fzf --prompt="Select template: ")
-   [ -z "$template" ] && continue
```
<details>
<summary>full version</summary>

```bash
chp() {
  while true; do
    selection=$(chathist list | fzf --multi \
      --delimiter=$'\t' \
      --with-nth=2.. \
      --preview 'chathist pick {1} --stdout' \
      --preview-window 'right:45%:wrap' | cut -f1)

    [ -z "$selection" ] && break

    echo "$selection" | chathist pick
  done
}
```

</details>

Skip template selection
```diff
-   template=$(chathist pick --list-templates | fzf --prompt="Select template: ")
-   [ -z "$template" ] && continue
-   echo "$selection" | chathist pick -t "$template"
+   echo "$selection" | chathist pick
```

<details>
<summary>full version</summary>

```bash
chp() {
  template=$(chathist pick --list-templates | fzf --prompt="Select template: ")
  [ -z "$template" ] && return

  while true; do
    selection=$(chathist list | fzf --multi \
      --delimiter=$'\t' \
      --with-nth=2.. \
      --preview "chathist pick -t $template {1} --stdout" \
      --preview-window 'right:45%:wrap' | cut -f1)

    [ -z "$selection" ] && break

    echo "$selection" | chathist pick -t "$template"
  done
}
```

</details>

Alternatively, you can set up a keybinding to invoke chathist directly with `Ctrl+H`.

```bash
function chathist-widget() {
  while true; do
    local selection=$(chathist list | fzf-tmux --multi \
      --delimiter=$'\t' \
      --with-nth=2.. \
      --preview 'chathist pick {1} --stdout' \
      --preview-window 'right:45%:wrap' | cut -f1)

    [ -z "$selection" ] && break

    local template=$(chathist pick --list-templates | fzf-tmux --prompt="Select template: ")
    [ -z "$template" ] && continue

    echo "$selection" | chathist pick -t "$template"
  done

  zle reset-prompt
}

zle -N chathist-widget
bindkey "^h" chathist-widget
```

### Core Commands

* `chathist list`: List all chat sessions.
* `chathist pick <session_id>`: Opens the specified session in the editor.
* `chathist pick --stdout <session_id>`: Dump content to terminal (ideal for `fzf` previews).
* `chathist pick --template <name> <session_id>`: Use a predefined template.
* `chathist pick --list-templates`: List available template names (for fzf integration).

#### Example: Copy to Clipboard

```bash
# Select a session via fzf and copy to clipboard using the github format.
chathist list | fzf --multi \
  --delimiter=$'\t' \
  --with-nth=2.. \
  --preview 'chathist pick -t standard {1} --stdout' \
  --preview-window 'right:45%:wrap' \
  | cut -f1 | chathist pick -t github --stdout | pbcopy
```

## Configuration

Chathist looks for its configuration file at `~/.config/chathist/config.lua` by default. Run `chathist config` to open the file. If you want to use a different location, set the `CHATHIST_CONFIG_FILE_PATH` environment variable.

* The app looks for an editor in this order: config.editor, $EDITOR, then vim.
* Claude logs default to the following directories: $CLAUDE_CONFIG_DIR/projects/ or ~/.claude/projects/.

The Lua table below outlines the internal defaults. To customize your setup, simply override these values in your `config.lua`.

```lua
local chathist = require("chathist")

return {
  -- editor = "vim",  -- Uses $EDITOR or vim if not set
  commands = {
    list = {
      template = "$session_id\t$title:50\t$relative_time:>15\t$message_count:>5",
    },
    pick = {
      template = {
        preset = {
          standard = chathist.template.pick.standard,
          github = chathist.template.pick.github,
          ["github-compact"] = chathist.template.pick.github_compact,
          slack = chathist.template.pick.slack,
        },
        default = "standard",
        -- list_hidden = { "github-compact" },  -- Hide from --list-templates output
      },
    },
  },
}
```

### commands.list

Customize output by setting `commands.list.template` in your config.

#### Syntax

| Format | Description |
|--------|-------------|
| `$var` | Expand variable |
| `$var:N` | Expand with width N (left-aligned, truncated if longer) |
| `$var:>N` | Expand with width N (right-aligned) |

#### Available Variables

| Variable | Description |
|----------|-------------|
| `$session_id` | Session ID |
| `$title` | Session title (first message or summary) |
| `$time` | Timestamp |
| `$relative_time` | Relative time (e.g., "2 hours ago") |
| `$message_count` | Number of messages in the session |
| `$branch` | Git branch |

### commands.pick

Templates use Jinja2 syntax (via minijinja). Customize output by setting `commands.pick.template` in your config.

#### Available Variables

| Variable | Description |
|----------|-------------|
| `sessions` | Array of sessions |
| `sessions[].id` | Session ID |
| `sessions[].messages` | Array of messages |
| `sessions[].messages[].role` | `"user"` or `"assistant"` |
| `sessions[].messages[].content` | Message content |
| `sessions[].plan` | Plan content (if exists) |


#### Available Filters

| Filter | Description |
|--------|-------------|
| `title` | Capitalize first letter (e.g., `"user"` → `"User"`) |
| `truncate(length=N)` | Truncate string with `...` suffix |
| `replace` | String replacement |

#### Template Examples

| Template | Description |
|----------|-------------|
| [standard](src/config/templates/pick/standard.j2) | Plain Markdown |
| [github](src/config/templates/pick/github.j2) | Markdown wrapped in `<details>` for GitHub |
| [github-compact](src/config/templates/pick/github_compact.j2) | Fully nested `<details>` for GitHub |
| [slack](src/config/templates/pick/slack.j2) | Slack mrkdwn format (paste directly into Slack) |

Use Jinja2 conditionals to filter messages by role.

```jinja2
{% for message in session.messages %}
{% if message.role == "user" %}
## User

{{ message.content }}

{% endif %}
{% endfor %}
```
