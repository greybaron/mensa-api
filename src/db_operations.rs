use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use std::collections::BTreeMap;

use crate::{
    campus_request_funcs::build_date_string,
    types::{MealGroup, DB_FILENAME},
};

pub fn check_or_create_db_tables() -> rusqlite::Result<()> {
    let conn = Connection::open(DB_FILENAME)?;

    // table of all mensa names
    conn.prepare(
        "create table if not exists mensen (
            mensa_id integer primary key,
            mensa_name text not null unique
        )",
    )?
    .execute([])?;

    // table of all meals
    conn.prepare(
        "create table if not exists meals (
            mensa_id integer,
            date text,
            json_text text,
            foreign key (mensa_id) references mensen(mensa_id)
        )",
    )?
    .execute([])?;

    Ok(())
}

pub fn init_mensa_id_db(mensen: &BTreeMap<u8, String>) -> rusqlite::Result<()> {
    let conn = Connection::open(DB_FILENAME)?;
    let mut stmt = conn.prepare_cached(
        "replace into mensen (mensa_id, mensa_name)
            values (?1, ?2)",
    )?;

    for (id, name) in mensen.iter() {
        stmt.execute(params![id.to_string(), name])?;
    }

    Ok(())
}

pub async fn save_meal_to_db(date: &str, mensa: u8, json_text: &str) -> rusqlite::Result<()> {
    let conn = Connection::open(DB_FILENAME)?;
    conn.execute(
        "delete from meals where mensa_id = ?1 and date = ?2",
        [mensa.to_string(), date.to_string()],
    )?;

    let mut stmt = conn.prepare_cached(
        "insert into meals (mensa_id, date, json_text)
            values (?1, ?2, ?3)",
    )?;

    stmt.execute(params![mensa, date, json_text])?;

    Ok(())
}

pub async fn get_meals_from_db(requested_date: NaiveDate, mensa: u8) -> Result<Vec<MealGroup>> {
    let date_str = build_date_string(requested_date);
    let json_text = get_meal_from_db(&date_str, mensa).await?;
    if let Some(json_text) = json_text {
        json_to_meal(&json_text).await
    } else {
        Ok(vec![])
    }
}

async fn json_to_meal(json_text: &str) -> Result<Vec<MealGroup>> {
    Ok(serde_json::from_str(json_text)?)
}

pub async fn get_meal_from_db(date: &str, mensa: u8) -> rusqlite::Result<Option<String>> {
    let conn = Connection::open(DB_FILENAME)?;
    let mut stmt =
        conn.prepare_cached("select json_text from meals where (mensa_id, date) = (?1, ?2)")?;
    let mut rows = stmt.query([&mensa.to_string(), date])?;

    Ok(rows.next().unwrap().map(|row| row.get(0).unwrap()))
}

pub fn mensa_name_get_id_db(mensa_name: &str) -> rusqlite::Result<Option<u8>> {
    let conn = Connection::open(DB_FILENAME)?;
    let mut stmt = conn.prepare_cached("SELECT (mensa_id) FROM mensen WHERE mensa_name = ?1")?;

    let mut id_iter = stmt.query_map([mensa_name], |row| row.get(0))?;

    Ok(id_iter.next().map(|row| row.unwrap()))
}
