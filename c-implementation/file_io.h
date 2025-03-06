#ifndef FILE_IO_H
#define FILE_IO_H

#include "editor.h"

void openFile(const char *filename, Buffer *buffer);
void saveFile(const char *filename, Buffer *buffer);

#endif // FILE_IO_H