use std::fmt::Write;
use crate::rule::counter;

const SENTENCE_FINALIZERS: &[char] = &['。', '.', '！', '!', '”', '’', '\'', '"', '」', '?'];

macro_rules! generate_trim {
    ($s:expr, $p:expr, $m:ident, $f:ident) => {
        $s.lines()
            .fold(String::with_capacity($s.len()), |mut acc, line| {
                let trimmed = match $p {
                    Some(x) => line.$m(|c| x.chars().any(|e| e == c)),
                    None => line.$f(),
                };
                writeln!(&mut acc, "{trimmed}").unwrap();
                acc
            })
    };
}

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

pub fn fmt(var: &'static str, mut s: String, p: &[String]) -> String {
    let mut iter = p.iter().map(|x| x.as_str());
        let Some(fmt) = iter.next() else {
            return s;
        };
        let to = fmt.as_bytes().iter().copied().fold(
            Vec::with_capacity(fmt.len() + 32),
            |mut acc, c| {
                if let Some(b'%') = acc.last() {
                    acc.pop();
                    match c {
                        b'%' => acc.push(b'%'),
                        b'd' => acc.append(
                            &mut counter(iter.next().unwrap_or("bad"))
                                .unwrap_or_default()
                                .to_string()
                                .into_bytes(),
                        ),
                        _ => acc.push(b'?'),
                    }
                } else {
                    acc.push(c);
                }
                acc
            },
        );
        match var {
            "append" => s.push_str(&String::from_utf8_lossy(&to)),
            "ob" => s = format!("{}{}", String::from_utf8_lossy(&to), s),
            _ => unreachable!(),
        }

        s
}

pub fn make_para_begin_with(s: String, param: &str) -> String {
    s.lines().fold(
        String::with_capacity(s.len() + param.len()),
        |mut acc, x| {
            if !x.starts_with(param) {
                writeln!(&mut acc, "{param}{x}").unwrap();
            } else {
                writeln!(&mut acc, "{x}").unwrap();
            }
            acc
        },
    )
}

pub fn trim(s: String, param: Option<&str>) -> String {
    generate_trim!(s, param, trim_matches, trim)
}

pub fn trim_begin(s: String, param: Option<&str>) -> String {
    generate_trim!(s, param, trim_start_matches, trim_start)
}

pub fn trim_end(s: String, param: Option<&str>) -> String {
    generate_trim!(s, param, trim_end_matches, trim_end)
}

pub fn append(mut s: String, param: &str) -> String {
    s.push_str(param);
    s
}

pub fn append_line(mut s: String, param: &str) -> String {
    if s.bytes().last() != Some(b'\n') {
        s.push('\n');
    }
    s.push_str(param);
    s.push('\n');
    s
}
