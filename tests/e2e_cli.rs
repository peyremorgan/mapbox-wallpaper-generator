use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_text_is_available() {
    let mut cmd = Command::cargo_bin("mapbox_wallpaper_generator").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Generate ultra high-resolution wallpapers"));
}

#[test]
fn rejects_zero_rows() {
    let mut cmd = Command::cargo_bin("mapbox_wallpaper_generator").unwrap();
    cmd.args(["Paris", "--rows", "0"]);
    cmd.assert()
        .failure()
        .stderr(contains("value must be >= 1"));
}

#[test]
fn rejects_too_high_concurrency() {
    let mut cmd = Command::cargo_bin("mapbox_wallpaper_generator").unwrap();
    cmd.args(["Paris", "--concurrency", "64"]);
    cmd.assert()
        .failure()
        .stderr(contains("concurrency must be between 1 and 32"));
}
