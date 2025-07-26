#include "editor.h"
#include "buffer.h"
#include "editor_ops.h"
#include "display.h"
#include "file_io.h"
#include "undo.h"

// Command buffer for vim-like commands
char commandBuffer[256] = {0};
int commandMode = 0; // 0 = normal command mode, 1 = command input mode

void printUsage(const char* programName) {
    printf("Usage: %s [options] [file1] [file2] ...\n", programName);
    printf("Options:\n");
    printf("  -r, --readonly    Open files in read-only mode\n");
    printf("  -h, --help        Show this help message\n");
    printf("\nVim-like commands:\n");
    printf("  :e <file>         Edit/open file\n");
    printf("  :w                Write/save file\n");
    printf("  :q                Quit\n");
    printf("  :wq               Write and quit\n");
    printf("  ESC               Exit command mode\n");
}

int parseArguments(int argc, char *argv[], EditorState *state, int *readonly) {
    int fileCount = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-r") == 0 || strcmp(argv[i], "--readonly") == 0) {
            *readonly = 1;
        } else if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            printUsage(argv[0]);
            return -1; // Signal to exit
        } else {
            // This is a filename
            if (fileCount == 0) {
                strcpy(state->filename, argv[i]);
                openFile(state->filename, &state->buffer);
                fileCount++;
            }
            // For simplicity, C version only handles one file at a time
            // Additional files are ignored with a warning
            else {
                printf("Warning: Multiple files specified. Only opening '%s'\n", state->filename);
            }
        }
    }

    if (fileCount == 0) {
        strcpy(state->filename, "untitled.txt");
    }

    return fileCount;
}

void executeCommand(EditorState *state, WINDOW *statusWin, const char *cmd) {
    char command[256];
    char filename[256];

    // Parse command
    if (sscanf(cmd, "%s %s", command, filename) < 1) {
        mvwprintw(statusWin, 0, 0, "Invalid command");
        wrefresh(statusWin);
        return;
    }

    if (strcmp(command, "w") == 0 || strcmp(command, "write") == 0) {
        if (strlen(filename) > 0) {
            // Save as different filename
            strcpy(state->filename, filename);
        }
        saveFile(state->filename, &state->buffer);
        mvwprintw(statusWin, 0, 0, "File saved: %s", state->filename);
    }
    else if (strcmp(command, "e") == 0 || strcmp(command, "edit") == 0) {
        if (strlen(filename) > 0) {
            strcpy(state->filename, filename);
            freeBuffer(&state->buffer);
            initBuffer(&state->buffer);
            openFile(state->filename, &state->buffer);
            mvwprintw(statusWin, 0, 0, "Opened: %s", state->filename);
        } else {
            mvwprintw(statusWin, 0, 0, "Usage: :e <filename>");
        }
    }
    else if (strcmp(command, "o") == 0 || strcmp(command, "open") == 0) {
        if (strlen(filename) > 0) {
            strcpy(state->filename, filename);
            freeBuffer(&state->buffer);
            initBuffer(&state->buffer);
            openFile(state->filename, &state->buffer);
            mvwprintw(statusWin, 0, 0, "Opened: %s", state->filename);
        } else {
            mvwprintw(statusWin, 0, 0, "Usage: :o <filename>");
        }
    }
    else {
        mvwprintw(statusWin, 0, 0, "Unknown command: %s", command);
    }

    wrefresh(statusWin);
}

