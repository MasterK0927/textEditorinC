#include "undo.h"

static char undoStack[MAX_HISTORY][MAX_BUFFER];
static int undoTop = -1;
static char redoStack[MAX_HISTORY][MAX_BUFFER];
static int redoTop = -1;

void initUndoSystem(void) {
    undoTop = -1;
    redoTop = -1;
}

void saveUndoState(Buffer *buffer) {
    if (undoTop < MAX_HISTORY - 1) {
        size_t len = strlen(buffer->content);
        if (len >= MAX_BUFFER) len = MAX_BUFFER - 1;
        memcpy(undoStack[++undoTop], buffer->content, len);
        undoStack[undoTop][len] = '\0';
        redoTop = -1;  // Reset redo stack on new changes
    }
}

void undo(Buffer *buffer) {
    if (undoTop >= 0) {
        size_t curLen = strlen(buffer->content);
        if (curLen >= MAX_BUFFER) curLen = MAX_BUFFER - 1;
        memcpy(redoStack[++redoTop], buffer->content, curLen);
        redoStack[redoTop][curLen] = '\0';

        size_t prevLen = strlen(undoStack[undoTop]);
        if (prevLen >= (size_t)buffer->capacity) {
            while (prevLen + 1 >= (size_t)buffer->capacity) buffer->capacity *= 2;
            buffer->content = realloc(buffer->content, buffer->capacity);
        }
        memcpy(buffer->content, undoStack[undoTop--], prevLen + 1);
        buffer->length = prevLen;
    }
}

void redo(Buffer *buffer) {
    if (redoTop >= 0) {
        size_t curLen = strlen(buffer->content);
        if (curLen >= MAX_BUFFER) curLen = MAX_BUFFER - 1;
        memcpy(undoStack[++undoTop], buffer->content, curLen);
        undoStack[undoTop][curLen] = '\0';

        size_t nextLen = strlen(redoStack[redoTop]);
        if (nextLen >= (size_t)buffer->capacity) {
            while (nextLen + 1 >= (size_t)buffer->capacity) buffer->capacity *= 2;
            buffer->content = realloc(buffer->content, buffer->capacity);
        }
        memcpy(buffer->content, redoStack[redoTop--], nextLen + 1);
        buffer->length = nextLen;
    }
}