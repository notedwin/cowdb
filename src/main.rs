use rustyline::{error::ReadlineError, Config, Editor, Result};
use std::result as std_result;

const SIZE_OF_ID: usize = std::mem::size_of::<u32>();
const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / SIZE_OF_ID;
const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

enum StatementType {
    Insert,
    Select,
}

struct Statement {
    type_: StatementType,
    row_insert: Row,
}
#[derive(Clone, Default, Debug)]
struct Row {
    id: u32,
    username: String,
    email: String,
}

struct Table {
    num_rows: usize,
    pages: Vec<Page>,
    // pager: Pager,
}

impl Table {
    // fn new(filename: &str) -> Self {
    fn new() -> Self {
        Table {
            num_rows: 0,
            pages: vec![],
            // pager: open_pager(filename).unwrap(),
        }
    }
}

#[derive(Clone)]
struct Page {
    rows: Vec<Row>,
}

impl Page {
    fn new() -> Self {
        Page { rows: vec![] }
    }
}

// struct Pager {
//     file: std::fs::File,
//     file_length: usize,
//     pages: Vec<Page>,
// }


// fn open_pager(filename: &str) -> std_result::Result<Pager, String> {
//     let file = std::fs::OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .open(filename)
//         .map_err(|e| e.to_string())?;
//     let file_length = file
//         .metadata()
//         .map_err(|e| e.to_string())?
//         .len() as usize;
//     let pages = (0..TABLE_MAX_PAGES).map(|_| Page::new()).collect();
//     Ok(Pager {
//         file,
//         file_length,
//         pages,
//     })
// }



fn table_row_slot(table: &Table, row_num: usize) -> &Row {
    let page_num = row_num / ROWS_PER_PAGE;
    let row_offset = row_num % ROWS_PER_PAGE;
    &table.pages[page_num].rows[row_offset]
}

// serialize row using memcpy
fn serialize_row(source: &Row, destination: &mut Vec<u8>) {
    let id_bytes = source.id.to_le_bytes();
    let username_bytes = source.username.as_bytes();
    let email_bytes = source.email.as_bytes();
    destination.extend_from_slice(&id_bytes);
    destination.extend_from_slice(username_bytes);
    destination.extend_from_slice(email_bytes);
}

// fn deserialize_row(source: &[u8]) -> Row {
//     let id = u32::from_le_bytes([source[0], source[1], source[2], source[3]]);
//     let username_start = SIZE_OF_ID;
//     let username_end = username_start + source[username_start..]
//         .iter()
//         .position(|&x| x == 0)
//         .unwrap();
//     let username = String::from_utf8(source[username_start..username_end].to_vec()).unwrap();
//     let email_start = username_end + 1;
//     let email_end = email_start + source[email_start..]
//         .iter()
//         .position(|&x| x == 0)
//         .unwrap();
//     let email = String::from_utf8(source[email_start..email_end].to_vec()).unwrap();
//     Row {
//         id,
//         username,
//         email,
//     }
// }

fn do_meta_command(input: &str) -> std_result::Result<(), String> {
    if input == ".exit" {
        std::process::exit(0);
    } else {
        Err(format!("Unrecognized command '{}'.", input))
    }
}

fn prepare_statement(input: &str) -> std_result::Result<Statement, String> {
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
            _ => {
                return Err(format!("Unrecognized keyword at start of '{}'.", input));
            }
        }
    }
    return Err(format!("Unrecognized input '{}'.", input));
}

fn execute_statement(statement: Statement, table: &mut Table) -> std_result::Result<(), String> {
    match statement.type_ {
        StatementType::Insert => {
            if table.num_rows >= TABLE_MAX_ROWS {
                return Err("Table full.".to_owned());
            }
            let row_to_insert = statement.row_insert;
            let mut destination =
                vec![0u8; SIZE_OF_ID + row_to_insert.username.len() + row_to_insert.email.len()];
            serialize_row(&row_to_insert, &mut destination);
            let row_num = table.num_rows;
            let page_num = row_num / ROWS_PER_PAGE;
            let row_offset = row_num % ROWS_PER_PAGE;

            if page_num >= table.pages.len() {
                table.pages.push(Page::new());
            }

            table.pages[page_num].rows.insert(row_offset, row_to_insert);
            table.num_rows += 1;
            return Ok(());
        }
        StatementType::Select => {
            for i in 0..table.num_rows {
                let row = table_row_slot(table, i);
                println!("{:?}", row);
            }
            Ok(())
        }
    }
}

fn handle_valid_input(input: String, table: &mut Table) -> std::result::Result<(), String> {
    if input.starts_with(".") {
        return do_meta_command(&input);
    }

    match prepare_statement(&input) {
        Ok(statement) => return execute_statement(statement, table),
        Err(e) => return Err(e),
    }
}

fn main() -> Result<()> {
    let mut table = Table::new();

    let config = Config::builder().auto_add_history(true).build();
    let history = rustyline::sqlite_history::SQLiteHistory::open(config, "history.sqlite3")?;

    let mut repl_: Editor<(), _> = Editor::with_history(config, history).unwrap();

    loop {
        let readline = repl_.readline("db > ");

        match readline {
            Ok(input) => {
                repl_.add_history_entry(input.as_str())?;
                match handle_valid_input(input, &mut table) {
                    Ok(_) => println!("Command executed."),
                    Err(e) => println!("Error: {}", e),
                }
            }
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_row_slot() {
        let table = Table {
            num_rows: 0,
            pages: vec![
                Page {
                    rows: vec![
                        Row {
                            id: 0,
                            username: "bruh".to_owned(),
                            email: "bruh".to_owned(),
                        };
                        ROWS_PER_PAGE
                    ]
                };
                TABLE_MAX_PAGES
            ],
        };
        let row = table_row_slot(&table, 0);
        assert_eq!(row.id, 0);
        assert_eq!(row.username, "bruh".to_owned());
        assert_eq!(row.email, "bruh".to_owned());
    }

    #[test]
    fn test_prepare_statement() {
        let statement = prepare_statement("insert 1 test nah");
        assert_eq!(statement.is_ok(), true);
        let statement = prepare_statement("insert 1 test");
        assert_eq!(statement.err().unwrap(), "Invalid input format");
        let statement = prepare_statement("insert -1 test nah nah");
        // rust handles negative numbers since they can't be parsed as u32
        assert_eq!(statement.err().unwrap(), "Invalid ID format");

        let statement = prepare_statement("select");
        assert_eq!(statement.is_ok(), true);
        let statement = prepare_statement("select 1");
        assert_eq!(statement.is_ok(), true);
    }

    #[test]
    fn test_execute_statement() {
        let mut table = Table {
            num_rows: 0,
            pages: vec![
                Page {
                    rows: vec![
                        Row {
                            id: 0,
                            username: "".to_owned(),
                            email: "".to_owned(),
                        };
                        ROWS_PER_PAGE
                    ]
                };
                TABLE_MAX_PAGES
            ],
        };
        let statement = Statement {
            type_: StatementType::Insert,
            row_insert: Row {
                id: 1,
                username: "reallylongnamethatshouldfailbruhhhhhhhhh".to_owned(),
                email: "reallylongnamethatshouldfailbruh".to_owned(),
            },
        };
        let result = execute_statement(statement, &mut table);
        // rust handles long strings since we don't have a max length
        assert_eq!(result.is_ok(), true);
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
