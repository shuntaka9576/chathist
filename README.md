# chathist

A lightweight CLI tool to view and export your AI coding agent's chat history. (Currently supporting Claude Code.)

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

### Zsh Integration

Add this function to your .zshrc for a powerful session picker with live previews.

* Tab: Multi-select sessions
* Shift + Up/Down (or Scroll): Scroll through the preview window

```bash
chp() {
  while true; do
    selection=$(chathist list | fzf --multi --with-nth=2.. \
      --preview 'chathist pick {1} --stdout' \
      --preview-window 'right:45%:wrap' | cut -f1)

    [ -z "$selection" ] && break

    echo "$selection" | chathist pick
  done
}
```

Alternatively, you can set up a keybinding to invoke chathist directly with `Ctrl+H`.

```bash
function chathist-widget() {
  while true; do
    local selection=$(chathist list | fzf-tmux --multi --with-nth=2.. \
      --preview 'chathist pick {1} --stdout' \
      --preview-window 'right:45%:wrap' | cut -f1)

    [ -z "$selection" ] && break

    echo "$selection" | chathist pick
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

#### Example: Copy to Clipboard

```bash
# Initialize the configuration and open it in your editor to register template presets (standard, collapsible).
chathist config

# Select a session via fzf and copy to clipboard using the collapsible format.
chathist list | fzf --multi --with-nth=2.. \
  --preview 'chathist pick -t standard {1} --stdout' \
  --preview-window 'right:45%:wrap' \
  | cut -f1 | chathist pick -t collapsible --stdout | pbcopy
```

## Configuration

Chathist looks for its configuration file at `~/.config/chathist/config.lua` by default. Run `chathist config` to open the file. If you want to use a different location, set the `CHATHIST_CONFIG_FILE_PATH` environment variable.

* The app looks for an editor in this order: config.editor, $EDITOR, then vim.
* Claude logs default to the following directories: $CLAUDE_CONFIG_DIR/projects/ or ~/.claude/projects/.

The Lua table below outlines the internal defaults. To customize your setup, simply override these values in your `config.lua`.

```lua
local chathist = require("chathist")
local experimental = require("chathist.experimental")

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
          collapsible = experimental.template.pick.collapsible,
        },
        default = "standard",
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


#### Template Examples

| Template | Description |
|----------|-------------|
| [standard](src/config/templates/pick/standard.j2) | Default template |
| [collapsible](src/config/templates/pick/collapsible.j2) | Using HTML `<details>` tag (experimental) |

Use Jinja2 conditionals to filter messages by role.

```jinja2
{% for message in session.messages %}
{% if message.role == "user" %}
## User

{{ message.content }}

{% endif %}
{% endfor %}
```
