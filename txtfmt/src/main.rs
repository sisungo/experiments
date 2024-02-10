#[derive(clap::Parser)]
struct Cmdline {
    #[arg(short, long)]
    merge_lines: bool,

    #[arg(short, long)]
    duplicate_newline: bool,

    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmdline: Cmdline = clap::Parser::parse();
    let mut s = std::fs::read_to_string(&cmdline.file)?;
    if s.contains("\r\n") {
        s = dos_to_unix(&s);
    }

    if cmdline.merge_lines {
        s = merge_lines(&s);
    }

    if cmdline.duplicate_newline {
        s = s.replace('\n', "\n\n");
    }

    println!("{s}");

    Ok(())
}

fn dos_to_unix(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn merge_lines(s: &str) -> String {
    s.chars()
        .fold(Vec::with_capacity(s.len()), |mut acc, x| {
            match x {
                '\n' => match acc.last() {
                    Some('。' | '.') => acc.push('\n'),
                    _ => {}
                },
                x => acc.push(x),
            }
            acc
        })
        .into_iter()
        .collect()
}
