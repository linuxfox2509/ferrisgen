use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn prints_password() {
    let mut cmd = cargo_bin_cmd!("ferrisgen-cli");
    cmd.arg("--length").arg("12");
    // Match 12 characters plus optional trailing newline (works on Windows and Unix)
    cmd.assert().success().stdout(predicates::str::is_match("^.{12}\r?\n?$").unwrap());
}