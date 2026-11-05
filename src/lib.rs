pub mod agent;
pub mod config;
pub mod evidence;
pub mod project;
pub mod rules;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn terminal_safe(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
