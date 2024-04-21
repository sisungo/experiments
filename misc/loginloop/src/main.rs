use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    loop {
        Command::new("clear").spawn().unwrap().wait().unwrap();

        let mut login = String::with_capacity(16);
        println!("Welcome to Embedded Server OS!");
        println!();
        let hostname =
            std::fs::read_to_string("/etc/hostname").unwrap_or_else(|_| String::from("localhost"));
        let hostname = hostname.trim();
        print!("{hostname} login: ");
        std::io::stdout().flush().ok();
        let Ok(_) = std::io::stdin().read_line(&mut login) else {
            continue;
        };
        let login = login.trim();

        Command::new("login")
            .arg(&login)
            .uid(0)
            .gid(0)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
