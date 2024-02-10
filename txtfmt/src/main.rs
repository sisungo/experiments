mod rule;
mod tools;

use std::path::PathBuf;

#[derive(clap::Parser)]
struct Cmdline {
    #[arg(short, long)]
    rule: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    file: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmdline: Cmdline = clap::Parser::parse();

    let mut s = std::fs::read_to_string(&cmdline.file)?;
    let dos = s.contains('\r');
    s = tools::dos2unix(s);

    let rules = rule::parse(&std::fs::read_to_string(&cmdline.rule)?)?;
    for rule in rules {
        s = rule.run(s);
    }

    if dos {
        s = tools::unix2dos(s);
    }
    let path = match cmdline.output.as_ref() {
        Some(x) => x,
        None => &cmdline.file,
    };
    std::fs::write(path, s.as_bytes())?;

    Ok(())
}
