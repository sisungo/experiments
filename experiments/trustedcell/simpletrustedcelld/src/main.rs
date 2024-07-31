//! # simpletrustedcelld
//! Very simple TrustedCell daemon. For demonstration and testing usage.

use std::{collections::HashMap, fs::File, io::{Read, Write}};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
struct Request {
    request_id: i64,
    uid: u32,
    cell: String,
    category: String,
    owner: String,
    action: String,
}
impl Request {
    fn parse(s: &str) -> Self {
        let mut splited = s.split(' ');
        let request_id = splited.next().unwrap().parse().unwrap();
        let uid = splited.next().unwrap().parse().unwrap();
        let cell = splited.next().unwrap().to_string();
        let category = splited.next().unwrap().to_string();
        let owner = splited.next().unwrap().to_string();
        let action = splited.next().unwrap().to_string();
        Self {
            request_id,
            uid,
            cell,
            category,
            owner,
            action,
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut request_cache = HashMap::new();
    let mut file = File::options()
        .read(true)
        .write(true)
        .open("/sys/kernel/security/trustedcell/host")?;
    let mut buffer = [0u8; 512];

    loop {
        let n = file.read(&mut buffer)?;
        let slice = unsafe { std::mem::transmute(&buffer[..n]) };
        let request = Request::parse(slice);
        let mut req2 = request.clone();
        req2.request_id = 0;
        if request_cache.get(&req2).is_some() {
            _ = file.write(resp_ok(&request).as_bytes());
            continue;
        }
        let s = match judge(&request) {
            true => resp_ok(&request),
            false => resp_bad(&request),
        };
        _ = file.write(s.as_bytes());
        request_cache.insert(req2, s);
    }
}

fn judge(request: &Request) -> bool {
    loop {
        match native_dialog::MessageDialog::new()
            .set_title("simpletrustedcelld -- GUI Helper")
            .set_text(&format_request(request)[..])
            .set_type(native_dialog::MessageType::Warning)
            .show_confirm() {
            Ok(x) => break x,
            Err(x) => {
                eprintln!("{x}");
                continue;
            },
        }
    }
}

fn format_request(request: &Request) -> String {
    format!("是否允许 {} 对 {} 的 {} 进行 {}？", request.cell, request.owner, request.category, request.action)
}

fn resp_ok(request: &Request) -> String {
    format!("{} 1 1", request.request_id)
}

fn resp_bad(request: &Request) -> String {
    format!("{} 0 1", request.request_id)
}
