use clap::Parser;
use landlock::{Ruleset, RulesetCreated};

#[derive(Parser)]
struct Cmdline {}

fn main() {
    let cmdline = Cmdline::parse();
    let ruleset = Ruleset::default().create().unwrap();
}
