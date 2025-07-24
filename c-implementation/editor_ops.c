#include "editor_ops.h"
#include "buffer.h"

char clipboard[MAX_BUFFER] = {0};

void insertChar(EditorState *state, char ch) {
    int pos = state->cursor.y * COLS + state->cursor.x;
    if (pos < 0) pos = 0;
    if (pos > state->buffer.length) pos = state->buffer.length;
    insertIntoBuffer(&state->buffer, pos, ch);
    moveCursor(state, 1, 0);
}

void deleteChar(EditorState *state) {
    int pos = state->cursor.y * COLS + state->cursor.x;
    if (pos < 0) pos = 0;
    if (pos > state->buffer.length) pos = state->buffer.length;
    if (pos > 0 && state->buffer.length > 0) {
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
    int x = state->cursor.x;
    if (x < 0) x = 0;
    if (x >= COLS) x = COLS - 1;
    return x;
}

int getScreenY(EditorState *state) {
    int sy = state->cursor.y - state->scroll_offset;
    if (sy < 0) sy = 0;
    if (sy >= EDITOR_HEIGHT) sy = EDITOR_HEIGHT - 1;
    return sy;
}

void copySelection(EditorState *state, int start, int end) {
    if (start > end) { int tmp = start; start = end; end = tmp; }
    if (start < 0) start = 0;
    if (end > state->buffer.length) end = state->buffer.length;
    int length = end - start;
    if (length > 0 && length < MAX_BUFFER) {
        strncpy(clipboard, state->buffer.content + start, length);
        clipboard[length] = '\0';
    }
}

void cutSelection(EditorState *state, int start, int end) {
    if (start > end) { int tmp = start; start = end; end = tmp; }
    if (start < 0) start = 0;
    if (end > state->buffer.length) end = state->buffer.length;
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
        if (pos < 0) pos = 0;
        if (pos > state->buffer.length) pos = state->buffer.length;
        // Ensure capacity
        if (state->buffer.length + clipboardLength + 1 >= state->buffer.capacity) {
            while (state->buffer.length + clipboardLength + 1 >= state->buffer.capacity) {
                state->buffer.capacity *= 2;
            }
            state->buffer.content = realloc(state->buffer.content, state->buffer.capacity);
        }
        memmove(state->buffer.content + pos + clipboardLength, state->buffer.content + pos, state->buffer.length - pos + 1);
        memcpy(state->buffer.content + pos, clipboard, clipboardLength);
        state->buffer.length += clipboardLength;
        state->buffer.content[state->buffer.length] = '\0';
    }
}