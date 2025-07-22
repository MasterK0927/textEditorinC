#include "file_io.h"
#include "buffer.h"

void openFile(const char *filename, Buffer *buffer) {
    FILE *file = fopen(filename, "r");
    if (file == NULL) {
        return;
    }
    // Use a fixed-size buffer here because ncurses' COLS is undefined before initscr().
    // A reasonably large line buffer avoids dependency on terminal initialization.
    char line[4096];
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