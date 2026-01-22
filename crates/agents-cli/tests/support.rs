use assert_cmd::Command;

pub fn agents_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("agents"))
}
