#ifndef BUFFER_H
#define BUFFER_H

#include "editor.h"

void initBuffer(Buffer *buffer);
void appendToBuffer(Buffer *buffer, const char *str);
void insertIntoBuffer(Buffer *buffer, int pos, char ch);
void deleteFromBuffer(Buffer *buffer, int pos);
void freeBuffer(Buffer *buffer);

#endif // BUFFER_H