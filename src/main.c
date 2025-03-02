#include "editor.h"
#include "buffer.h"
#include "editor_ops.h"
#include "display.h"
#include "file_io.h"
#include "undo.h"

int main(int argc, char *argv[]) {
    EditorState state = {0};
    initBuffer(&state.buffer);
    initUndoSystem();
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
                    showHelp(editorWin, &state);
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