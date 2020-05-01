use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Error;
use assert_cmd::Command;
use predicates::prelude::*;
use rexpect::spawn_bash;
use tempdir::TempDir;

fn create_temp_dir(name: &str) -> Result<TempDir, Error> {
    Ok(TempDir::new(name)?)
}

fn make_config_file(tempdir: &TempDir) -> Result<PathBuf, Error> {
    let db_dir = tempdir.path().join("db");
    let themes_dir = tempdir.path().join("themes");
    let config_contents = format!(
        "theme = 'base16-ocean.dark'\n\
db_dir = \"{}\"\n\
themes_dir = \"{}\"",
        db_dir.to_str().unwrap(),
        themes_dir.to_str().unwrap()
    );
    let config_file = tempdir.path().join("the-way.toml");
    fs::write(&config_file, config_contents)?;
    Ok(config_file.to_path_buf())
}

fn get_current_config_file() -> Result<Option<String>, Error> {
    let mut cmd = Command::cargo_bin("the-way")?;
    let output = cmd
        .arg("config")
        .arg("get")
        .assert()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output)?;
    let output = output.trim();
    if !output.is_empty() {
        Ok(Some(output.to_owned()))
    } else {
        Ok(None)
    }
}

#[test]
fn it_works() -> Result<(), Error> {
    let mut cmd = Command::cargo_bin("the-way")?;
    // Pretty much the only command that works without assuming any input or modifying anything
    cmd.arg("list").assert().success();
    Ok(())
}

#[test]
fn change_config_file() -> Result<(), Error> {
    let temp_dir = create_temp_dir("change_config_file")?;
    let config_file = make_config_file(&temp_dir)?;
    let mut cmd = Command::cargo_bin("the-way")?;
    let output = cmd
        .env("THE_WAY_CONFIG", &config_file)
        .arg("config")
        .arg("get")
        .assert()
        .get_output()
        .stdout
        .clone();
    let output_config_file = String::from_utf8(output)?.trim().to_owned();
    let output_config_file = Path::new(&output_config_file);
    assert!(output_config_file.exists(), "{:?}", output_config_file);
    assert_eq!(output_config_file, config_file);
    temp_dir.close()?;
    Ok(())
}

#[test]
fn change_theme() -> Result<(), Error> {
    let temp_dir = create_temp_dir("change_theme")?;
    let config_file = make_config_file(&temp_dir)?;
    let theme = "base16-ocean.dark";
    let mut cmd = Command::cargo_bin("the-way")?;
    cmd.env("THE_WAY_CONFIG", &config_file)
        .arg("themes")
        .arg("set")
        .arg(theme)
        .assert()
        .success();
    let mut cmd = Command::cargo_bin("the-way")?;
    let output = cmd
        .env("THE_WAY_CONFIG", &config_file)
        .arg("themes")
        .arg("current")
        .assert()
        .get_output()
        .stdout
        .clone();
    let theme_output = String::from_utf8(output)?;
    assert_eq!(theme_output.trim(), theme);
    temp_dir.close()?;
    Ok(())
}

fn add_snippet_rexpect(
    config_file: PathBuf,
    previous_config_file: Option<String>,
) -> rexpect::errors::Result<()> {
    let mut p = spawn_bash(Some(30_000))?;
    // Change to new directory
    p.send_line(&format!(
        "export THE_WAY_CONFIG={}",
        config_file.to_string_lossy()
    ))?;
    // Assert that change worked
    let current_config_file = get_current_config_file();
    assert!(current_config_file.is_ok());
    let current_config_file = current_config_file.unwrap();
    assert!(current_config_file.is_some());
    assert_eq!(
        current_config_file.unwrap(),
        config_file.to_string_lossy().to_owned()
    );

    // Add a snippet
    p.execute("target/release/the-way", "Description:").unwrap();
    p.send_line("test description")?;
    p.exp_string("Language:")?;
    p.send_line("rust")?;
    p.exp_regex("Tags \\(.*\\):")?;
    p.send_line("tag1 tag2")?;
    p.exp_regex("Code snippet \\(.*\\):")?;
    p.send_line("code")?;
    p.exp_regex("Added snippet #1")?;
    p.wait_for_prompt()?;

    // Change back to old directory
    let mut p = spawn_bash(Some(30_000))?;
    if let Some(previous_config_file) = previous_config_file {
        p.send_line(&format!("export THE_WAY_CONFIG={}", previous_config_file))?;
    } else {
        p.send_line("unset THE_WAY_CONFIG")?;
    }
    Ok(())
}

#[test]
fn add_snippet() -> Result<(), Error> {
    let previous_config_file = get_current_config_file()?;
    let temp_dir = create_temp_dir("add_snippet")?;
    let config_file = make_config_file(&temp_dir)?;
    assert!(add_snippet_rexpect(config_file, previous_config_file).is_ok());
    temp_dir.close()?;
    Ok(())
}
