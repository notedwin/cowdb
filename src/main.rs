use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

const TABLE_MAX_ROWS: usize = 100;

enum StatementType {
    Insert,
    Select,
    Truncate
}

struct Statement {
    type_: StatementType,
    row_insert: Row,
}
#[derive(Default, Serialize, Deserialize)]
struct Row {
    id: u32,
    username: String,
    email: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Table {
    name: String,
    num_rows: usize,
    rows: Vec<Row>,
}

impl Table {
    fn new(name: Option<String>) -> Table {
        return Table {
            name: name.unwrap_or("fct".to_owned()),
            num_rows: 0,
            rows: vec![],
        };
    }

    fn open_file(filename: &str) -> Result<File, String> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .map_err(|e| e.to_string())?;
        Ok(file)
    }

    fn db_close(&mut self) -> Result<(), String> {
        let serialized_bytes = bincode::serialize(&self).expect("Serialization failed");
        let mut file = Table::open_file(&self.name).unwrap();
        file.write_all(&serialized_bytes)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn db_open(filename: String) -> Result<Table, String> {
    let mut file = match Table::open_file(&filename).map_err(|e| e.to_string()) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut buffer = vec![];
    if let Err(e) = file.read_to_end(&mut buffer) {
        return Err(e.to_string());
    }

    if buffer.is_empty() {
        return Ok(Table::new(None));
    }

    match bincode::deserialize(&buffer) {
        Ok(deserialized_table) => Ok(deserialized_table),
        Err(e) => Err(format!("Deserialization failed: {}", e)),
    }
}

fn prepare_statement(input: &str) -> Result<Statement, String> {
    let mut iter = input.split_whitespace();
    if let Some(command) = iter.next() {
        match command {
            "insert" => {
                // Parse the remaining words based on the expected format
                if let (Some(id), Some(username), Some(email)) =
                    (iter.next(), iter.next(), iter.next())
                {
                    if let Ok(id) = id.parse::<u32>() {
                        return Ok(Statement {
                            type_: StatementType::Insert,
                            row_insert: Row {
                                id: id,
                                username: username.to_owned(),
                                email: email.to_owned(),
                            },
                        });
                    } else {
                        return Err("Invalid ID format".to_owned());
                    }
                } else {
                    return Err("Invalid input format".to_owned());
                }
            }
            "select" => {
                return Ok(Statement {
                    type_: StatementType::Select,
                    row_insert: Row::default(),
                });
            }
            "truncate" => {
                return Ok(Statement {
                    type_: StatementType::Truncate,
                    row_insert: Row::default(),
                });
            }
            _ => {
                return Err(format!("Unrecognized keyword at start of '{}'.", input));
            }
        }
    }
    return Err(format!("Unrecognized input '{}'.", input));
}

fn execute_statement(statement: Statement, table: &mut Table) -> Result<(), String> {
    match statement.type_ {
        StatementType::Insert => {
            if table.num_rows >= TABLE_MAX_ROWS {
                return Err("Table full.".to_owned());
            }
            let row_to_insert = statement.row_insert;
            let row_num = table.num_rows;
            table.rows.insert(row_num, row_to_insert);
            table.num_rows += 1;
            return Ok(());
        }
        StatementType::Select => {
            for i in 0..table.num_rows {
                let row = &table.rows[i];
                println!("({}, {}, {})", row.id, row.username, row.email);
            }
            Ok(())
        }
        StatementType::Truncate => {
            table.rows.clear();
            table.num_rows = 0;
            Ok(())
        }
    }
}

fn handle_valid_input(input: String, table: &mut Table) -> Result<(), String> {
    if input.starts_with(".") {
        if input == ".exit" {
            table.db_close().unwrap();
            std::process::exit(0);
        } else {
            return Err(format!("Unrecognized command '{}'.", input));
        }
    }

    match prepare_statement(&input) {
        Ok(statement) => return execute_statement(statement, table),
        Err(e) => return Err(e),
    }
}

fn main() {
    let mut table = db_open("fct".to_owned()).unwrap();
    // don't worry about history yet
    let mut repl_ = DefaultEditor::new().expect("can't open REPL");
    loop {
        let readline = repl_.readline("db > ");
        match readline {
            Ok(input) => match handle_valid_input(input, &mut table) {
                Ok(_) => println!("Command executed."),
                Err(e) => println!("Error: {}", e),
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            _ => {
                println!("Error");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut table = Table::new(None);
        let statement = Statement {
            type_: StatementType::Insert,
            row_insert: Row {
                id: 1,
                username: "test".to_owned(),
                email: "nah".to_owned(),
            },
        };
        let result = execute_statement(statement, &mut table);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_select() {
        let mut table = Table::new(None);
        let statement = Statement {
            type_: StatementType::Select,
            row_insert: Row {
                id: 0,
                username: "".to_owned(),
                email: "".to_owned(),
            },
        };
        let result = execute_statement(statement, &mut table);
        assert_eq!(result.is_ok(), true);
    }
}
