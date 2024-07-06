use crate::rime::key_table::{
    get_key_name, get_keycode_by_name, get_modifier_by_name, get_modifier_name, Mask,
    XK_VOID_SYMBOL,
};
use log::error;
use std::default;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct KeyEvent {
    keycode: u32,
    modifier: i32,
}

impl KeyEvent {
    pub fn new(keycode: u32, modifier: i32) -> Self {
        Self { keycode, modifier }
    }

    pub fn keycode(&self) -> u32 {
        self.keycode
    }

    fn set_keycode(&mut self, value: u32) {
        self.keycode = value;
    }

    pub(crate) fn modifier(&self) -> i32 {
        self.modifier
    }

    fn set_modifier(&mut self, value: i32) {
        self.modifier = value;
    }

    pub fn shift(&self) -> bool {
        (self.modifier & (Mask::Shift as i32)) != 0
    }

    pub fn ctrl(&self) -> bool {
        (self.modifier & (Mask::Control as i32)) != 0
    }

    pub fn alt(&self) -> bool {
        (self.modifier & (Mask::Alt as i32)) != 0
    }

    fn caps(&self) -> bool {
        (self.modifier & (Mask::Lock as i32)) != 0
    }

    fn super_(&self) -> bool {
        (self.modifier & (Mask::Super as i32)) != 0
    }

    pub fn release(&self) -> bool {
        (self.modifier & (Mask::Release as i32)) != 0
    }

    // The keys are represented as text in the form of "state+key_name"
    // If there is no key name, it is represented as a four or six digit hexadecimal number
    // For example, "0x12ab", "0xfffffe"
    pub fn repr(&self) -> String {
        // Stringify modifiers
        let mut modifiers = String::new();
        if self.modifier != 0 {
            let mut k = self.modifier & (Mask::Modifier as i32);
            let mut i = 0;
            while k != 0 {
                if (k & 1) != 0 {
                    if let Some(modifier_name) = get_modifier_name(1 << i) {
                        modifiers.push_str(modifier_name);
                        modifiers.push('+');
                    }
                }
                k >>= 1;
                i += 1;
            }
        }

        // First lookup predefined key name
        if let Some(name) = get_key_name(self.keycode) {
            return format!("{}{}", modifiers, name);
        }

        // No name :-| return its hex value
        if self.keycode <= 0xffff {
            format!("{}0x{:04x}", modifiers, self.keycode)
        } else if self.keycode <= 0xffffff {
            format!("{}0x{:06x}", modifiers, self.keycode)
        } else {
            "(unknown)".to_string()
        }
    }

    // Parse key representation from string
    pub fn parse(&mut self, repr: &str) -> bool {
        self.keycode = 0;
        self.modifier = 0;
        if repr.is_empty() {
            return false;
        }
        if repr.len() == 1 {
            self.keycode = repr.chars().next().unwrap() as u32;
            return true;
        }

        let tokens: Vec<&str> = repr.split('+').collect();
        for &token in &tokens[..tokens.len() - 1] {
            let mask = get_modifier_by_name(Some(token));
            if mask != 0 {
                self.modifier |= mask;
            } else {
                error!("Parse error: unrecognized modifier '{}'", token);
                return false;
            }
        }

        let key_token = tokens[tokens.len() - 1];
        self.keycode = get_keycode_by_name(key_token);
        if self.keycode == XK_VOID_SYMBOL {
            error!("Parse error: unrecognized key '{}'", key_token);
            return false;
        }
        true
    }
}

impl Display for KeyEvent {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.repr())
    }
}

impl FromStr for KeyEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut event = KeyEvent::default();
        if !event.parse(s) {
            event.keycode = 0;
            event.modifier = 0;
        }
        Ok(event)
    }
}

#[derive(Debug, Default)]
pub struct KeySequence(Vec<KeyEvent>);

impl Deref for KeySequence {
    type Target = Vec<KeyEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeySequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for KeySequence {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.repr())
    }
}

impl FromStr for KeySequence {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sequence = KeySequence::default();
        if !sequence.parse(s) && sequence.len() > 0 {
            sequence.clear();
        }
        Ok(sequence)
    }
}

impl KeySequence {
    /*
     * Representable as a sequence of text.
     * If it includes keys that do not produce printable characters, mark them as {key_name}.
     * Combination keys are also marked as {combination_key_state+key_name}.
     */
    pub fn repr(&self) -> String {
        let mut result = String::new();
        for key_event in self.deref() {
            let k = key_event.repr();
            if k.len() == 1 {
                result.push_str(&k);
            } else if is_unescaped_character(key_event) {
                result.push(key_event.keycode() as u8 as char);
            } else {
                result.push('{');
                result.push_str(&k);
                result.push('}');
            }
        }
        result
    }

    // Parse key sequence description text
    pub fn parse(&mut self, repr: &str) -> bool {
        self.0.clear();
        let repr_bytes = repr.as_bytes();
        let n = repr.len();
        let mut i = 0;
        while i < n {
            let (start, len) = if repr_bytes[i] == b'{' && i + 1 < n {
                let start = i + 1;
                if let Some(len) = repr[start..].find('}') {
                    i = start + len;
                    (start, len)
                } else {
                    error!("Parse error: unparalleled brace in '{}'", repr);
                    return false;
                }
            } else {
                (i, 1)
            };

            let mut ke = KeyEvent::default();
            if !ke.parse(&repr[start..start + len]) {
                error!("Parse error: unrecognized key sequence",);
                return false;
            }
            self.0.push(ke);
            i += 1;
        }
        true
    }
}

fn is_unescaped_character(key_event: &KeyEvent) -> bool {
    let ch = key_event.keycode();
    key_event.modifier() == 0 && ch >= 0x20 && ch <= 0x7e && ch != '{' as u32 && ch != '}' as u32
}
