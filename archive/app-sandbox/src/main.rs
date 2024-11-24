use landlock::Access;

mod rule;

#[derive(clap::Parser)]
struct Cmdline {
    #[arg(short, long)]
    rule: std::path::PathBuf,

    #[arg(long)]
    net: bool,

    #[arg(long)]
    fs: bool,

    #[arg(last = true)]
    command: Vec<String>,
}

fn main() -> eyre::Result<()> {
    use clap::Parser;
    use landlock::{AccessFs, AccessNet, BitFlags, RulesetAttr};
    use std::os::unix::process::CommandExt;

    let cmdline = Cmdline::parse();
    if cmdline.command.is_empty() {
        Err(eyre::eyre!("no command specified"))?;
    }
    let rule_s = std::fs::read_to_string(&cmdline.rule)?;

    let mut ruleset = landlock::Ruleset::default();

    if cmdline.net {
        ruleset = ruleset.handle_access(BitFlags::<AccessNet>::all())?;
    }
    if cmdline.fs {
        ruleset = ruleset.handle_access(AccessFs::from_all(landlock::ABI::V4))?;
    }

    let mut ruleset = ruleset.create()?;

    rule::parse_to(&mut ruleset, &cmdline.rule, &rule_s)?;

    ruleset.restrict_self()?;

    Err(std::process::Command::new(cmdline.command.first().unwrap())
        .args(&cmdline.command[1..])
        .exec())?;

    Ok(())
}
