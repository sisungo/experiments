use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Default, Serialize, Deserialize)]
struct Context {
    saved_words: HashSet<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./wgctx.json");
    if !path.exists() {
        std::fs::write(path, serde_json::to_vec_pretty(&Context::default())?)?;
    }
    let mut context: Context = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut rl = rustyline::DefaultEditor::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => process(&mut context, line)?,
            Err(err) => eprintln!("Error: {err}"),
        }
    }
}

fn process(ctx: &mut Context, line: String) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./wgctx.json");
    let line = line.trim();
    if line == "exit" {
        std::fs::write(path, serde_json::to_vec_pretty(ctx)?)?;
        std::process::exit(0);
    }

    if line == "next" {
        next(ctx, None);
    }

    if let Some(n) = line.strip_prefix("next ") {
        let Ok(n) = n.parse::<u32>() else {
            eprintln!("Error: not a positive integer");
            return Ok(());
        };
        next(ctx, Some(n));
    }
    Ok(())
}

fn next(ctx: &mut Context, n: Option<u32>) {
    let n = match n {
        Some(n) => n,
        None => random_wordlen(),
    };

    let mut result = String::with_capacity(n as _);

    for _ in 0..n {
        result.push(next_char(result.as_bytes().last().map(|x| *x as char)));
    }

    if !ctx.saved_words.insert(result.clone()) {
        next(ctx, Some(n));
    } else {
        println!("{result}");
    }
}

fn random_wordlen() -> u32 {
    (rand::random::<u32>() % 18) + 2
}

fn next_char(last: Option<char>) -> char {
    let expect: Vec<u8> = match last {
        Some('c' | 's') => b"haiu".into(),
        Some('a' | 'i' | 'u') => b"bcdfghjklmnprstvwxyz".into(),
        Some('t') => b"haiu".into(),
        Some(_) => b"aiu".into(),
        None => b"abcdfghijklmnprstuvwxyz".into(),
    };
    let ord = rand::random::<usize>() % expect.len();
    expect[ord] as char
}
