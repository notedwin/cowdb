// simple hello world

#include <iostream>
using namespace std;

// create an enum for meta commands
enum MetaCommandResult {
	META_COMMAND_SUCCESS,
	META_COMMAND_UNRECOGNIZED_COMMAND
};

enum PrepareResult {
	PREPARE_SUCCESS,
	PREPARE_UNRECOGNIZED_STATEMENT
};

enum StatementType {
	STATEMENT_INSERT,
	STATEMENT_SELECT
};

struct Statement {
	StatementType type;
};

MetaCommandResult do_meta_command(string command) {
	if (command == ".exit") {
		exit(0);
	} else {
		return META_COMMAND_UNRECOGNIZED_COMMAND;
	}
}

PrepareResult prepare_statement(string input, Statement* statement) {
	if (input.substr(0, 6) == "insert") {
		statement->type = STATEMENT_INSERT;
		return PREPARE_SUCCESS;
	}
	if (input.substr(0, 6) == "select") {
		statement->type = STATEMENT_SELECT;
		return PREPARE_SUCCESS;
	}
	return PREPARE_UNRECOGNIZED_STATEMENT;
}

void execute_statement(Statement* statement) {
	switch (statement->type) {
		case (STATEMENT_INSERT):
			cout << "This is where we would do an insert." << endl;
			break;
		case (STATEMENT_SELECT):
			cout << "This is where we would do a select." << endl;
			break;
	}
}

void repl() {
	string input;
	while (true) {
		cout << "db > ";
		getline(cin, input);

		if (input[0] == '.') {
			switch (do_meta_command(input)) {
				case (META_COMMAND_SUCCESS):
					continue;
				case (META_COMMAND_UNRECOGNIZED_COMMAND):
					cout << "Unrecognized command '" << input << "'." << endl;
					continue;
			}
		}

		Statement statement;
		switch (prepare_statement(input, &statement)) {
			case PREPARE_SUCCESS:
				execute_statement(&statement);
				break;
			case PREPARE_UNRECOGNIZED_STATEMENT:
				continue;
		}

		cout << "SQL command: " << input << endl;
	}
}

int main() {
	repl();
	return 0;
}