use console::style;
use url::Url;

/// Runs the setup wizard.
pub fn setup_wizard() -> anyhow::Result<()> {
    eprintln!("📦 {}", style("Setup the Vinyl Server").underlined());
    eprintln!();
    eprintln!("It seemed that it's the first time that the Vinyl Server is running on");
    eprintln!("this directory. To make the server work, some configurations are necessary");
    eprintln!("like database connection, etc. This wizard will guide you to setup the");
    eprintln!("server.");
    eprintln!();

    setup_boot_config()?;

    std::fs::File::create_new(".setup_done")?;

    eprintln!();
    eprintln!("🎉 {}", style("Congratulations!").blue().bold());
    eprintln!("You have successfully initialized Vinyl. Enjoy it! Please press ENTER, and your");
    eprintln!("server will continue starting.");

    std::io::stdin().lines().next();

    Ok(())
}

/// Sets up the boot configuration.
fn setup_boot_config() -> anyhow::Result<()> {
    let db = setup_database();
    let net = setup_net();
    let object = setup_object();

    let dotenv = [db, net, object].join("\n");
    std::fs::write("env", dotenv.as_bytes())?;

    Ok(())
}

/// Sets up the network configuration.
fn setup_net() -> String {
    let listen_url = ask(
        "Which URL would the server listen to?",
        |x| x.parse::<Url>().is_ok(),
        Some("http://0.0.0.0:8080"),
    )
    .parse::<Url>()
    .unwrap();

    format!("NETWORK_LISTEN_URL={listen_url}")
}

/// Sets up the database configuration.
fn setup_database() -> String {
    let vendor = choose(
        "Which database vendor would you like to use?",
        &["mysql", "postgresql"],
        Some("postgresql"),
    );
    let addr = ask(
        "Which database server would you like to connect to?",
        |_| true,
        None,
    );
    let user = ask(
        "Which user would you use to access the database?",
        |_| true,
        None,
    );
    let passwd = ask(
        "What's the password of the specified database user?",
        |_| true,
        None,
    );
    let database = ask(
        "What's name of the database you want to connect to?",
        |_| true,
        None,
    );

    let proto = match &vendor[..] {
        "postgresql" => "postgres",
        "mysql" => "mysql",
        _ => unreachable!(),
    };

    [
        format!("DATABASE_URL={proto}://{user}:{passwd}@{addr}/{database}"),
        format!("DATABASE_CRON_ENABLED=true"),
    ]
    .join("\n")
}

/// Sets up the object storage configuration.
fn setup_object() -> String {
    let url = ask(
        "Which S3 server do you want use to store objects?",
        |x| x.parse::<Url>().is_ok(),
        None,
    );
    let access_key_id = ask("What's your access key id?", |_| true, None);
    let secret_access_key = ask("What's your secret access key?", |_| true, None);
    let bucket = ask("Which S3 bucket do you want to use?", |_| true, None);

    [
        format!("OBJECT_STORAGE=s3"),
        format!("AWS_ENDPOINT_URL={url}"),
        format!("AWS_ACCESS_KEY_ID={access_key_id}"),
        format!("AWS_SECRET_ACCESS_KEY={secret_access_key}"),
        format!("AWS_DEFAULT_REGION=us-west-2"),
        format!("S3_BUCKET={bucket}"),
    ]
    .join("\n")
}

/// Prompts the user to choose one of the choices.
fn choose(prompt: &str, choices: &[&str], default: Option<&str>) -> String {
    if let Some(default) = default {
        if !choices.contains(&default) {
            panic!("`default` must be a member of `choices`");
        }
    }

    let mut choices_prompt = String::new();
    choices_prompt.push_str("[ ");
    for (n, choice) in choices.iter().enumerate() {
        if n != 0 {
            choices_prompt.push_str(" / ");
        }

        if Some(*choice) != default {
            choices_prompt.push_str(&format!("{}", style(choice).dim()));
        } else {
            choices_prompt.push_str(&format!("{}", style(choice).bold().underlined()));
        }
    }
    choices_prompt.push_str(" ]");

    loop {
        eprintln!(
            "{} {} {}",
            style(">").bold(),
            style(prompt).underlined(),
            choices_prompt
        );

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            continue;
        }
        let input = input.trim();
        if input.is_empty() {
            if let Some(default) = default {
                return default.to_string();
            }
            continue;
        }
        if !choices.contains(&input) {
            continue;
        }
        return input.to_string();
    }
}

/// Prompts the user to input a value.
fn ask(prompt: &str, mut validation: impl FnMut(&str) -> bool, default: Option<&str>) -> String {
    loop {
        if let Some(default) = default {
            eprintln!(
                "{} {} [{}]",
                style(">").bold(),
                style(prompt).underlined(),
                style(default).bold().underlined()
            );
        } else {
            eprintln!("{} {}", style(">").bold(), style(prompt).underlined());
        }
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            continue;
        }
        let input = input.trim();
        if let Some(default) = default {
            if input.is_empty() {
                return default.to_string();
            }
        }
        if !validation(input) {
            continue;
        }
        return input.to_string();
    }
}
