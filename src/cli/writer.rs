use crate::error::{Error, Result};
use colored::Colorize;
use inquire::Confirm;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

/// Write content to a file with diff preview and user confirmation
pub fn write_with_confirmation(path: &Path, content: &str, skip_prompts: bool) -> Result<()> {
    // Check if file exists
    if path.exists() {
        let existing = fs::read_to_string(path)?;

        if skip_prompts {
            // Non-interactive mode: just overwrite
            fs::write(path, content)?;
            println!("{}", format!("✓ Overwrote {}", path.display()).green());
            return Ok(());
        }

        // Show diff
        println!("\n{}", format!("File exists: {}", path.display()).yellow());
        println!("{}", "Showing differences:".cyan());
        println!();

        show_diff(&existing, content);

        // Ask user what to do
        let choices = vec!["Overwrite", "Skip", "Backup and overwrite", "Abort"];
        let choice = inquire::Select::new("What would you like to do?", choices)
            .prompt()
            .map_err(|_| Error::UserCancelled)?;

        match choice {
            "Overwrite" => {
                fs::write(path, content)?;
                println!("{}", format!("✓ Overwrote {}", path.display()).green());
            }
            "Skip" => {
                println!("{}", format!("⊘ Skipped {}", path.display()).yellow());
            }
            "Backup and overwrite" => {
                let backup_path = path.with_extension(format!(
                    "{}.bak.{}",
                    path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                    chrono::Local::now().format("%Y%m%d_%H%M%S")
                ));
                fs::copy(path, &backup_path)?;
                fs::write(path, content)?;
                println!("{}", format!("✓ Backed up to {}", backup_path.display()).green());
                println!("{}", format!("✓ Wrote {}", path.display()).green());
            }
            "Abort" => {
                return Err(Error::UserCancelled);
            }
            _ => unreachable!(),
        }
    } else {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if skip_prompts {
            // Non-interactive mode: just create
            fs::write(path, content)?;
            println!("{}", format!("✓ Created {}", path.display()).green());
        } else {
            // Confirm creation of new file
            let confirm = Confirm::new(&format!("Create {}?", path.display()))
                .with_default(true)
                .prompt()
                .map_err(|_| Error::UserCancelled)?;

            if confirm {
                fs::write(path, content)?;
                println!("{}", format!("✓ Created {}", path.display()).green());
            } else {
                println!("{}", format!("⊘ Skipped {}", path.display()).yellow());
            }
        }
    }

    Ok(())
}

/// Show a colored diff between old and new content
fn show_diff(old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);

    let mut line_num_old = 1;
    let mut line_num_new = 1;

    for change in diff.iter_all_changes() {
        let (sign, style, line_old, line_new) = match change.tag() {
            ChangeTag::Delete => ("-", "red", Some(line_num_old), None),
            ChangeTag::Insert => ("+", "green", None, Some(line_num_new)),
            ChangeTag::Equal => (" ", "white", Some(line_num_old), Some(line_num_new)),
        };

        let line_marker = match (line_old, line_new) {
            (Some(o), Some(n)) => format!("{:4}|{:4}", o, n),
            (Some(o), None) => format!("{:4}|    ", o),
            (None, Some(n)) => format!("    |{:4}", n),
            (None, None) => "    |    ".to_string(),
        };

        let line = format!("{} {} {}", line_marker, sign, change);

        match style {
            "red" => println!("{}", line.red()),
            "green" => println!("{}", line.green()),
            _ => println!("{}", line),
        }

        if change.tag() == ChangeTag::Delete || change.tag() == ChangeTag::Equal {
            line_num_old += 1;
        }
        if change.tag() == ChangeTag::Insert || change.tag() == ChangeTag::Equal {
            line_num_new += 1;
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_show_diff() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\nline 4\n";

        // Just ensure it doesn't panic
        show_diff(old, new);
    }
}
