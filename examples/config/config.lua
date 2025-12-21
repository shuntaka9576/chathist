-- Built-in templates
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
