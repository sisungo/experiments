use std::fmt::Write;

const SENTENCE_FINALIZERS: &[char] = &['。', '.', '！', '!', '”', '’', '\'', '"', '」'];

pub fn dos2unix(s: String) -> String {
    s.replace("\r\n", "\n")
}

pub fn unix2dos(s: String) -> String {
    s.replace('\n', "\r\n")
}

pub fn replace(s: String, params: (&str, &str)) -> String {
    s.replace(params.0, params.1)
}

pub fn unsplit_lines(s: String) -> String {
    s.chars()
        .fold(Vec::with_capacity(s.len()), |mut acc, x| {
            match x {
                '\n' => match acc.last() {
                    Some(last) if SENTENCE_FINALIZERS.contains(last) => acc.push('\n'),
                    _ => {}
                },
                x => acc.push(x),
            }
            acc
        })
        .into_iter()
        .collect()
}

pub fn on_para_begin(s: String, param: &str) -> String {
    s.lines().fold(
        String::with_capacity(s.len() + param.len()),
        |mut acc, x| {
            writeln!(&mut acc, "{}{}", param, x).unwrap();
            acc
        },
    )
}

pub fn trim(s: String, param: Option<&str>) -> String {
    match param {
        Some(x) if x.chars().count() == 1 => s.trim_matches(x.chars().next().unwrap()),
        Some(x) => s.trim_matches(|c| x.chars().any(|e| e == c)),
        None => s.trim(),
    }
    .to_owned()
}

pub fn trim_begin(s: String, param: Option<&str>) -> String {
    match param {
        Some(x) if x.chars().count() == 1 => s.trim_start_matches(x.chars().next().unwrap()),
        Some(x) => s.trim_start_matches(|c| x.chars().any(|e| e == c)),
        None => s.trim_start(),
    }
    .to_owned()
}
