/*
 * Copyright Inokentiy Babushkin and contributors (c) 2016-2017
 *
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *     * Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *
 *     * Redistributions in binary form must reproduce the above
 *       copyright notice, this list of conditions and the following
 *       disclaimer in the documentation and/or other materials provided
 *       with the distribution.
 *
 *     * Neither the name of Inokentiy Babushkin nor the names of other
 *       contributors may be used to endorse or promote products derived
 *       from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use std::cmp::Ordering;
use std::process::Command;
use std::str::FromStr;

use toml::Value;

use xkb;

use kbd::err::*;
use kbd::modmask;

/// An index representing a mode.
pub type Mode = usize;

/// A mode switching action.
#[derive(Clone, Copy, Debug)]
pub enum ModeSwitchDesc {
    /// A mode switching action changing the current mode permanently.
    Permanent(Mode),
    /// A temporary mode switching action, changing behaviour only for the next chain.
    Temporary(Mode),
}

/// A command to be executed in reaction to specific key events.
#[derive(Debug)]
pub enum CmdDesc {
    /// A string to be passed to a shell to execute the command.
    Shell(String),
    /// A mode to switch to.
    ModeSwitch(ModeSwitchDesc),
}

impl CmdDesc {
    /// Run a command and possibly return an resulting mode switching action to perform.
    pub fn run(&self) -> Option<ModeSwitchDesc> {
        match *self {
            CmdDesc::Shell(ref repr) => {
                let _ = Command::new("sh").args(&["-c", repr]).spawn();
                None
            },
            CmdDesc::ModeSwitch(ref switch) => {
                Some(*switch)
            },
        }
    }

    /// Construct a command from a TOML value.
    pub fn from_value(bind_str: String, value: Value) -> KbdResult<CmdDesc> {
        if let Value::String(repr) = value {
            Ok(CmdDesc::Shell(repr))
        } else {
            Err(KbdError::KeyTypeMismatch(bind_str, true))
        }
    }
}

/// A keysym wrapper used for various trait implementations.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct KeysymDesc(xkb::Keysym);

impl KeysymDesc {
    pub fn new(inner: xkb::Keysym) -> Self {
        KeysymDesc(inner)
    }
}

impl Ord for KeysymDesc {
    fn cmp(&self, other: &KeysymDesc) -> Ordering {
        let self_inner: u32 = self.0.into();

        self_inner.cmp(&other.0.into())
    }
}

impl PartialOrd for KeysymDesc {
    fn partial_cmp(&self, other: &KeysymDesc) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ::std::fmt::Display for KeysymDesc {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0.utf8())
    }
}

/// A chord description.
///
/// A *chord* is a set of modifiers and a key pressed at the same time, represented
/// by a symbolic keysym value (which is independent of keymap).
#[derive(Debug, PartialEq, Eq)]
pub struct ChordDesc {
    /// The keysym of the chord.
    keysym: KeysymDesc,
    /// The modifier mask of the non-depressed mods of the chord.
    modmask: xkb::ModMask,
}

impl Ord for ChordDesc {
    fn cmp(&self, other: &ChordDesc) -> Ordering {
        let modmask: u32 = self.modmask.into();

        self.keysym.cmp(&other.keysym).then(modmask.cmp(&other.modmask.into()))
    }
}

impl PartialOrd for ChordDesc {
    fn partial_cmp(&self, other: &ChordDesc) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ChordDesc {
    /// Construct a chord description from a string representation of modifiers and a keysym.
    ///
    /// Assuming no spaces are present in the string, interpret a sequence of `+`-separated
    /// modifier descriptions, and a single symbol. Interpolates the `$modkey` variable with the
    /// given modifier mask. The part of the string following the first keysym representation is
    /// discarded.
    pub fn from_string(desc: &str, modkey_mask: xkb::ModMask) -> KbdResult<ChordDesc> {
        let mut modmask = xkb::ModMask(0);

        for word in desc.split('+') {
            if word == "$modkey" {
                debug!("added default modifier");
                modmask::combine(&mut modmask, modkey_mask);
            } else if modmask::from_str(word, &mut modmask) {
                debug!("modifier decoded, continuing chord: {} (modmask={:b})", word, modmask.0);
            } else if let Ok(sym) = xkb::Keysym::from_str(word) {
                debug!("keysym decoded, assuming end of chord: {} ({:?})", word, sym);
                modmask::filter_ignore(&mut modmask);
                return Ok(ChordDesc {
                    keysym: KeysymDesc(sym),
                    modmask,
                });
            } else {
                error!("could not decode keysym or modifier from word, continuing: {}", word);
            }
        }

        Err(KbdError::InvalidChord(desc.to_owned()))
    }

    pub fn new(keysym: KeysymDesc, mut modmask: xkb::ModMask) -> ChordDesc {
        modmask::filter_ignore(&mut modmask);
        ChordDesc { keysym, modmask }
    }

    pub fn keysym(&self) -> KeysymDesc {
        self.keysym
    }

    pub fn modmask(&self) -> u16 {
        self.modmask.0 as u16
    }
}

/// A chain description.
///
/// A *chain* is an ordered sequence of chords to be pressed after each other.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChainDesc {
    /// The chords in the chain, in order.
    chords: Vec<ChordDesc>,
}

impl ChainDesc {
    /// Construct a chain description from a string representation.
    ///
    /// Interpret the string as a sequence of space-separated strings representing chords.
    pub fn from_string(desc: &str, modkey_mask: xkb::ModMask) -> KbdResult<ChainDesc> {
        let mut chords = Vec::new();

        for expr in desc.split(' ') {
            chords.push(ChordDesc::from_string(expr, modkey_mask)?);
        }

        Ok(ChainDesc { chords })
    }

    /// Check if a given chain is a logical prefix of another one.
    ///
    /// Takes into account ignored modifiers and etc. (NB: not yet correctly implemented).
    pub fn is_prefix_of(&self, other: &ChainDesc) -> bool {
        other.chords.starts_with(&self.chords)

        // chord comparison mechanism to use:
        // (keysym == shortcut_keysym) &&
        // ((state_mods & ~consumed_mods & significant_mods) == shortcut_mods)
        // xkb_state_mod_index_is_active etc
        // xkb_state_mod_index_is_consumed etc
    }

    pub fn chords(&self) -> &Vec<ChordDesc> {
        &self.chords
    }

    pub fn clear(&mut self) {
        self.chords.clear();
    }

    pub fn push(&mut self, chord: ChordDesc) {
        self.chords.push(chord);
    }

    pub fn len(&self) -> usize {
        self.chords.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A mode description.
#[derive(Debug)]
pub struct ModeDesc {
    /// An optional command to execute when the given mode is entered.
    enter_cmd: Option<CmdDesc>,
    /// An optional command to execute when the given mode is left.
    leave_cmd: Option<CmdDesc>,
}

impl ModeDesc {
    pub fn new(enter_cmd: Option<CmdDesc>, leave_cmd: Option<CmdDesc>) -> ModeDesc {
        ModeDesc { enter_cmd, leave_cmd }
    }

    pub fn enter_cmd(&self) -> Option<&CmdDesc> {
        self.enter_cmd.as_ref()
    }

    pub fn leave_cmd(&self) -> Option<&CmdDesc> {
        self.leave_cmd.as_ref()
    }
}
