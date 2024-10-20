#include <stdio.h>
#include <stdlib.h>
#include <ncurses.h>
#include <string.h>
#include <ctype.h>

#define MAX_BUFFER 1000000
#define STATUS_HEIGHT 1
#define EDITOR_HEIGHT (LINES - STATUS_HEIGHT)
#define MAX_HISTORY 100
#define TAB_SIZE 4

const char *keywords[] = {"int", "return", "if", "else", "while", "for", "char", "void", "include", NULL};

typedef struct {
    int x, y;
} Cursor;

typedef struct {
    char *content;
    int length;
    int capacity;
} Buffer;

typedef struct {
    Buffer buffer;
    Cursor cursor;
    int scroll_offset;
    char filename[256];
    int mode; // 0 for edit mode, 1 for command mode
} EditorState;

char undoStack[MAX_HISTORY][MAX_BUFFER];
int undoTop = -1;
char redoStack[MAX_HISTORY][MAX_BUFFER];
int redoTop = -1;

char clipboard[MAX_BUFFER] = {0};

// Function prototypes
void initBuffer(Buffer *buffer);
void appendToBuffer(Buffer *buffer, const char *str);
void insertIntoBuffer(Buffer *buffer, int pos, char ch);
void deleteFromBuffer(Buffer *buffer, int pos);
void freeBuffer(Buffer *buffer);
void saveUndoState(Buffer *buffer);
void undo(Buffer *buffer);
void redo(Buffer *buffer);
void syntaxHighlight(WINDOW *editorWin, const char *line, int length, int cursorX);
void printStatusBar(WINDOW *statusWin, EditorState *state);
void openFile(const char *filename, Buffer *buffer);
void saveFile(const char *filename, Buffer *buffer);
void showHelp(WINDOW *editorWin, EditorState *state);
void insertChar(EditorState *state, char ch);
void deleteChar(EditorState *state);
void moveCursor(EditorState *state, int dx, int dy);
void scrollEditor(EditorState *state);
int getScreenX(EditorState *state);
int getScreenY(EditorState *state);
void copySelection(EditorState *state, int start, int end);
void cutSelection(EditorState *state, int start, int end);
void pasteAtCursor(EditorState *state);

int main(int argc, char *argv[]) {
    EditorState state = {0};
    initBuffer(&state.buffer);
    strcpy(state.filename, argc > 1 ? argv[1] : "untitled.txt");
    
    if (argc > 1) {
        openFile(state.filename, &state.buffer);
    }

    WINDOW *editorWin, *statusWin;

    // Initialize ncurses
    initscr();
    start_color();
    init_pair(1, COLOR_BLUE, COLOR_BLACK);  // Keywords
    init_pair(2, COLOR_CYAN, COLOR_BLACK);  // Numbers
    init_pair(3, COLOR_RED, COLOR_BLACK);   // Strings
    init_pair(4, COLOR_BLACK, COLOR_WHITE); // Cursor highlight
    noecho();
    raw();
    keypad(stdscr, TRUE);

    // Create windows
    editorWin = newwin(EDITOR_HEIGHT, COLS, 0, 0);
    statusWin = newwin(STATUS_HEIGHT, COLS, EDITOR_HEIGHT, 0);

    saveUndoState(&state.buffer);  // Initial state saved for undo

    int ch;
    int selectionStart = -1;

    // Main editing loop
    while (1) {
        // Clear and redraw editor window
        werase(editorWin);
        
        // Display the visible portion of the buffer
        int lineStart = 0;
        for (int i = 0; i < EDITOR_HEIGHT; i++) {
            wmove(editorWin, i, 0);
            int lineEnd = lineStart;
            while (lineEnd < state.buffer.length && state.buffer.content[lineEnd] != '\n') {
                lineEnd++;
            }
            syntaxHighlight(editorWin, state.buffer.content + lineStart, lineEnd - lineStart, 
                            i == getScreenY(&state) ? state.cursor.x : -1);
            lineStart = lineEnd + 1;
            if (lineStart >= state.buffer.length) break;
        }

        // Move cursor to the correct position
        wmove(editorWin, getScreenY(&state), getScreenX(&state));
        
        // Update status bar
        printStatusBar(statusWin, &state);
        
        // Refresh windows
        wrefresh(editorWin);
        wrefresh(statusWin);

        // Get user input
        ch = wgetch(editorWin);
        
        if (state.mode == 0) { // Edit mode
            switch (ch) {
                case KEY_UP:
                case KEY_DOWN:
                case KEY_LEFT:
                case KEY_RIGHT:
                    moveCursor(&state, ch == KEY_LEFT ? -1 : ch == KEY_RIGHT ? 1 : 0,
                               ch == KEY_UP ? -1 : ch == KEY_DOWN ? 1 : 0);
                    break;
                case KEY_BACKSPACE:
                case 127: // ASCII DEL
                    saveUndoState(&state.buffer);
                    deleteChar(&state);
                    break;
                case KEY_DC: // Delete key
                    saveUndoState(&state.buffer);
                    moveCursor(&state, 1, 0);
                    deleteChar(&state);
                    moveCursor(&state, -1, 0);
                    break;
                case '\t':
                    saveUndoState(&state.buffer);
                    for (int i = 0; i < TAB_SIZE; i++) {
                        insertChar(&state, ' ');
                    }
                    break;
                case KEY_ENTER:
                case '\n':
                    saveUndoState(&state.buffer);
                    insertChar(&state, '\n');
                    break;
                case 27: // ESC key
                    state.mode = 1; // Switch to command mode
                    selectionStart = -1; // Clear selection
                    break;
                case KEY_HOME:
                    while (state.cursor.x > 0 && state.buffer.content[state.cursor.y * COLS + state.cursor.x - 1] != '\n') {
                        moveCursor(&state, -1, 0);
                    }
                    break;
                case KEY_END:
                    while (state.cursor.x < COLS - 1 && state.buffer.content[state.cursor.y * COLS + state.cursor.x] != '\n' && state.cursor.y * COLS + state.cursor.x < state.buffer.length) {
                        moveCursor(&state, 1, 0);
                    }
                    break;
                default:
                    if (isprint(ch)) {
                        saveUndoState(&state.buffer);
                        insertChar(&state, ch);
                    }
                    break;
            }
        } else { // Command mode
            switch (ch) {
                case 'q':
                    if (mvwprintw(statusWin, 0, 0, "Save before quit? (y/n)") == ERR) {
                        // Handle error
                    }
                    wrefresh(statusWin);
                    int choice = wgetch(statusWin);
                    if (choice == 'y' || choice == 'Y') {
                        saveFile(state.filename, &state.buffer);
                    }
                    endwin();
                    freeBuffer(&state.buffer);
                    return 0;
                case 's':
                    saveFile(state.filename, &state.buffer);
                    mvwprintw(statusWin, 0, COLS - 20, "File saved");
                    wrefresh(statusWin);
                    break;
                case 'h':
                    showHelp(editorWin, &state);  // Updated to pass EditorState
                    // Redraw the editor window after returning from help
                    redrawwin(editorWin);
                    wrefresh(editorWin);
                    break;
                case 'u':
                    undo(&state.buffer);
                    break;
                case 'r':
                    redo(&state.buffer);
                    break;
                case 'i':
                    state.mode = 0; // Switch back to edit mode
                    break;
                case 'v':
                    if (selectionStart == -1) {
                        selectionStart = state.cursor.y * COLS + state.cursor.x;
                    } else {
                        int selectionEnd = state.cursor.y * COLS + state.cursor.x;
                        copySelection(&state, selectionStart, selectionEnd);
                        selectionStart = -1;
                    }
                    break;
                case 'x':
                    if (selectionStart != -1) {
                        int selectionEnd = state.cursor.y * COLS + state.cursor.x;
                        cutSelection(&state, selectionStart, selectionEnd);
                        selectionStart = -1;
                    }
                    break;
                case 'p':
                    pasteAtCursor(&state);
                    break;
            }
        }

        scrollEditor(&state);
    }

    endwin();
    freeBuffer(&state.buffer);
    return 0;
}

void initBuffer(Buffer *buffer) {
    buffer->capacity = MAX_BUFFER;
    buffer->content = malloc(buffer->capacity);
    buffer->length = 0;
    buffer->content[0] = '\0';
}

void appendToBuffer(Buffer *buffer, const char *str) {
    int len = strlen(str);
    if (buffer->length + len >= buffer->capacity) {
        buffer->capacity *= 2;
        buffer->content = realloc(buffer->content, buffer->capacity);
    }
    strcpy(buffer->content + buffer->length, str);
    buffer->length += len;
}

void insertIntoBuffer(Buffer *buffer, int pos, char ch) {
    if (buffer->length + 1 >= buffer->capacity) {
        buffer->capacity *= 2;
        buffer->content = realloc(buffer->content, buffer->capacity);
    }
    memmove(buffer->content + pos + 1, buffer->content + pos, buffer->length - pos + 1);
    buffer->content[pos] = ch;
    buffer->length++;
}

