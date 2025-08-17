use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn honors_gitignore() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    fs::write(dir.path().join(".gitignore"), "ignored.txt\n")?;
    fs::write(dir.path().join("ignored.txt"), "ignored")?;
    fs::write(dir.path().join("visible.txt"), "visible")?;
    std::process::Command::new("git").arg("init").current_dir(dir.path()).output()?;

    let assert = Command::cargo_bin("xcat")?
        .current_dir(dir.path())
        .arg("--no-copy")
        .arg("--output")
        .arg("json")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let json: Value = serde_json::from_str(&stdout)?;
    let children = json["children"].as_array().unwrap();
    assert!(children.iter().any(|c| c["name"] == "visible.txt"));
    assert!(!children.iter().any(|c| c["name"] == "ignored.txt"));
    Ok(())
}

#[test]
fn excludes_lock_files_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    fs::write(dir.path().join("uv.lock"), "")?;
    fs::write(dir.path().join("visible.txt"), "visible")?;
    std::process::Command::new("git").arg("init").current_dir(dir.path()).output()?;

    let assert = Command::cargo_bin("xcat")?
        .current_dir(dir.path())
        .arg("--no-copy")
        .arg("--output")
        .arg("json")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let json: Value = serde_json::from_str(&stdout)?;
    let children = json["children"].as_array().unwrap();
    assert!(children.iter().any(|c| c["name"] == "visible.txt"));
    assert!(!children.iter().any(|c| c["name"] == "uv.lock"));

    let assert = Command::cargo_bin("xcat")?
        .current_dir(dir.path())
        .arg("--no-copy")
        .arg("--output")
        .arg("json")
        .arg("--include-locks")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let json: Value = serde_json::from_str(&stdout)?;
    let children = json["children"].as_array().unwrap();
    assert!(children.iter().any(|c| c["name"] == "uv.lock"));
    Ok(())
}
