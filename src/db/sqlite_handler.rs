use rusqlite::{params, Connection, Result as SqlResult, Error as SqlError, Row};
use serde::Serialize;
use serde_json::{self, Value};
use csv::{Reader as CsvReader, Trim};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use calamine::{open_workbook_auto, Reader};
use std::io::{Read, Cursor};

/// Creates a new connection to an SQLite database.
///
/// # Arguments
/// * `db_path` - The file path to the SQLite database.
///
/// # Returns
/// A result containing the database connection or an error.
pub fn create_connection(db_path: &str) -> SqlResult<Connection> {
    Connection::open(db_path)
}

/// Executes a given SQL query on the provided database connection.
///
/// # Arguments
/// * `conn` - A reference to the SQLite connection.
/// * `query` - The SQL query to be executed.
///
/// # Returns
/// A result indicating success or failure.
pub fn execute_query(conn: &Connection, query: &str) -> SqlResult<()> {
    conn.execute(query, params![])?;
    Ok(())
}

/// Inserts data from a JSON string into the specified table.
///
/// # Arguments
/// * `conn` - A reference to the SQLite connection.
/// * `table_name` - The name of the table to insert data into.
/// * `data` - The JSON string representing the data to insert.
///
/// # Returns
/// A result indicating success or failure.
pub fn insert_from_json<T: Serialize>(conn: &Connection, table_name: &str, data: &T) -> SqlResult<()> {
    let json_str = serde_json::to_string(data)?;
    let json_value: Value = serde_json::from_str(&json_str)?;

    if let Value::Object(map) = json_value {
        let transaction = conn.transaction()?;
        for (key, value) in map.iter() {
            let insert_query = format!("INSERT INTO {} ({}) VALUES (?);", table_name, key);
            transaction.execute(&insert_query, &[value])?;
        }
        transaction.commit()?;
    } else {
        return Err(SqlError::ExecuteReturnedResults);
    }
    Ok(())
}

// Continue with more functions...
