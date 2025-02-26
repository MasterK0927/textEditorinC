#include "editor_ops.h"
#include "buffer.h"

char clipboard[MAX_BUFFER] = {0};

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