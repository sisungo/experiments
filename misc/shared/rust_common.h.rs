#[allow(unused)]
type StdError = Box<dyn std::error::Error>;

#[allow(unused)]
fn system(cmd: &str) -> Result<Result<(), StdError>, StdError> {
    let exit_status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()?
        .wait()?;

    Ok(match exit_status.success() {
        true => Ok(()),
        false => Err(match exit_status.code() {
            Some(n) => Box::from(format!("process exited with code {n}")),
            None => Box::from("process terminated by signal"),
        }),
    })
}

#[allow(unused)]
fn perform<I, S>(cmd: I) -> Result<Result<(), StdError>, StdError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = cmd.into_iter();
    let exit_status =
        std::process::Command::new(cmd.next().expect("there must be at least one parameter"))
            .args(cmd)
            .spawn()?
            .wait()?;

    Ok(match exit_status.success() {
        true => Ok(()),
        false => Err(match exit_status.code() {
            Some(n) => Box::from(format!("process exited with code {n}")),
            None => Box::from("process terminated by signal"),
        }),
    })
}

#[allow(unused)]
fn ask(hint: &str) -> std::io::Result<String> {
    let mut buf = String::new();
    let mut lock = std::io::stdin().lock();

    loop {
        print!("{hint}");
        std::io::Write::flush(&mut std::io::stdout())?;
        std::io::BufRead::read_line(&mut lock, &mut buf)?;
        if !buf.ends_with("\n") {
            println!();
            continue;
        }
        break Ok(buf.trim().to_owned());
    }
}

#[allow(unused)]
fn ask_until<F: FnMut(&str) -> bool>(hint: &str, mut validator: F) -> std::io::Result<String> {
    loop {
        let asked = ask(hint)?;
        if validator(&asked) {
            break Ok(asked);
        }
    }
}
