#ifndef DISPLAY_H
#define DISPLAY_H

#include "editor.h"

void syntaxHighlight(WINDOW *editorWin, const char *line, int length, int cursorX);
void printStatusBar(WINDOW *statusWin, EditorState *state);
void showHelp(WINDOW *editorWin, EditorState *state);
void listCommands(WINDOW *editorWin);
void displayHelpForCommand(WINDOW *editorWin, const char *command);
int displayCodeSnippet(WINDOW *editorWin, int y, const char *code);

#endif // DISPLAY_H