void deleteFromBuffer(Buffer *buffer, int pos) {
    if (pos < buffer->length) {
        memmove(buffer->content + pos, buffer->content + pos + 1, buffer->length - pos);
        buffer->length--;
    }
}

void freeBuffer(Buffer *buffer) {
    free(buffer->content);
    buffer->content = NULL;
    buffer->length = 0;
    buffer->capacity = 0;
}

void saveUndoState(Buffer *buffer) {
    if (undoTop < MAX_HISTORY - 1) {
        strcpy(undoStack[++undoTop], buffer->content);
        redoTop = -1;  // Reset redo stack on new changes
    }
}

void undo(Buffer *buffer) {
    if (undoTop >= 0) {
        strcpy(redoStack[++redoTop], buffer->content); // Save current state to redo stack
        strcpy(buffer->content, undoStack[undoTop--]); // Restore previous state from undo stack
        buffer->length = strlen(buffer->content);
    }
}

void redo(Buffer *buffer) {
    if (redoTop >= 0) {
        strcpy(undoStack[++undoTop], buffer->content); // Save current state to undo stack
        strcpy(buffer->content, redoStack[redoTop--]); // Restore next state from redo stack
        buffer->length = strlen(buffer->content);
    }
}

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

void openFile(const char *filename, Buffer *buffer) {
    FILE *file = fopen(filename, "r");
    if (file == NULL) {
        return;
    }
    char line[COLS + 1];
    while (fgets(line, sizeof(line), file) != NULL) {
        appendToBuffer(buffer, line);
    }
    fclose(file);
}

void saveFile(const char *filename, Buffer *buffer) {
    FILE *file = fopen(filename, "w");
    if (file == NULL) {
        return;
    }
    fputs(buffer->content, file);
    fclose(file);
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

void insertChar(EditorState *state, char ch) {
    int pos = state->cursor.y * COLS + state->cursor.x;
    insertIntoBuffer(&state->buffer, pos, ch);
    moveCursor(state, 1, 0);
}

void deleteChar(EditorState *state) {
    int pos = state->cursor.y * COLS + state->cursor.x;
    if (pos > 0) {
        moveCursor(state, -1, 0);
        deleteFromBuffer(&state->buffer, pos - 1);
    }
}

void moveCursor(EditorState *state, int dx, int dy) {
    state->cursor.x += dx;
    state->cursor.y += dy;

    if (state->cursor.x < 0) state->cursor.x = 0;
    if (state->cursor.y < 0) state->cursor.y = 0;
    if (state->cursor.x >= COLS) {
        state->cursor.x = 0;
        state->cursor.y++;
    }
    if (state->cursor.y >= LINES - STATUS_HEIGHT) {
        state->cursor.y = LINES - STATUS_HEIGHT - 1;
    }
}

void scrollEditor(EditorState *state) {
    if (state->cursor.y < state->scroll_offset) {
        state->scroll_offset = state->cursor.y;
    } else if (state->cursor.y >= state->scroll_offset + EDITOR_HEIGHT) {
        state->scroll_offset = state->cursor.y - EDITOR_HEIGHT + 1;
    }
}

int getScreenX(EditorState *state) {
    return state->cursor.x;
}

int getScreenY(EditorState *state) {
    return state->cursor.y - state->scroll_offset;
}

void copySelection(EditorState *state, int start, int end) {
    int length = end - start;
    if (length > 0 && length < MAX_BUFFER) {
        strncpy(clipboard, state->buffer.content + start, length);
        clipboard[length] = '\0';
    }
}

void cutSelection(EditorState *state, int start, int end) {
    copySelection(state, start, end);
    int length = end - start;
    if (length > 0) {
        memmove(state->buffer.content + start, state->buffer.content + end, state->buffer.length - end);
        state->buffer.length -= length;
        state->buffer.content[state->buffer.length] = '\0';
    }
}

void pasteAtCursor(EditorState *state) {
    int clipboardLength = strlen(clipboard);
    if (clipboardLength > 0) {
        int pos = state->cursor.y * COLS + state->cursor.x;
        if (state->buffer.length + clipboardLength < MAX_BUFFER) {
            memmove(state->buffer.content + pos + clipboardLength, state->buffer.content + pos, state->buffer.length - pos);
            memcpy(state->buffer.content + pos, clipboard, clipboardLength);
            state->buffer.length += clipboardLength;
            state->buffer.content[state->buffer.length] = '\0';
        }
    }
}