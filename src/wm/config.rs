//! # Configuration module for gabelstaplerwm
//! This module is intended to be edited by the user in order to customize
//! the software and adjust it to his or her needs. You can edit the enums
//! freely, but you need to keep the functions already declared, as well as
//! trait instances which are derived or implemented, though you can change
//! the implementations of the `Default` trait. All other edits are welcome
//! as well, but chances are you 
//!
//! * don't need them to customize your wm.
//! * should consider contributing your changes back instead, as it seems to be
//!   a more involved and complex feature that you are working on.
//!
//! But feel free to do otherwise if you wish.
use std::env::home_dir;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::process::{Command, Stdio};

use wm::client::{TagSet, TagStack, ClientSet, current_tagset};
use wm::kbd::*;

use wm::layout::{ScreenSize,LayoutMessage};
use wm::layout::monocle::Monocle;
use wm::layout::stack::{HStack,VStack};

use wm::window_system::{Wm, WmConfig, WmCommand};

/// All tags used by `gabelstaplerwm`
///
/// Tags are symbolic identifiers by which you can classify your clients.
/// Each window has one or more tags, and you can display zero or more tags.
/// This means that all windows having at least one of the tags of the
/// *tagset* to be displayed attached get displayed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tag {
    /// the web tag - for browsers and stuff
    Web,
    /// the bookmarks tag
    Marks,
    /// "unlimited" number of work tags
    Work(u8),
    /// the media tag - movies, music apps etc. go here
    Media,
    /// the chat tag - for IRC and IM
    Chat,
    /// the log tag - for log viewing
    Logs,
    /// the monitoring tag - for htop & co.
    Mon,
}

impl Default for Tag {
    fn default() -> Tag {
        Tag::Work(0)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Tag::Work(n) = *self {
            write!(f, "work/{}", n)
        } else {
            write!(f, "{}", match *self {
                Tag::Web => "web",
                Tag::Marks => "marks",
                Tag::Media => "media",
                Tag::Chat => "chat",
                Tag::Logs => "logs",
                Tag::Mon => "mon",
                _ => "",
            })
        }
    }
}

/// All keyboard modes used by `gabelstaplerwm`-
///
/// A mode represents the active set of keybindings and/or their functionality.
/// This works like the vim editor: different keybindings get triggered in
/// different modes, even if the same keys are pressed.
///
/// # Limitations
/// Be aware that currently, `gabelstaplerwm` grabs the key combinations
/// globally during setup. This allows for overlapping keybindings in different
/// modes, but passing a key combination once grabbed to apps depending on mode
/// is currently impossible. This can be lifted in the future, if we decide to
/// regrab on every mode change, but that's a rather expensive operation, given
/// the currrent, `HashMap`-based design.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// normal mode doing normal stuff
    Normal,
    /// toggle tag on client mode
    Toggle,
    /// move client to tag mode
    Move,
    /// toggle tag on tagset mode
    Setup,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Normal
    }
}

/// Generate a window manager config - colors, border width...
///
/// Here you can specify (or compute) the settings you want to have.
/// See the docs for `ScreenSize` for more information.
pub fn generate_config() -> WmConfig {
    WmConfig {
        f_color: (0x0000, 0x5555, 0x7777), // this is #005577 (dwm cyan)
        u_color: (0x0000, 0x0000, 0x0000), // and this is #000000 (black)
        border_width: 1,
        screen: ScreenSize {
            offset_x: 0,
            offset_y: 20,
            width: 1366,
            height: 768,
        },
    }
}

