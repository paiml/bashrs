# Medium Makefile (~200 lines, 20 targets)
# Sprint 84 Performance Benchmark - Medium Fixture

.PHONY: all clean test install uninstall doc lint format check coverage help

# Compiler and flags
CC = gcc
CXX = g++
CFLAGS = -Wall -Wextra -pedantic -O2 -std=c11
CXXFLAGS = -Wall -Wextra -pedantic -O2 -std=c++17
LDFLAGS = -lpthread -lm
PREFIX = /usr/local

# Directories
SRC_DIR = src
BUILD_DIR = build
OBJ_DIR = $(BUILD_DIR)/obj
BIN_DIR = $(BUILD_DIR)/bin
DOC_DIR = docs
TEST_DIR = tests

# Sources and objects
C_SOURCES = $(wildcard $(SRC_DIR)/*.c)
CXX_SOURCES = $(wildcard $(SRC_DIR)/*.cpp)
C_OBJECTS = $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(C_SOURCES))
CXX_OBJECTS = $(patsubst $(SRC_DIR)/%.cpp,$(OBJ_DIR)/%.o,$(CXX_SOURCES))
OBJECTS = $(C_OBJECTS) $(CXX_OBJECTS)

# Test sources
TEST_SOURCES = $(wildcard $(TEST_DIR)/*.c)
TEST_OBJECTS = $(patsubst $(TEST_DIR)/%.c,$(OBJ_DIR)/test_%.o,$(TEST_SOURCES))
TEST_BINS = $(patsubst $(TEST_DIR)/%.c,$(BIN_DIR)/test_%,$(TEST_SOURCES))

# Target binary
TARGET = $(BIN_DIR)/myapp

# Version info
VERSION = $(shell git describe --tags --always 2>/dev/null || echo "unknown")
BUILD_DATE = $(shell date +%Y-%m-%d)

# All target
all: $(TARGET)

# Create directories
$(OBJ_DIR) $(BIN_DIR):
	mkdir -p $@

# Build main target
$(TARGET): $(OBJECTS) | $(BIN_DIR)
	$(CXX) $(CXXFLAGS) -o $@ $^ $(LDFLAGS)

# Compile C sources
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -DVERSION=\"$(VERSION)\" -c $< -o $@

# Compile C++ sources
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.cpp | $(OBJ_DIR)
	$(CXX) $(CXXFLAGS) -DVERSION=\"$(VERSION)\" -c $< -o $@

# Compile test sources
$(OBJ_DIR)/test_%.o: $(TEST_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -I$(SRC_DIR) -c $< -o $@

# Build test binaries
$(BIN_DIR)/test_%: $(OBJ_DIR)/test_%.o $(filter-out $(OBJ_DIR)/main.o,$(OBJECTS)) | $(BIN_DIR)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

# Run tests
test: $(TEST_BINS)
	@echo "Running tests..."
	@for test in $(TEST_BINS); do \
		echo "Running $$test"; \
		$$test || exit 1; \
	done
	@echo "All tests passed!"

# Install
install: $(TARGET)
	mkdir -p $(PREFIX)/bin
	cp $(TARGET) $(PREFIX)/bin/
	chmod 755 $(PREFIX)/bin/$(notdir $(TARGET))

# Uninstall
uninstall:
	rm -f $(PREFIX)/bin/$(notdir $(TARGET))

# Generate documentation
doc:
	@if command -v doxygen >/dev/null 2>&1; then \
		doxygen Doxyfile; \
	else \
		echo "Error: doxygen not found"; \
		exit 1; \
	fi

# Lint with cppcheck
lint:
	@if command -v cppcheck >/dev/null 2>&1; then \
		cppcheck --enable=all --suppress=missingIncludeSystem $(SRC_DIR); \
	else \
		echo "Warning: cppcheck not found, skipping lint"; \
	fi

# Format code
format:
	@if command -v clang-format >/dev/null 2>&1; then \
		find $(SRC_DIR) -name '*.c' -o -name '*.cpp' -o -name '*.h' | xargs clang-format -i; \
	else \
		echo "Warning: clang-format not found, skipping format"; \
	fi

# Static analysis
check: lint
	@echo "Static analysis complete"

# Code coverage
coverage: CFLAGS += --coverage
coverage: CXXFLAGS += --coverage
coverage: LDFLAGS += --coverage
coverage: clean test
	@if command -v gcov >/dev/null 2>&1; then \
		gcov $(SRC_DIR)/*.c $(SRC_DIR)/*.cpp; \
	else \
		echo "Error: gcov not found"; \
		exit 1; \
	fi

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	rm -f *.gcov *.gcda *.gcno

# Clean everything including docs
distclean: clean
	rm -rf $(DOC_DIR)/html $(DOC_DIR)/latex

# Help
help:
	@echo "Available targets:"
	@echo "  all        - Build the application (default)"
	@echo "  test       - Build and run tests"
	@echo "  install    - Install to $(PREFIX)"
	@echo "  uninstall  - Remove from $(PREFIX)"
	@echo "  doc        - Generate documentation"
	@echo "  lint       - Run static analysis"
	@echo "  format     - Format source code"
	@echo "  check      - Run checks (lint)"
	@echo "  coverage   - Generate code coverage report"
	@echo "  clean      - Remove build artifacts"
	@echo "  distclean  - Remove everything including docs"
	@echo ""
	@echo "Version: $(VERSION)"
	@echo "Build Date: $(BUILD_DATE)"

# Show variables (for debugging)
show-vars:
	@echo "CC         = $(CC)"
	@echo "CXX        = $(CXX)"
	@echo "CFLAGS     = $(CFLAGS)"
	@echo "CXXFLAGS   = $(CXXFLAGS)"
	@echo "LDFLAGS    = $(LDFLAGS)"
	@echo "C_SOURCES  = $(C_SOURCES)"
	@echo "CXX_SOURCES = $(CXX_SOURCES)"
	@echo "OBJECTS    = $(OBJECTS)"
	@echo "TARGET     = $(TARGET)"
	@echo "VERSION    = $(VERSION)"

.SUFFIXES:
.SUFFIXES: .c .cpp .o

.DELETE_ON_ERROR:

# Include dependencies if they exist
-include $(OBJECTS:.o=.d)
