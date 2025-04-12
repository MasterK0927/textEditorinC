CC = gcc
CFLAGS = -Wall -Wextra -pedantic -std=c99
LDFLAGS = -lncurses

SRC_DIR = .
OBJ_DIR = obj

SRCS = $(wildcard *.c)
OBJS = $(SRCS:%.c=$(OBJ_DIR)/%.o)

TARGET = your-editor

.PHONY: all clean install uninstall

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) $^ -o $@ $(LDFLAGS)

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(TARGET)

install: $(TARGET)
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m 755 $(TARGET) $(DESTDIR)$(PREFIX)/bin/$(TARGET)
	install -d $(DESTDIR)$(PREFIX)/share/man/man1
	install -m 644 man/$(TARGET).1 $(DESTDIR)$(PREFIX)/share/man/man1/$(TARGET).1

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(TARGET)
	rm -f $(DESTDIR)$(PREFIX)/share/man/man1/$(TARGET).1