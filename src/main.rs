// #![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rusqlite::{Connection};
// use serde::Deserialize;
// use serde::Serialize;


#[derive(Debug, Serialize)]
struct TodoList {
    items: Vec<TodoItem>,
}


#[derive(Debug, Serialize)]
struct TodoItem {
    id: i64,
    item: String,
}

#[derive(Debug, Serialize)]
struct Status {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Add {
    data: String,
}

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[get("/todo")]
async fn get_todo() -> Result<Json<TodoList>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => return Err(String::from("Failed to connect to database")),
    };

    let mut statement = match db_connection.prepare("select id, item from todo_list;") {
        Ok(statement) => statement,
        Err(_) => return Err("Failed to prepare the quary".into()),
    };

    let results = statement.query_map(rusqlite::NO_PARAMS, |row| {
        Ok(TodoItem {
            id: row.get(0)?,
            item: row.get(1)?,
        })
    });

    match results {
        Ok(rows) => {
            // collect result so we can return as array
            let collection: rusqlite::Result<Vec<_>> = rows.collect();

            match collection {
                Ok(items) => Ok(Json(TodoList{items})),
                Err(_) => Err("Failed to collect todo item".into()),
            }
        }
        Err(_) => Err("Failed to fatch todo items".into())
    }

}


#[post("/todo", format = "json", data = "<item>")]
async fn add_todo(item : Json<String>) -> Result<Json<Status>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => return Err(String::from("Failed to connect to database")),
    };

    let mut statement =
    match db_connection.prepare("insert into todo_list (id, item) values (null, $1);") {
        Ok(statement) => statement,
        Err(_) => return Err("Failed to prepare query".into()),
    };
    let results = statement.execute(&[&item.0]);

    match results {
        Ok(rows) => Ok(Json(Status{message: rows.to_string()})),
        Err(_) => Err("Failed to insert todo".into()),
    }
}


#[delete("/todo/<id>")]
async fn delete_todo(id: i64) -> Result<Json<Status>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(String::from("Failed to connect to database"));
        }
    };

    let mut statement = match db_connection.prepare("delete from todo_list where id = $1;") {
        Ok(statement) => statement,
        Err(_) => return Err("Failed to prepare query".into()),
    };
    let results = statement.execute(&[&id]);

    match results {
        Ok(rows_affected) => Ok(Json(Status {
            message: format!("{} rows deleted!", rows_affected),
        })),
        Err(_) => Err("Failed to delete todo item".into()),
    }
}

#[launch]
fn rocket() -> _ {
    {
        let db_connection = Connection::open("data.sqlite").unwrap();

        db_connection
        .execute(
            "create table if not exists todo_list(
                id integer primary key,
                item varchar(64) not null
            );",
            rusqlite::NO_PARAMS,
        )
        .unwrap();
    }

    rocket::build()
    .mount("/", routes![index, get_todo, add_todo, delete_todo])
}