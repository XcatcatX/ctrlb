use crate::settings::{SETTING_LIST, Settings};

struct CommandInfo {
    name: &'static str,
    description: &'static str,
    example: &'static str,
}

const COMMAND_LIST: &[CommandInfo] = &[
    CommandInfo {
        name: "help",
        description: "Show this help.",
        example: "help",
    },
    CommandInfo {
        name: "settings",
        description: "Show all settings and their values.",
        example: "settings",
    },
    CommandInfo {
        name: "set <name> <value>",
        description: "Change a setting. The change is saved.",
        example: "set port 9000",
    },
    CommandInfo {
        name: "status",
        description: "Show the state of the server and the tunnel.",
        example: "status",
    },
    CommandInfo {
        name: "link",
        description: "Show the public link.",
        example: "link",
    },
    CommandInfo {
        name: "restart",
        description: "Restart the server and the tunnel.",
        example: "restart",
    },
    CommandInfo {
        name: "tray",
        description: "Close the terminal. The program keeps running in the tray.",
        example: "tray",
    },
    CommandInfo {
        name: "stop",
        description: "Stop everything and close the program.",
        example: "stop",
    },
];

pub fn build_help(settings: &Settings) -> String {
    let mut text = String::from("Commands:\n");
    for command in COMMAND_LIST {
        text.push_str(&format!(
            "  {}\n    {}\n    Example: {}\n",
            command.name, command.description, command.example
        ));
    }
    text.push_str("\nSettings:\n");
    for setting in SETTING_LIST {
        let applies = if setting.needs_restart {
            "after \"restart\""
        } else {
            "immediately"
        };
        text.push_str(&format!(
            "  {}\n    {}\n    Current value: {}\n    Example: {}\n    Change applies: {}\n",
            setting.name,
            setting.description,
            settings.display_value(setting.name),
            setting.example,
            applies
        ));
    }
    text
}
