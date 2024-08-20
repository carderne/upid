use postgres::{Client, NoTls};
use std::error::Error;
use upid::Upid;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=mypassword dbname=postgres",
        NoTls,
    )?;

    let create_ext = "CREATE EXTENSION IF NOT EXISTS upid_pg;";
    client.execute(create_ext, &[])?;
    println!("Extension ready");

    let drop_table = "DROP TABLE IF EXISTS test_upid;";
    client.execute(drop_table, &[])?;

    let create_table = r#"
        CREATE TABLE test_upid (
            id_upid TEXT NOT NULL,   -- passing string for upid type
            id_uuid UUID NOT NULL,   -- passing uuid for uuid type
            id_text TEXT NOT NULL    -- passing string for text type
        );
    "#;
    client.execute(create_table, &[])?;
    println!("Table created");

    let id = Upid::new("user");

    let query = r#"
        INSERT INTO test_upid (id_upid, id_uuid, id_text)
        VALUES ($1, $2, $3)
        RETURNING id_upid, id_uuid, id_text;
    "#;
    for row in client.query(
        query,
        &[&id.to_string(), &Uuid::from(id), &id.to_string()],
    )? {
        let id_upid: String = row.get(0);
        let id_uuid: Uuid = row.get(1);
        let id_text: String = row.get(2);
        println!(
            "Inserted:\nid_upid={id_upid}\nid_uuid={id_uuid}\nid_text={id_text}",
        );
    }

    Ok(())
}
