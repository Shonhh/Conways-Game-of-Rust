//! Exercise the real CLI with piped output rather than a terminal.
use std::process::Command;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_conway_game_of_rust"))
}

#[test]
fn help_does_not_require_a_terminal() {
    let result = command().arg("--help").output().unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().contains("--headless"));
}

#[test]
fn headless_csv_runs_without_a_terminal() {
    let result = command().args([
        "--headless", "--width", "5", "--height", "5", "--pattern", "block",
        "--generations", "3",
    ]).output().unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    let csv = String::from_utf8(result.stdout).unwrap();
    let lines: Vec<_> = csv.lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].starts_with("width,height,boundary,"));
    let initial: Vec<_> = lines[1].split(',').collect();
    let final_state: Vec<_> = lines[4].split(',').collect();
    assert_eq!(final_state[6], "3");
    assert_eq!(final_state[8], "4");
    assert_eq!(initial[9], final_state[9]);
}

#[test]
fn invalid_arguments_fail_without_csv() {
    let result = command().args(["--headless", "--width", "0"]).output().unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}

#[test]
fn existing_output_file_is_preserved() {
    // Each test binary gets its own process ID; this path is only used here.
    let path = std::env::temp_dir().join(format!("conway-output-test-{}.csv", std::process::id()));
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
    use std::io::Write;
    file.write_all(b"keep this content\n").unwrap();
    drop(file);
    let result = command().args(["--headless", "--generations", "0", "--output"])
        .arg(&path).output().unwrap();
    let contents = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(!result.status.success());
    assert_eq!(contents, b"keep this content\n");
}
