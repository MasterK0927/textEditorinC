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