int main(int argc, char *argv[]) {
    EditorState state = {0};
    int readonly = 0;

    initBuffer(&state.buffer);
    initUndoSystem();

    // Parse command line arguments
    int result = parseArguments(argc, argv, &state, &readonly);
    if (result == -1) {
        return 0; // Help was displayed, exit normally
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
    keypad(editorWin, TRUE);
    keypad(statusWin, TRUE);

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
                    if (!readonly) {
                        saveUndoState(&state.buffer);
                        deleteChar(&state);
                    }
                    break;
                case KEY_DC: // Delete key
                    if (!readonly) {
                        saveUndoState(&state.buffer);
                        moveCursor(&state, 1, 0);
                        deleteChar(&state);
                        moveCursor(&state, -1, 0);
                    }
                    break;
                case '\t':
                    if (!readonly) {
                        saveUndoState(&state.buffer);
                        for (int i = 0; i < TAB_SIZE; i++) {
                            insertChar(&state, ' ');
                        }
                    }
                    break;
                case KEY_ENTER:
                case '\n':
                    if (!readonly) {
                        saveUndoState(&state.buffer);
                        insertChar(&state, '\n');
                    }
                    break;
                case 27: // ESC key
                    state.mode = 1; // Switch to command mode
                    selectionStart = -1; // Clear selection
                    commandMode = 0; // Reset command mode
                    memset(commandBuffer, 0, sizeof(commandBuffer));
                    break;
                case ':': // Start command input
                    if (!readonly) {
                        state.mode = 1; // Switch to command mode
                        commandMode = 1; // Enter command input mode
                        strcpy(commandBuffer, ":");
                        mvwprintw(statusWin, 0, 0, ":%s", commandBuffer + 1);
                        wrefresh(statusWin);
                    }
                    break;
                case KEY_HOME:
                    while (state.cursor.x > 0) {
                        int pos = state.cursor.y * COLS + (state.cursor.x - 1);
                        if (pos < 0 || pos >= state.buffer.length) break;
                        if (state.buffer.content[pos] == '\n') break;
                        moveCursor(&state, -1, 0);
                    }
                    break;
                case KEY_END:
                    while (state.cursor.x < COLS - 1) {
                        int pos = state.cursor.y * COLS + state.cursor.x;
                        if (pos < 0 || pos >= state.buffer.length) break;
                        if (state.buffer.content[pos] == '\n') break;
                        moveCursor(&state, 1, 0);
                    }
                    break;
                default:
                    if (isprint(ch) && !readonly) {
                        saveUndoState(&state.buffer);
                        insertChar(&state, ch);
                    }
                    break;
            }
        } else { // Command mode
            if (commandMode == 1) { // Command input mode
                switch (ch) {
                    case '\n':
                    case KEY_ENTER:
                        if (strlen(commandBuffer) > 1) {
                            if (strcmp(commandBuffer, ":q") == 0 || strcmp(commandBuffer, ":quit") == 0) {
                                endwin();
                                freeBuffer(&state.buffer);
                                return 0;
                            } else if (strcmp(commandBuffer, ":wq") == 0) {
                                if (!readonly) {
                                    saveFile(state.filename, &state.buffer);
                                }
                                endwin();
                                freeBuffer(&state.buffer);
                                return 0;
                            } else {
                                executeCommand(&state, statusWin, commandBuffer + 1);
                            }
                        }
                        commandMode = 0;
                        state.mode = 0;
                        memset(commandBuffer, 0, sizeof(commandBuffer));
                        break;
                    case 27: // ESC
                        commandMode = 0;
                        state.mode = 0;
                        memset(commandBuffer, 0, sizeof(commandBuffer));
                        break;
                    case KEY_BACKSPACE:
                    case 127:
                        if (strlen(commandBuffer) > 1) {
                            commandBuffer[strlen(commandBuffer) - 1] = '\0';
                            mvwprintw(statusWin, 0, 0, "%s", commandBuffer);
                            wclrtoeol(statusWin);
                            wrefresh(statusWin);
                        } else {
                            commandMode = 0;
                            state.mode = 0;
                            memset(commandBuffer, 0, sizeof(commandBuffer));
                        }
                        break;
                    default:
                        if (isprint(ch) && strlen(commandBuffer) < sizeof(commandBuffer) - 1) {
                            strncat(commandBuffer, (char*)&ch, 1);
                            mvwprintw(statusWin, 0, 0, "%s", commandBuffer);
                            wrefresh(statusWin);
                        }
                        break;
                }
            } else { // Normal command mode
                switch (ch) {
                    case 'q':
                        if (mvwprintw(statusWin, 0, 0, "Save before quit? (y/n)") == ERR) {
                            // Handle error
                        }
                        wrefresh(statusWin);
                        int choice = wgetch(statusWin);
                        if (choice == 'y' || choice == 'Y') {
                            if (!readonly) {
                                saveFile(state.filename, &state.buffer);
                            }
                        }
                        endwin();
                        freeBuffer(&state.buffer);
                        return 0;
                    case 's':
                        if (!readonly) {
                            saveFile(state.filename, &state.buffer);
                            mvwprintw(statusWin, 0, COLS - 20, "File saved");
                        } else {
                            mvwprintw(statusWin, 0, 0, "Cannot save in read-only mode");
                        }
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
                        state.mode = 0;
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
                    case ':':
                        commandMode = 1;
                        strcpy(commandBuffer, ":");
                        mvwprintw(statusWin, 0, 0, "%s", commandBuffer);
                        wrefresh(statusWin);
                        break;
                }
            }
        }
        scrollEditor(&state);
    }
    endwin();
    freeBuffer(&state.buffer);
    return 0;
}