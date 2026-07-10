use std::fmt::Display;

pub fn is_default_tab_name(name: &str) -> bool {
    if name.starts_with("Tab #") && name[5..].parse::<u32>().is_ok() {
        return true;
    }
    if let Some(colon) = name.find(": ") {
        let prefix = &name[..colon];
        if prefix.parse::<u32>().is_ok() {
            return true;
        }
    }
    false
}

pub fn log(msg: impl Display) {
    eprintln!("[plugin-layoutswitch] {}", msg);
}
