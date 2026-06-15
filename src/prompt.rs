use std::io::{self, Write};

use anyhow::{Result, anyhow};

pub fn text(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;

    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(anyhow!("input cannot be empty"));
    }
    Ok(value)
}

pub fn choose_index(label: &str, len: usize) -> Result<usize> {
    loop {
        let raw = text(label)?;
        match raw.parse::<usize>() {
            Ok(index) if (1..=len).contains(&index) => return Ok(index - 1),
            _ => println!("Enter a number from 1 to {len}."),
        }
    }
}
