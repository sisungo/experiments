use chrono::NaiveDate;
use tokio_postgres::{Client, NoTls};
use toolkit::*;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!("no action");
    }
    let db_cred = std::env::var("db_cred").unwrap();

    let (mut client, connection) = tokio_postgres::connect(&db_cred, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    match &args[1][..] {
        "add_novel" => add_novel(&client).await,
        "add_chapter" => add_novel_chapter(&mut client).await,
        "add_single" => add_single(&mut client).await,
        "get_novel" => get_novel(&mut client).await,
        "ls_novel" => ls_novel(&client).await,
        "del_novel" => del_novel(&mut client).await,
        _ => panic!("unknown action"),
    }
}

async fn ls_novel(client: &Client) {
    let queried = client
        .query("SELECT id, title FROM novels", &[])
        .await
        .unwrap();
    for item in queried {
        println!(
            "{}: {}",
            item.get::<'_, _, i64>("id"),
            item.get::<'_, _, String>("title")
        );
    }
}

async fn del_novel(client: &mut Client) {
    let trans = client.transaction().await.unwrap();
    trans
        .execute("LOCK TABLE novels IN EXCLUSIVE MODE", &[])
        .await
        .unwrap();
    trans
        .execute("LOCK TABLE novel_toc IN EXCLUSIVE MODE", &[])
        .await
        .unwrap();
    let id = ask("Novel ID: ").unwrap().parse::<i64>().unwrap();
    trans
        .execute("CALL novel_delete($1)", &[&id])
        .await
        .unwrap();
}

async fn add_single(client: &mut Client) {
    let trans = client.transaction().await.unwrap();
    trans
        .execute("LOCK TABLE novels IN EXCLUSIVE MODE", &[])
        .await
        .unwrap();
    trans
        .execute("LOCK TABLE novel_toc IN EXCLUSIVE MODE", &[])
        .await
        .unwrap();

    let title = ask("Title: ").unwrap();
    let author = ask_nullable("Author: ").unwrap();
    let description = ask_nullable("Description: ").unwrap();
    let date = ask_nullable("Date: ").unwrap();
    let date = date.map(|x| NaiveDate::parse_from_str(&x, "%F").unwrap());
    let cover_image: Option<String> = None; // not supported yet
    let queried = trans.query("INSERT INTO novels(title, author, description, date, cover_image) VALUES($1, $2, $3, $4, $5) RETURNING id", &[&title, &author, &description, &date, &cover_image]).await.unwrap();
    let id = queried[0].get::<'_, _, i64>("id");

    let chapter_nos = trans
        .query(
            "SELECT chapter_no FROM novel_toc WHERE id=$1 ORDER BY id",
            &[&id],
        )
        .await
        .unwrap();
    let last = chapter_nos
        .last()
        .map(|x| x.get::<'_, _, i32>("chapter_no"))
        .unwrap_or(1);

    let chapter_name = String::new();
    let data_file = ask("Data File: ").unwrap();
    let data = tokio::fs::read_to_string(&data_file).await.unwrap();

    trans.execute("INSERT INTO novel_toc(id, chapter_no, chapter_name, chapter_data) VALUES($1, $2, $3, $4)", &[&id, &last, &chapter_name, &data]).await.unwrap();
    trans.commit().await.unwrap();
}

async fn get_novel(client: &mut Client) {
    let trans = client
        .build_transaction()
        .read_only(true)
        .start()
        .await
        .unwrap();
    trans
        .execute("LOCK TABLE novel_toc IN SHARE MODE", &[])
        .await
        .unwrap();

    let id = ask("ID: ").unwrap().parse::<i64>().unwrap();
    let mut this = trans
        .query("SELECT * FROM novels WHERE id=$1", &[&id])
        .await
        .unwrap();
    let this = this.remove(0);

    println!("Title: {}", this.get::<'_, _, String>("title"));
    println!(
        "Author: {}",
        this.get::<'_, _, Option<String>>("author")
            .unwrap_or_default()
    );
    println!(
        "Description: {}",
        this.get::<'_, _, Option<String>>("description")
            .unwrap_or_default()
    );
    println!(
        "Date: {}",
        this.get::<'_, _, Option<NaiveDate>>("date")
            .unwrap_or_default()
    );
    println!();

    let chapters = trans
        .query("SELECT * FROM novel_toc WHERE id=$1", &[&id])
        .await
        .unwrap();
    for chapter in chapters {
        let no = chapter.get::<'_, _, i32>("chapter_no");
        let name = chapter.get::<'_, _, String>("chapter_name");
        let data = chapter.get::<'_, _, String>("chapter_data");
        println!("Chapter {}: {}", no, name);
        println!("{}", data);
        println!();
    }
}

async fn add_novel(client: &Client) {
    let title = ask("Title: ").unwrap();
    let author = ask_nullable("Author: ").unwrap();
    let description = ask_nullable("Description: ").unwrap();
    let date = ask_nullable("Date: ").unwrap();
    let date = date.map(|x| NaiveDate::parse_from_str(&x, "%F").unwrap());
    let cover_image: Option<String> = None; // not supported yet
    client.execute("INSERT INTO novels(title, author, description, date, cover_image) VALUES($1, $2, $3, $4, $5)", &[&title, &author, &description, &date, &cover_image]).await.unwrap();
}

async fn add_novel_chapter(client: &mut Client) {
    let trans = client.build_transaction().start().await.unwrap();

    trans
        .execute("LOCK TABLE novels IN EXCLUSIVE MODE", &[])
        .await
        .unwrap();

    let id = ask("Novel ID: ").unwrap().parse::<i64>().unwrap();
    let id_rows = trans
        .query("SELECT id FROM novels WHERE id=$1", &[&id])
        .await
        .unwrap();
    if id_rows.is_empty() {
        panic!("id nonexistent");
    }

    let chapter_nos = trans
        .query(
            "SELECT chapter_no FROM novel_toc WHERE id=$1 ORDER BY id",
            &[&id],
        )
        .await
        .unwrap();
    let last = chapter_nos
        .last()
        .map(|x| x.get::<'_, _, i32>("chapter_no"))
        .unwrap_or(0) + 1;

    let chapter_name = ask("Chapter Name: ").unwrap();
    let data_file = ask("Data File: ").unwrap();
    let data = tokio::fs::read_to_string(&data_file).await.unwrap();

    trans.execute("INSERT INTO novel_toc(id, chapter_no, chapter_name, chapter_data) VALUES($1, $2, $3, $4)", &[&id, &last, &chapter_name, &data]).await.unwrap();
    trans.commit().await.unwrap();
}
