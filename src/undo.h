#ifndef UNDO_H
#define UNDO_H

#include "editor.h"

void saveUndoState(Buffer *buffer);
void undo(Buffer *buffer);
void redo(Buffer *buffer);
void initUndoSystem(void);

#endif // UNDO_H