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
      },
    },
  },
}
