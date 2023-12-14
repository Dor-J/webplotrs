// SQLite interaction logic
use sqlite;
use serde::{Serialize, Deserialize};
use serde_json;
use csv;
use quick_xml;
use calamine::{Reader, open_workbook_auto};
use std::error::Error;
use std::io::Cursor;

// Function to create and return a SQLite connection
pub fn create_connection(db_path: &str) -> Result<sqlite::Connection, Box<dyn Error>> {
    let conn = sqlite::open(db_path)?;
    Ok(conn)
}

// Function to execute a query
pub fn execute_query(conn: &sqlite::Connection, query: &str) -> Result<(), Box<dyn Error>> {
    conn.execute(query)?;
    Ok(())
}

// Insert data from JSON
// Assuming T is a serializable Rust structure
pub fn insert_from_json<T: Serialize>(conn: &sqlite::Connection, table_name: &str, data: &T) -> Result<(), Box<dyn Error>> {
    let json_str = serde_json::to_string(data)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_str)?;

    match json_value {
        serde_json::Value::Object(map) => {
            for (key, value) in map.iter() {
                let insert_query = format!("INSERT INTO {} ({}) VALUES (?);", table_name, key);
                conn.execute(&insert_query, &[value])?;
            }
        },
        _ => return Err(Box::new(sqlite::Error::from("Invalid JSON format for insertion")))
    }
    Ok(())
}


// Insert data from CSV
use std::io::Read;

pub fn insert_from_csv<R: Read>(conn: &sqlite::Connection, table_name: &str, csv_reader: R) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(csv_reader);
    let headers = rdr.headers()?.clone();

    for result in rdr.records() {
        let record = result?;
        let placeholders = std::iter::repeat("?").take(record.len()).collect::<Vec<_>>().join(",");
        let insert_query = format!("INSERT INTO {} ({}) VALUES ({});", table_name, headers.join(","), placeholders);

        let params = record.iter().map(|s| s as &dyn rusqlite::ToSql).collect::<Vec<_>>();
        conn.execute(&insert_query, params)?;
    }
    Ok(())
}


// Insert data from XML
pub fn insert_from_xml(conn: &sqlite::Connection, table_name: &str, xml_data: &str) -> Result<(), Box<dyn Error>> {
    let reader = quick_xml::Reader::from_str(xml_data);
    let mut buf = Vec::new();

    for event in reader.into_iter() {
        match event? {
            quick_xml::events::Event::Start(ref e) => {
                // Logic to handle start of an XML element
            },
            quick_xml::events::Event::Text(e) => {
                // Logic to handle text within an XML element
            },
            quick_xml::events::Event::End(ref e) => {
                // Logic to handle end of an XML element
                let end_tag = e.name();
                // Construct and execute SQL insertion based on the parsed XML data
            },
            _ => {}
        }
    }
    Ok(())
}


// Insert data from Excel (xlsx)
pub fn insert_from_excel(conn: &sqlite::Connection, table_name: &str, excel_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut reader = open_workbook_auto(Cursor::new(excel_data))?;
    if let Some(Ok(range)) = reader.worksheet_range("Sheet1") {
        for row in range.rows() {
            let placeholders = std::iter::repeat("?").take(row.len()).collect::<Vec<_>>().join(",");
            let insert_query = format!("INSERT INTO {} VALUES ({});", table_name, placeholders);

            let params = row.iter().map(|cell| cell.to_string() as &dyn rusqlite::ToSql).collect::<Vec<_>>();
            conn.execute(&insert_query, params)?;
        }
    }
    Ok(())
}


// Function to insert data similar to a Pandas DataFrame or JavaScript object
use serde_json::Value;
use rusqlite::{params, Connection, Error as RusqliteError};

// Assuming T is a serializable Rust structure representing the DataFrame
use serde_json::Value;
use rusqlite::{params, Connection, Error as RusqliteError};

// Assuming T is a serializable Rust structure representing the DataFrame
pub fn insert_from_dataframe<T: Serialize>(conn: &Connection, table_name: &str, data: &T) -> Result<(), RusqliteError> {
    // Serialize the data structure into JSON
    let serialized_data = serde_json::to_string(data)?;

    // Parse the JSON string to serde_json::Value
    let json_data: Value = serde_json::from_str(&serialized_data)?;

    if let Value::Array(rows) = json_data {
        let transaction = conn.transaction()?;

        for row in rows.iter() {
            if let Value::Object(columns) = row {
                let keys: Vec<String> = columns.keys().cloned().collect();
                let values: Vec<&Value> = columns.values().collect();

                let placeholders: String = keys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let insert_query = format!("INSERT INTO {} ({}) VALUES ({});", table_name, keys.join(","), placeholders);

                // Convert serde_json::Value to rusqlite::ToSql
                let params = values.iter().map(|v| serde_json::to_value(v).unwrap()).collect::<Vec<_>>();

                transaction.execute(&insert_query, params.as_slice())?;
            } else {
                return Err(RusqliteError::ExecuteReturnedResults);
            }
        }

        transaction.commit()?;
    } else {
        return Err(RusqliteError::ExecuteReturnedResults);
    }
    Ok(())
}


// Add more utility functions as needed for SQL operations, data conversions, etc.