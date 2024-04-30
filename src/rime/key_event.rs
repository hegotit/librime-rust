use std::fmt;
use std::str::FromStr;

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyEvent {
    keycode: i32,
    modifier: i32,
}

impl KeyEvent {
    pub fn new(keycode: i32, modifier: i32) -> Self {
        Self { keycode, modifier }
    }

    pub fn keycode(&self) -> i32 {
        self.keycode
    }

    pub fn set_keycode(&mut self, value: i32) {
        self.keycode = value;
    }

    pub fn modifier(&self) -> i32 {
        self.modifier
    }

    pub fn set_modifier(&mut self, value: i32) {
        self.modifier = value;
    }

    pub fn shift(&self) -> bool {
        (self.modifier & K_SHIFT_MASK) != 0
    }

    pub fn ctrl(&self) -> bool {
        (self.modifier & K_CONTROL_MASK) != 0
    }

    pub fn alt(&self) -> bool {
        (self.modifier & K_ALT_MASK) != 0
    }

    pub fn caps(&self) -> bool {
        (self.modifier & K_LOCK_MASK) != 0
    }

    pub fn super_key(&self) -> bool {
        (self.modifier & K_SUPER_MASK) != 0
    }

    pub fn release(&self) -> bool {
        (self.modifier & K_RELEASE_MASK) != 0
    }

    // 按键表示为形如「状态+键名」的文字
    // Return key representation as string
    pub fn repr(&self) -> String {
        let mut modifiers: String = String::new();
        if self.modifier != 0 {
            let mut k: i32 = self.modifier & K_MODIFIER_MASK;
            let mut i: i32 = 0;
            while k != 0 {
                if k & 1 != 0 {
                    if let Some(modifier_name) = get_modifier_name(1 << i) {
                        modifiers.push_str(modifier_name);
                        modifiers.push('+');
                    }
                }
                i += 1;
                k >>= 1;
            }
        }
        if let Some(name) = get_key_name(self.keycode) {
            return format!("{}{}", modifiers, name);
        }

        if self.keycode <= 0xffff {
            format!("{}0x{:04x}", modifiers, self.keycode)
        } else if self.keycode <= 0xffffff {
            format!("{}0x{:06x}", modifiers, self.keycode)
        } else {
            "(unknown)".to_string()
        }
    }

    // 解析文字表示的按键
    // Parse key representation from string
    pub fn parse(&mut self, repr: &str) -> bool {
        self.keycode = 0;
        self.modifier = 0;
        if repr.is_empty() {
            return false;
        }
        if repr.len() == 1 {
            self.keycode = repr.chars().next().unwrap() as i32;
            return true;
        }

        let tokens: Vec<&str> = repr.split('+').collect();
        for &token in &tokens[..tokens.len() - 1] {
            let mask = get_modifier_by_name(token);
            if mask != 0 {
                self.modifier |= mask;
            } else {
                eprintln!("parse error: unrecognized modifier '{}'", token);
                return false;
            }
        }

        let key_token = tokens[tokens.len() - 1];
        self.keycode = get_keycode_by_name(key_token);
        if self.keycode == -1 {
            eprintln!("parse error: unrecognized key '{}'", key_token);
            return false;
        }
        true
    }
}

impl FromStr for KeyEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut event = KeyEvent::default();
        if event.parse(s) {
            Ok(event)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

// KeySequence 类
// KeySequence class
#[derive(Default, Debug)]
pub struct KeySequence {
    events: Vec<KeyEvent>,
}

impl KeySequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn repr(&self) -> String {
        let mut result = String::new();
        for event in &self.events {
            let k = event.repr();
            if k.len() == 1 {
                result.push_str(&k);
            } else if is_unescaped_character(event) {
                result.push(event.keycode() as u8 as char);
            } else {
                result.push_str(&format!("{{{}}}", k));
            }
        }
        result
    }

    pub fn parse(&mut self, repr: &str) -> bool {
        self.events.clear();
        let mut i = 0;
        let n = repr.len();
        while i < n {
            if repr.as_bytes()[i] == b'{' && i + 1 < n {
                let start = i + 1;
                if let Some(end) = repr[start..].find('}') {
                    let len = end;
                    i += len + 2;
                    let mut ke = KeyEvent::default();
                    if !ke.parse(&repr[start..start + len]) {
                        eprintln!("parse error: unrecognized key sequence");
                        return false;
                    }
                    self.events.push(ke);
                } else {
                    eprintln!("parse error: unparalleled brace in '{}'", repr);
                    return false;
                }
            } else {
                let start = i;
                i += 1;
                let mut ke = KeyEvent::default();
                if !ke.parse(&repr[start..start + 1]) {
                    eprintln!("parse error: unrecognized key sequence");
                    return false;
                }
                self.events.push(ke);
            }
        }
        true
    }
}

impl FromStr for KeySequence {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sequence = KeySequence::new();
        if sequence.parse(s) {
            Ok(sequence)
        } else {
            Err(())
        }
    }
}

// 实现Display trait用于格式化输出
impl fmt::Display for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

// 判断是否为未转义字符
fn is_unescaped_character(key_event: &KeyEvent) -> bool {
    let ch = key_event.keycode();
    key_event.modifier() == 0 && ch >= 0x20 && ch <= 0x7e && ch != '{' as i32 && ch != '}' as i32
}

// 模拟获取修饰符名称
fn get_modifier_name(mask: i32) -> Option<&'static str> {
    match mask {
        K_SHIFT_MASK => Some("Shift"),
        K_CONTROL_MASK => Some("Ctrl"),
        K_ALT_MASK => Some("Alt"),
        K_LOCK_MASK => Some("Caps"),
        K_SUPER_MASK => Some("Super"),
        K_RELEASE_MASK => Some("Release"),
        _ => None,
    }
}

// 模拟获取键名称
fn get_key_name(keycode: i32) -> Option<&'static str> {
    match keycode {
        0x20 => Some("Space"),
        0x09 => Some("Tab"),
        0xff => Some("Backspace"),
        // Add more keycode to name mappings as needed
        _ => None,
    }
}

fn main() {
    // 测试示例
    let ke = KeyEvent::new(0x20, K_SHIFT_MASK | K_CONTROL_MASK);
    println!("{}", ke);

    let ks: KeySequence = "{Shift+Ctrl+Space}".parse().unwrap();
    println!("{}", ks);
}

// 模拟通过名称获取修饰符
fn get_modifier_by_name(name: &str) -> i32 {
    match name {
        "Shift" => K_SHIFT_MASK,
        "Ctrl" => K_CONTROL_MASK,
        "Alt" => K_ALT_MASK,
        "Caps" => K_LOCK_MASK,
        "Super" => K_SUPER_MASK,
        "Release" => K_RELEASE_MASK,
        _ => 0,
    }
}

// 模拟通过名称获取键代码
fn get_keycode_by_name(name: &str) -> i32 {
    match name {
        "Space" => 0x20,
        "Tab" => 0x09,
        "Backspace" => 0xff,
        // Add more name to keycode mappings as needed
        _ => -1,
    }
}

const K_SHIFT_MASK: i32 = 1 << 0;
const K_CONTROL_MASK: i32 = 1 << 1;
const K_ALT_MASK: i32 = 1 << 2;
const K_LOCK_MASK: i32 = 1 << 3;
const K_SUPER_MASK: i32 = 1 << 4;
const K_RELEASE_MASK: i32 = 1 << 5;
const K_MODIFIER_MASK: i32 = 0xFF;