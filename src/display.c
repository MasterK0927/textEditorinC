#include "display.h"

const char *keywords[] = {"int", "return", "if", "else", "while", "for", "char", "void", "include", NULL};

void syntaxHighlight(WINDOW *editorWin, const char *line, int length, int cursorX) {
    int i = 0;
    while (i < length) {
        if (i == cursorX) {
            wattron(editorWin, COLOR_PAIR(4)); // Highlight cursor position
            waddch(editorWin, line[i]);
            wattroff(editorWin, COLOR_PAIR(4));
            i++;
        } else if (isalpha(line[i])) {
            // Check for keywords
            int j = 0;
            while (keywords[j] != NULL) {
                if (strncmp(&line[i], keywords[j], strlen(keywords[j])) == 0 &&
                   (i + strlen(keywords[j]) >= length || !isalnum(line[i + strlen(keywords[j])]))) {
                    wattron(editorWin, COLOR_PAIR(1)); // Set color for keyword
                    waddnstr(editorWin, keywords[j], strlen(keywords[j]));
                    wattroff(editorWin, COLOR_PAIR(1));
                    i += strlen(keywords[j]);
                    break;
                }
                j++;
            }
            if (keywords[j] == NULL) {
                waddch(editorWin, line[i++]);
            }
        } else if (isdigit(line[i])) {
            // Highlight numbers
            wattron(editorWin, COLOR_PAIR(2));
            while (i < length && isdigit(line[i])) {
                waddch(editorWin, line[i++]);
            }
            wattroff(editorWin, COLOR_PAIR(2));
        } else if (line[i] == '"') {
            // Highlight strings
            wattron(editorWin, COLOR_PAIR(3));
            waddch(editorWin, line[i++]);
            while (i < length && line[i] != '"') {
                waddch(editorWin, line[i++]);
            }
            if (i < length && line[i] == '"') {
                waddch(editorWin, line[i++]); // Close quote
            }
            wattroff(editorWin, COLOR_PAIR(3));
        } else {
            waddch(editorWin, line[i++]); // Default case: print character as is
        }
    }
}

void printStatusBar(WINDOW *statusWin, EditorState *state) {
    werase(statusWin);
    mvwprintw(statusWin, 0, 0, "File: %s | Position: %d:%d | Mode: %s",
              state->filename, state->cursor.y + 1, state->cursor.x + 1, state->mode ? "Command" : "Edit");
    wrefresh(statusWin);
}

void listCommands(WINDOW *editorWin) {
    const char *commands[] = {
        "insert", "delete", "move", "undo", "redo", "copy", "cut", "paste",
        "save", "quit", "search", "replace", "exit", NULL
    };

    int y = 2;
    mvwprintw(editorWin, y++, 0, "Available commands:");
    for (int i = 0; commands[i] != NULL; i++) {
        mvwprintw(editorWin, y++, 2, "- %s", commands[i]);
    }
}

int displayCodeSnippet(WINDOW *editorWin, int y, const char *code) {
    mvwprintw(editorWin, y, 4, "%s", code);
    return y + 1;
}

void displayHelpForCommand(WINDOW *editorWin, const char *command) {
    int y = 2;

    if (strcmp(command, "insert") == 0) {
        mvwprintw(editorWin, y++, 0, "Insert: Adds a character at the current cursor position.");
        mvwprintw(editorWin, y++, 0, "Implementation:");
        y = displayCodeSnippet(editorWin, y, "void insertChar(EditorState *state, char ch) {");
        y = displayCodeSnippet(editorWin, y, "    int pos = state->cursor.y * COLS + state->cursor.x;");
        y = displayCodeSnippet(editorWin, y, "    insertIntoBuffer(&state->buffer, pos, ch);");
        y = displayCodeSnippet(editorWin, y, "    moveCursor(state, 1, 0);");
        y = displayCodeSnippet(editorWin, y, "}");
        y++;
        mvwprintw(editorWin, y++, 0, "This function calculates the position in the buffer based on");
        mvwprintw(editorWin, y++, 0, "the cursor's x and y coordinates, inserts the character, and");
        mvwprintw(editorWin, y++, 0, "moves the cursor one position to the right.");
    } else if (strcmp(command, "delete") == 0) {
        mvwprintw(editorWin, y++, 0, "Delete: Removes the character before the cursor.");
        mvwprintw(editorWin, y++, 0, "Implementation:");
        y = displayCodeSnippet(editorWin, y, "void deleteChar(EditorState *state) {");
        y = displayCodeSnippet(editorWin, y, "    int pos = state->cursor.y * COLS + state->cursor.x;");
        y = displayCodeSnippet(editorWin, y, "    if (pos > 0) {");
        y = displayCodeSnippet(editorWin, y, "        moveCursor(state, -1, 0);");
        y = displayCodeSnippet(editorWin, y, "        deleteFromBuffer(&state->buffer, pos - 1);");
        y = displayCodeSnippet(editorWin, y, "    }");
        y = displayCodeSnippet(editorWin, y, "}");
        y++;
        mvwprintw(editorWin, y++, 0, "This function first checks if there's a character to delete,");
        mvwprintw(editorWin, y++, 0, "then moves the cursor back and removes the character from the buffer.");
    }
}

void showHelp(WINDOW *editorWin, EditorState *state) {
    werase(editorWin);
    mvwprintw(editorWin, 0, 0, "Help System");
    mvwprintw(editorWin, 2, 0, "Enter the name of the functionality you want to know about:");
    mvwprintw(editorWin, 3, 0, "(e.g., 'insert', 'delete', 'undo', 'redo', 'copy', 'paste', 'save', 'quit')");
    mvwprintw(editorWin, 4, 0, "Or type 'list' to see all available commands.");
    mvwprintw(editorWin, 5, 0, "Type 'exit' to return to the editor.");

    char input[50];
    int y = 7;

    while (1) {
        mvwprintw(editorWin, y, 0, "> ");
        wrefresh(editorWin);
        echo();
        wgetnstr(editorWin, input, sizeof(input) - 1);
        noecho();

        if (strcmp(input, "exit") == 0) {
            break;
        }

        werase(editorWin);
        mvwprintw(editorWin, 0, 0, "Help: %s", input);

        if (strcmp(input, "list") == 0) {
            listCommands(editorWin);
        } else {
            displayHelpForCommand(editorWin, input);
        }

        mvwprintw(editorWin, LINES - 3, 0, "Press any key to continue or type another command.");
        wrefresh(editorWin);
        y = LINES - 2;
    }
}