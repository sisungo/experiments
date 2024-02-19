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
        match std::io::BufRead::read_line(&mut lock, &mut buf) {
            Ok(_) => break Ok(buf.trim().to_owned()),
            Err(err) => match err.kind() {
                std::io::ErrorKind::UnexpectedEof => continue,
                _ => break Err(err),
            },
        };
    }
}
