use std::io::{stdin, stdout, Write};

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
#[derive(Clone)]
struct Row {
    id: u32,
    username: String,
    email: String,
}

struct Table {
    num_rows: usize,
    pages: Vec<Page>,
}
#[derive(Clone)]
struct Page {
    rows: Vec<Row>,
}

fn table_row_slot(table: &Table, row_num: usize) -> &Row {
    let page_num = row_num / ROWS_PER_PAGE;
    let row_offset = row_num % ROWS_PER_PAGE;
    &table.pages[page_num].rows[row_offset]
}

// serialize row using memcpy
fn serialize_row(source: &Row, destination: &mut [u8]) {
    let mut offset = 0;
    let id_bytes = source.id.to_le_bytes();
    let username_bytes = source.username.as_bytes();
    let email_bytes = source.email.as_bytes();
    destination[..SIZE_OF_ID].copy_from_slice(&id_bytes);
    offset += SIZE_OF_ID;
    destination[offset..offset + username_bytes.len()].copy_from_slice(username_bytes);
    offset += username_bytes.len();
    destination[offset..offset + email_bytes.len()].copy_from_slice(email_bytes);
}

fn do_meta_command(input: &str) -> Result<(), String> {
    if input == ".exit" {
        std::process::exit(0);
    } else {
        Err(format!("Unrecognized command '{}'.", input))
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
                        // Use the parsed values as needed
                        let new_row: Row = Row {
                            id: id,
                            username: username.to_owned(),
                            email: email.to_owned(),
                        };
                        println!("ID: {}", id);
                        println!("Username: {}", username);
                        println!("Email: {}", email);
                        return Ok(Statement {
                            type_: StatementType::Insert,
                            row_insert: new_row,
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
                    row_insert: Row {
                        id: 0,
                        username: "".to_owned(),
                        email: "".to_owned(),
                    },
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
            let mut destination =
                vec![0u8; SIZE_OF_ID + row_to_insert.username.len() + row_to_insert.email.len()];
            serialize_row(&row_to_insert, &mut destination);
            let row_num = table.num_rows;
            let page_num = row_num / ROWS_PER_PAGE;
            let row_offset = row_num % ROWS_PER_PAGE;
            table.pages[page_num].rows[row_offset] = row_to_insert;
            table.num_rows += 1;
            return Ok(());
        }
        StatementType::Select => {
            for i in 0..table.num_rows {
                let row = table_row_slot(table, i);
                println!("({}, {}, {})", row.id, row.username, row.email);
            }
            Ok(())
        }
    }
}

fn main() {
    let mut input: String = String::new();
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

    loop {
        print!("\ndb >");
        stdout().flush().unwrap();

        input.clear();
        stdin().read_line(&mut input).unwrap();
        // remove trailing newline
        input.pop();

        if input.starts_with(".") {
            match do_meta_command(&input) {
                Ok(_) => continue,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
        }

        match prepare_statement(&input) {
            Ok(statement) => match execute_statement(statement, &mut table) {
                Ok(_) => continue,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            },
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_row() {
        let row = Row {
            id: 1,
            username: "test".to_owned(),
            email: "nah".to_owned(),
        };
        let mut destination = vec![0u8; SIZE_OF_ID + row.username.len() + row.email.len()];
        serialize_row(&row, &mut destination);
        assert_eq!(
            destination,
            vec![1, 0, 0, 0, 116, 101, 115, 116, 110, 97, 104]
        );
    }

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
        assert_eq!(
            statement.err().unwrap(),
            "Invalid input format"
        );
        let statement = prepare_statement("insert -1 test nah nah");
        // rust handles negative numbers since they can't be parsed as u32
        assert_eq!(
            statement.err().unwrap(),
            "Invalid ID format"
        );

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
