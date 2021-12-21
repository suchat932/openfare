use anyhow::{format_err, Result};

pub use openfare_lib::config::Config as Metrics;

fn get_regex() -> Result<regex::Regex> {
    Ok(regex::Regex::new(r"metrics\.(.*)")?)
}

pub fn is_match(name: &str) -> Result<bool> {
    Ok(get_regex()?.is_match(name))
}

pub fn set(metrics: &mut Metrics, name: &str, value: &str) -> Result<()> {
    let name_error_message = format!("Unknown setting field name: {}", name);

    let captures = get_regex()?
        .captures(name)
        .ok_or(format_err!(name_error_message.clone()))?;
    let field = captures
        .get(1)
        .ok_or(format_err!(name_error_message.clone()))?
        .as_str();

    match field {
        "developers-count" => {
            metrics.developers_count = Some(value.parse::<u64>()?);
            Ok(())
        }
        _ => Err(format_err!(name_error_message.clone())),
    }
}

pub fn get(metrics: &Metrics, name: &str) -> Result<String> {
    let name_error_message = format!("Unknown setting field name: {}", name);

    let captures = get_regex()?
        .captures(name)
        .ok_or(format_err!(name_error_message.clone()))?;
    let field = captures
        .get(1)
        .ok_or(format_err!(name_error_message.clone()))?
        .as_str();

    match field {
        "developers-count" => Ok(if let Some(developers_count) = metrics.developers_count {
            developers_count.to_string()
        } else {
            "".to_string()
        }),
        _ => Err(format_err!(name_error_message.clone())),
    }
}
