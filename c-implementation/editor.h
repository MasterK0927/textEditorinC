#ifndef EDITOR_H
#define EDITOR_H

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

extern const char *keywords[];

#endif // EDITOR_H