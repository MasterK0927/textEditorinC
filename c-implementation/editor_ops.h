#ifndef EDITOR_OPS_H
#define EDITOR_OPS_H

#include "editor.h"

void insertChar(EditorState *state, char ch);
void deleteChar(EditorState *state);
void moveCursor(EditorState *state, int dx, int dy);
void scrollEditor(EditorState *state);
int getScreenX(EditorState *state);
int getScreenY(EditorState *state);
void copySelection(EditorState *state, int start, int end);
void cutSelection(EditorState *state, int start, int end);
void pasteAtCursor(EditorState *state);

#endif // EDITOR_OPS_H