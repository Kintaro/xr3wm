use layout::*;
use keycode::*;
use std::default::Default;
use std::iter::range;
use workspaces::WorkspaceConfig;
use commands::{Cmd, ManageHook, CmdManage, LogHook, LogInfo, CmdLogHook};

include!(concat!(env!("HOME"), "/.xr3wm/config.rs"));

pub struct Keybinding {
  pub mods: u8,
  pub key: String,
  pub cmd: Cmd
}

pub struct Config<'a> {
  pub workspaces: Vec<WorkspaceConfig<'a>>,
  pub mod_key: u8,
  pub border_width: u32,
  pub border_color: u32,
  pub border_focus_color: u32,
  pub border_urgent_color: u32,
  pub greedy_view: bool,
  pub keybindings: Vec<Keybinding>,
  pub manage_hooks: Vec<ManageHook>,
  pub log_hook: Option<LogHook<'a>>
}

impl<'a> Default for Config<'a> {
  fn default() -> Config<'a> {
    let mut config = Config {
      workspaces: range(1us, 10).map(|idx| WorkspaceConfig { tag: idx.to_string(), screen: 0, layout: HSplitLayout::new()/*TallLayout::new(1, 0.5, 0.05)*/ }).collect(),
      mod_key: MOD_4,
      border_width: 2,
      border_color: 0x002e2e2e,
      border_focus_color: 0x002a82e6,
      border_urgent_color: 0x00ff0000,
      greedy_view: false,
      keybindings: vec![
        Keybinding {
          mods: 0,
          key: String::from_str("Return"),
          cmd: Cmd::Exec(String::from_str("xterm -u8"))
        },
        Keybinding {
          mods: 0,
          key: String::from_str("d"),
          cmd: Cmd::Exec(String::from_str("dmenu_run"))
        },
        Keybinding {
          mods: 0,
          key: String::from_str("a"),
          cmd: Cmd::NestContainer(0)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("s"),
          cmd: Cmd::NestContainer(1)
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("q"),
          cmd: Cmd::KillClient
        },
        Keybinding {
          mods: 0,
          key: String::from_str("Left"),
          cmd: Cmd::MoveFocus(MoveOp::Left)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("Right"),
          cmd: Cmd::MoveFocus(MoveOp::Right)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("Up"),
          cmd: Cmd::MoveFocus(MoveOp::Up)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("Down"),
          cmd: Cmd::MoveFocus(MoveOp::Down)
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("Left"),
          cmd: Cmd::MoveWindow(MoveOp::Left)
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("Right"),
          cmd: Cmd::MoveWindow(MoveOp::Right)
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("Up"),
          cmd: Cmd::MoveWindow(MoveOp::Up)
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("Down"),
          cmd: Cmd::MoveWindow(MoveOp::Down)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("comma"),
          cmd: Cmd::SendLayoutMsg(LayoutMsg::IncreaseMaster)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("period"),
          cmd: Cmd::SendLayoutMsg(LayoutMsg::DecreaseMaster)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("l"),
          cmd: Cmd::SendLayoutMsg(LayoutMsg::Increase)
        },
        Keybinding {
          mods: 0,
          key: String::from_str("h"),
          cmd: Cmd::SendLayoutMsg(LayoutMsg::Decrease)
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("c"),
          cmd: Cmd::Exit
        },
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str("x"),
          cmd: Cmd::Reload
        }
      ],
      manage_hooks: Vec::new(),
      log_hook: None
    };

    for i in range(1us, 10) {
      config.keybindings.push(
        Keybinding {
          mods: 0,
          key: i.to_string(),
          cmd: Cmd::SwitchWorkspace(i)
        });

      config.keybindings.push(
        Keybinding {
          mods: MOD_SHIFT,
          key: i.to_string(),
          cmd: Cmd::MoveToWorkspace(i)
        });
    }

    for &(i, key) in vec![(1, "w"), (2, "e"), (3, "r")].iter() {
      config.keybindings.push(
        Keybinding {
          mods: 0,
          key: String::from_str(key),
          cmd: Cmd::SwitchScreen(i)
        });

      config.keybindings.push(
        Keybinding {
          mods: MOD_SHIFT,
          key: String::from_str(key),
          cmd: Cmd::MoveToScreen(i)
        });
    }

    config
  }
}
