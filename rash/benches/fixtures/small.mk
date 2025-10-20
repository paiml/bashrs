# Small Makefile (~50 lines, 5 targets)
# Sprint 84 Performance Benchmark - Small Fixture

.PHONY: all clean test install help

CC = gcc
CFLAGS = -Wall -Wextra -O2
PREFIX = /usr/local

SOURCES = main.c utils.c
OBJECTS = $(SOURCES:.c=.o)
TARGET = myapp

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJECTS) $(TARGET)

test: $(TARGET)
	./$(TARGET) --test

install: $(TARGET)
	mkdir -p $(PREFIX)/bin
	cp $(TARGET) $(PREFIX)/bin/

uninstall:
	rm -f $(PREFIX)/bin/$(TARGET)

help:
	@echo "Available targets:"
	@echo "  all      - Build the application"
	@echo "  clean    - Remove build artifacts"
	@echo "  test     - Run tests"
	@echo "  install  - Install to $(PREFIX)"
	@echo "  uninstall - Remove from $(PREFIX)"

.SUFFIXES:
.SUFFIXES: .c .o

.DELETE_ON_ERROR:
