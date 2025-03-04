#include "buffer.h"

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