/// Setup datastructures for the window manager.
///
/// This includes keybindings, default tag stack and matching.
pub fn setup_wm(wm: &mut Wm) {
    // keybindings
    let modkey = MOD4;
    wm.setup_bindings(vec![
        // focus n'th-tagset - modkey+[1-9]
        bind!(10, modkey, Mode::Normal, push_tagset!(0;; current_tagset)),
        bind!(11, modkey, Mode::Normal, push_tagset!(1;; current_tagset)),
        bind!(12, modkey, Mode::Normal, push_tagset!(2;; current_tagset)),
        bind!(13, modkey, Mode::Normal, push_tagset!(3;; current_tagset)),
        bind!(14, modkey, Mode::Normal, push_tagset!(4;; current_tagset)),
        bind!(15, modkey, Mode::Normal, push_tagset!(5;; current_tagset)),
        bind!(16, modkey, Mode::Normal, push_tagset!(6;; current_tagset)),
        bind!(17, modkey, Mode::Normal, push_tagset!(7;; current_tagset)),
        bind!(18, modkey, Mode::Normal, push_tagset!(8;; current_tagset)),
        // toggle tags on current client - modkey+[1-6]
        bind!(10, modkey, Mode::Toggle, toggle_tag!(Tag::Web)),
        bind!(11, modkey, Mode::Toggle, toggle_tag!(Tag::Marks)),
        bind!(12, modkey, Mode::Toggle, toggle_tag!(Tag::Chat)),
        bind!(13, modkey, Mode::Toggle, toggle_tag!(Tag::Media)),
        bind!(14, modkey, Mode::Toggle, toggle_tag!(Tag::Logs)),
        bind!(15, modkey, Mode::Toggle, toggle_tag!(Tag::Mon)),
        // move client to tags - modkey+[1-6]
        bind!(10, modkey, Mode::Move, move_to_tag!(Tag::Web)),
        bind!(11, modkey, Mode::Move, move_to_tag!(Tag::Marks)),
        bind!(12, modkey, Mode::Move, move_to_tag!(Tag::Chat)),
        bind!(13, modkey, Mode::Move, move_to_tag!(Tag::Media)),
        bind!(14, modkey, Mode::Move, move_to_tag!(Tag::Logs)),
        bind!(15, modkey, Mode::Move, move_to_tag!(Tag::Mon)),
        // toggle tags on current tagset - modkey+[1-6]
        bind!(10, modkey, Mode::Setup,
              toggle_show_tag!(Tag::Web;; current_tagset)),
        bind!(11, modkey, Mode::Setup,
              toggle_show_tag!(Tag::Marks;; current_tagset)),
        bind!(12, modkey, Mode::Setup,
              toggle_show_tag!(Tag::Chat;; current_tagset)),
        bind!(13, modkey, Mode::Setup,
              toggle_show_tag!(Tag::Media;; current_tagset)),
        bind!(14, modkey, Mode::Setup,
              toggle_show_tag!(Tag::Logs;; current_tagset)),
        bind!(15, modkey, Mode::Setup,
              toggle_show_tag!(Tag::Mon;; current_tagset)),
        // quit the window manager - modkey+CTRL+q
        bind!(24, modkey+CTRL, Mode::Normal, |_, _| {
            let _ = Command::new("killall")
                .arg("lemonbar")
                .spawn();
            WmCommand::Quit
        }),
        // spawn alarm/reminder notification with a delay - modkey+q
        bind!(24, modkey, Mode::Normal, |_, _| exec_script("alarm.zsh", &[])),
        // spawn custom dmenu - modkey+w
        bind!(25, modkey, Mode::Normal, |_, _| exec_script("menu.sh", &[])),
        // spawn dmenu_run - modkey+SHIFT-w
        bind!(25, modkey+SHIFT, Mode::Normal, |_, _|
              exec_command("dmenu_run", &["-y", "20"])),
        // spawn password manager script for dmenu - modkey+e
        bind!(26, modkey, Mode::Normal, |_, _| exec_script("pass.sh", &[])),
        // switch to normal mode - modkey+r
        bind!(27, modkey, Mode::Toggle, |_, _| {
            write_mode("NORMAL");
            WmCommand::ModeSwitch(Mode::Normal)
        }),
        bind!(27, modkey, Mode::Move, |_, _| {
            write_mode("NORMAL");
            WmCommand::ModeSwitch(Mode::Normal)
        }),
        bind!(27, modkey, Mode::Setup, |_, _| {
            write_mode("NORMAL");
            WmCommand::ModeSwitch(Mode::Normal)
        }),
        // switch to toggle mode - modkey+t
        bind!(28, modkey, Mode::Normal, |_, _| {
            write_mode("TOGGLE");
            WmCommand::ModeSwitch(Mode::Toggle)
        }),
        bind!(28, modkey, Mode::Move, |_, _| {
            write_mode("TOGGLE");
            WmCommand::ModeSwitch(Mode::Toggle)
        }),
        bind!(28, modkey, Mode::Setup, |_, _| {
            write_mode("TOGGLE");
            WmCommand::ModeSwitch(Mode::Toggle)
        }),
        // switch to move mode - modkey+z
        bind!(29, modkey, Mode::Normal, |_, _| {
            write_mode("MOVE");
            WmCommand::ModeSwitch(Mode::Move)
        }),
        bind!(29, modkey, Mode::Toggle, |_, _| {
            write_mode("MOVE");
            WmCommand::ModeSwitch(Mode::Move)
        }),
        bind!(29, modkey, Mode::Setup, |_, _| {
            write_mode("MOVE");
            WmCommand::ModeSwitch(Mode::Move)
        }),
        // switch to setup mode - modkey+u
        bind!(30, modkey, Mode::Normal, |_, _| {
            write_mode("SETUP");
            WmCommand::ModeSwitch(Mode::Setup)
        }),
        bind!(30, modkey, Mode::Toggle, |_, _| {
            write_mode("SETUP");
            WmCommand::ModeSwitch(Mode::Setup)
        }),
        bind!(30, modkey, Mode::Move, |_, _| {
            write_mode("SETUP");
            WmCommand::ModeSwitch(Mode::Setup)
        }),
        // spawn a terminal - modkey+i
        bind!(31, modkey, Mode::Normal, |_, _| exec_command("termite", &[])),
        // spawn an agenda notification - modkey+o
        bind!(32, modkey, Mode::Normal, |_, _| exec_script("org.sh", &[])),
        // spawn a weather notification - modkey+p
        bind!(33, modkey, Mode::Normal, |_, _| exec_script("weather.sh", &[])),
        // lock screen - modkey+s
        bind!(39, modkey, Mode::Normal, |_, _| exec_command("slock", &[])),
        // shutdown system - modkey+CTRL+s
        bind!(39, modkey+CTRL, Mode::Normal, |_, _|
              exec_command("sudo", &["shutdown", "-h", "now"])),
        // go back in tagset history - modkey+g
        bind!(42, modkey, Mode::Normal, |c, s| {
            if s.view_prev() {
                println!("{}", current_tagset(c, s));
                WmCommand::Redraw
            } else {
                WmCommand::NoCommand
            }
        }),
        // focus windows by direction or order - modkey+[hjkl+-]
        bind!(43, modkey, Mode::Normal, focus!(ClientSet::focus_left)),
        bind!(44, modkey, Mode::Normal, focus!(ClientSet::focus_bottom)),
        bind!(45, modkey, Mode::Normal, focus!(ClientSet::focus_top)),
        bind!(46, modkey, Mode::Normal, focus!(ClientSet::focus_right)),
        bind!(35, modkey, Mode::Normal, focus!(ClientSet::focus_next)),
        bind!(61, modkey, Mode::Normal, focus!(ClientSet::focus_prev)),
        // swap windows by direction or order - modkey+SHIFT+[hjkl+-]
        bind!(43, modkey+SHIFT, Mode::Normal, swap!(ClientSet::swap_left)),
        bind!(44, modkey+SHIFT, Mode::Normal, swap!(ClientSet::swap_bottom)),
        bind!(45, modkey+SHIFT, Mode::Normal, swap!(ClientSet::swap_top)),
        bind!(46, modkey+SHIFT, Mode::Normal, swap!(ClientSet::swap_right)),
        bind!(35, modkey+SHIFT, Mode::Normal, swap!(ClientSet::swap_next)),
        bind!(61, modkey+SHIFT, Mode::Normal, swap!(ClientSet::swap_prev)),
        // change layout attributes - modkey+CTRL+[jk]
        bind!(44, modkey+CTRL, Mode::Normal, edit_layout!(
                LayoutMessage::MasterFactorRel(-5),
                LayoutMessage::ColumnRel(-1))),
        bind!(45, modkey+CTRL, Mode::Normal, edit_layout!(
                LayoutMessage::MasterFactorRel(5),
                LayoutMessage::ColumnRel(1))),
        // change work tagset - modkey+CTRL+[hl]
        bind!(43, modkey+CTRL, Mode::Normal, |c, s| {
            let res = if let Some(&mut [Tag::Work(ref mut n), ..]) =
                s.current_mut().map(|s| s.tags.as_mut_slice()) {
                *n = n.saturating_sub(1);
                WmCommand::Redraw
            } else {
                WmCommand::NoCommand
            };
            if res == WmCommand::Redraw {
                println!("{}", current_tagset(c, s));
            }
            res
        }),
        bind!(46, modkey+CTRL, Mode::Normal, |c, s| {
            let res = if let Some(&mut [Tag::Work(ref mut n), ..]) =
                s.current_mut().map(|s| s.tags.as_mut_slice()) {
                *n = n.saturating_add(1);
                WmCommand::Redraw
            } else {
                WmCommand::NoCommand
            };
            if res == WmCommand::Redraw {
                println!("{}", current_tagset(c, s));
            }
            res
        }),
        // move a client to an adjacent work tagset - modkey+CTRL+SHIFT+[hl]
        bind!(43, modkey+CTRL+SHIFT, Mode::Normal, |c, s|
            if let Some(&[Tag::Work(ref n), ..]) =
                s.current().map(|s| s.tags.as_slice()) {
                s.current()
                    .and_then(|t| c.get_focused_window(&t.tags))
                    .and_then(|w| c.update_client(w, |mut cl| {
                        cl.set_tags(&[Tag::Work(n.saturating_sub(1))]);
                        WmCommand::Redraw
                    }))
                    .unwrap_or(WmCommand::NoCommand)
            } else {
                WmCommand::NoCommand
            }
        ),
        bind!(46, modkey+CTRL+SHIFT, Mode::Normal, |c, s|
            if let Some(&[Tag::Work(ref n), ..]) =
                s.current().map(|s| s.tags.as_slice()) {
                s.current()
                    .and_then(|t| c.get_focused_window(&t.tags))
                    .and_then(|w| c.update_client(w, |mut cl| {
                        cl.set_tags(&[Tag::Work(n.saturating_add(1))]);
                        WmCommand::Redraw
                    }))
                    .unwrap_or(WmCommand::NoCommand)
            } else {
                WmCommand::NoCommand
            }
        ),
        // warp the mouse pointer out of the way - modkey+y
        bind!(52, modkey, Mode::Normal, |_, _|
              exec_command("swarp", &["0", "768"])),
        // kill current client - modkey+SHIFT+c
        bind!(54, modkey+SHIFT, Mode::Normal, |c, s| s
            .current()
            .and_then(|t| c.get_focused_window(&t.tags))
            .map(WmCommand::Kill)
            .unwrap_or(WmCommand::NoCommand)
        ),
        // volume controls - XF86Audio{Mute,{Raise,Lower}Volume}
        bind!(121, 0, Mode::Normal, |_, _|
              exec_script("volume.sh", &["toggle"])),
        bind!(122, 0, Mode::Normal, |_, _| exec_script("volume.sh", &["5%-"])),
        bind!(123, 0, Mode::Normal, |_, _| exec_script("volume.sh", &["5%+"])),
        // backlight controls - XF86MonBrightness{Down,Up}
        bind!(232, 0, Mode::Normal, |_, _|
              exec_command("xbacklight", &["-dec", "5"])),
        bind!(233, 0, Mode::Normal, |_, _|
              exec_command("xbacklight", &["-inc", "5"])),
    ]);
    // default tag stack
    wm.setup_tags(
        TagStack::from_presets(
            vec![
                TagSet::new(vec![Tag::Web, Tag::Marks], VStack {
                    master_factor: 75,
                    inverted: false,
                    fixed: true,
                }),
                TagSet::new(vec![Tag::Work(0)], VStack::default()),
                TagSet::new(vec![Tag::Chat], HStack {
                    master_factor: 75,
                    inverted: true,
                    fixed: false,
                }),
                TagSet::new(vec![Tag::Media], Monocle::default()),
                TagSet::new(vec![Tag::Logs, Tag::Mon], HStack {
                    master_factor: 75,
                    inverted: true,
                    fixed: false,
                }),
            ], 1
        )
    );
    // matching function deciding upon client placement
    wm.setup_matching(Box::new(
        |props| if props.name == "Mozilla Firefox" {
            Some((vec![Tag::Web], true))
        } else if props.class.contains(&String::from("uzbl-core")) {
            Some((vec![Tag::Web], true))
        } else if props.class.contains(&String::from("Marks")) {
            Some((vec![Tag::Marks], false))
        } else if props.class.contains(&String::from("Chat")) {
            Some((vec![Tag::Chat], false))
        } else if props.class.contains(&String::from("mpv")) {
            Some((vec![Tag::Media], false))
        } else if props.class.contains(&String::from("Mon")) {
            Some((vec![Tag::Mon], false))
        } else {
            None
        }
    ));
}

fn write_mode(mode: &str) {
    if let Some(path) = home_dir()
        .map(|mut dir| {
            dir.push("tmp");
            dir.push("mode_fifo");
            dir.into_os_string()
        }) {
        if let Ok(mut f) = File::create(path) {
            let _ = writeln!(f, "{}", mode);
        }
    }
}

fn exec_script(script: &str, args: &[&str]) -> WmCommand {
    let _ = home_dir()
        .map(|mut dir| {
            dir.push("dotfiles");
            dir.push("scripts");
            dir.push(script);
            Command::new(dir.into_os_string())
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        });
    WmCommand::NoCommand
}

fn exec_command(command: &str, args: &[&str]) -> WmCommand {
    let _ = Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    WmCommand::NoCommand
}
