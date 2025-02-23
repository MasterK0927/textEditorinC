#include "file_io.h"
#include "buffer.h"

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