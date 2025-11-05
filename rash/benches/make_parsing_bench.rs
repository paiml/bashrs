// Makefile Parsing Performance Benchmarks
// Sprint 73 Phase 4: Performance Benchmarking
//
// Target: <50ms for typical Makefile parsing
// Memory: <10MB for typical Makefile

use bashrs::make_parser::parser::parse_makefile;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// ============================================================================
// Test Makefiles
// ============================================================================

const SIMPLE_MAKEFILE: &str = r#"
.PHONY: all clean

all: program

program: main.o
	gcc -o program main.o

clean:
	rm -f *.o program
"#;

const MEDIUM_MAKEFILE: &str = r#"
# Build configuration
CC := gcc
CFLAGS := -Wall -O2
LDFLAGS := -lpthread

SOURCES := main.c utils.c parser.c
OBJECTS := $(SOURCES:.c=.o)
TARGET := myapp

.PHONY: all clean install test

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJECTS) $(TARGET)

install: $(TARGET)
	install -m 755 $(TARGET) /usr/local/bin/

test: $(TARGET)
	./run_tests.sh
"#;

const COMPLEX_MAKEFILE: &str = r#"
# Complex project Makefile with multiple targets

# Toolchain
CC := gcc
CXX := g++
AR := ar
RANLIB := ranlib

# Directories
SRC_DIR := src
BUILD_DIR := build
INCLUDE_DIR := include
LIB_DIR := lib
BIN_DIR := bin
TEST_DIR := tests

# Flags
CFLAGS := -Wall -Wextra -O2 -I$(INCLUDE_DIR)
CXXFLAGS := $(CFLAGS) -std=c++17
LDFLAGS := -L$(LIB_DIR) -lm -lpthread
AR_FLAGS := rcs

# Sources
C_SOURCES := $(wildcard $(SRC_DIR)/*.c)
CXX_SOURCES := $(wildcard $(SRC_DIR)/*.cpp)
HEADERS := $(wildcard $(INCLUDE_DIR)/*.h)

# Objects
C_OBJECTS := $(C_SOURCES:$(SRC_DIR)/%.c=$(BUILD_DIR)/%.o)
CXX_OBJECTS := $(CXX_SOURCES:$(SRC_DIR)/%.cpp=$(BUILD_DIR)/%.o)
ALL_OBJECTS := $(C_OBJECTS) $(CXX_OBJECTS)

# Targets
LIBRARY := $(LIB_DIR)/libmylib.a
EXECUTABLE := $(BIN_DIR)/myapp
TEST_EXECUTABLE := $(BIN_DIR)/test_runner

.PHONY: all clean distclean install uninstall test docs help

# Default target
all: directories $(LIBRARY) $(EXECUTABLE)

# Help target
help:
	@echo "Available targets:"
	@echo "  all       - Build library and executable"
	@echo "  clean     - Remove build artifacts"
	@echo "  distclean - Remove all generated files"
	@echo "  install   - Install to system"
	@echo "  uninstall - Remove from system"
	@echo "  test      - Run test suite"
	@echo "  docs      - Generate documentation"

# Create directories
directories:
	@mkdir -p $(BUILD_DIR) $(LIB_DIR) $(BIN_DIR)

# Static library
$(LIBRARY): $(ALL_OBJECTS)
	$(AR) $(AR_FLAGS) $@ $^
	$(RANLIB) $@

# Executable
$(EXECUTABLE): $(BUILD_DIR)/main.o $(LIBRARY)
	$(CXX) $(CXXFLAGS) -o $@ $< -L$(LIB_DIR) -lmylib $(LDFLAGS)

# Compile C sources
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c $(HEADERS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# Compile C++ sources
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.cpp $(HEADERS)
	@mkdir -p $(BUILD_DIR)
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Test executable
$(TEST_EXECUTABLE): $(wildcard $(TEST_DIR)/*.cpp) $(LIBRARY)
	$(CXX) $(CXXFLAGS) -o $@ $^ $(LDFLAGS)

# Test target
test: $(TEST_EXECUTABLE)
	@echo "Running tests..."
	@$(TEST_EXECUTABLE)

# Documentation
docs:
	@doxygen Doxyfile

# Installation
install: all
	@echo "Installing to /usr/local"
	@install -d /usr/local/bin
	@install -d /usr/local/lib
	@install -d /usr/local/include
	@install -m 755 $(EXECUTABLE) /usr/local/bin/
	@install -m 644 $(LIBRARY) /usr/local/lib/
	@install -m 644 $(HEADERS) /usr/local/include/

# Uninstallation
uninstall:
	@rm -f /usr/local/bin/myapp
	@rm -f /usr/local/lib/libmylib.a
	@rm -f /usr/local/include/*.h

# Clean
clean:
	@rm -rf $(BUILD_DIR)
	@rm -f $(LIBRARY) $(EXECUTABLE) $(TEST_EXECUTABLE)

# Distclean
distclean: clean
	@rm -rf $(LIB_DIR) $(BIN_DIR)
	@rm -f docs/html docs/latex

# Dependencies
-include $(ALL_OBJECTS:.o=.d)

# Generate dependency files
$(BUILD_DIR)/%.d: $(SRC_DIR)/%.c
	@$(CC) $(CFLAGS) -MM -MT '$(@:.d=.o)' $< -MF $@

$(BUILD_DIR)/%.d: $(SRC_DIR)/%.cpp
	@$(CXX) $(CXXFLAGS) -MM -MT '$(@:.d=.o)' $< -MF $@
"#;

// Real-world example from examples directory
const REALWORLD_MAKEFILE: &str = r#"
# Production Makefile for multi-service deployment

VERSION ?= $(shell date +%Y%m%d-%H%M%S)
REGISTRY := docker.io/mycompany
SERVICES := api worker scheduler frontend
DOCKER_COMPOSE := docker-compose -f docker-compose.prod.yml

.PHONY: all build test deploy clean help $(SERVICES)

all: build test

help:
	@echo "Makefile for production services"
	@echo ""
	@echo "Targets:"
	@echo "  build     - Build all Docker images"
	@echo "  test      - Run test suite"
	@echo "  deploy    - Deploy to production"
	@echo "  clean     - Remove build artifacts"

# Build all services
build: $(SERVICES)

$(SERVICES):
	@echo "Building $@..."
	@docker build -t $(REGISTRY)/$@:$(VERSION) -f services/$@/Dockerfile .
	@docker tag $(REGISTRY)/$@:$(VERSION) $(REGISTRY)/$@:latest

# Test target
test:
	@echo "Running tests..."
	@$(DOCKER_COMPOSE) run --rm test

# Deploy to production
deploy: build
	@echo "Deploying version $(VERSION)..."
	@docker push $(REGISTRY)/api:$(VERSION)
	@docker push $(REGISTRY)/worker:$(VERSION)
	@docker push $(REGISTRY)/scheduler:$(VERSION)
	@docker push $(REGISTRY)/frontend:$(VERSION)
	@kubectl set image deployment/api api=$(REGISTRY)/api:$(VERSION)
	@kubectl set image deployment/worker worker=$(REGISTRY)/worker:$(VERSION)
	@kubectl set image deployment/scheduler scheduler=$(REGISTRY)/scheduler:$(VERSION)
	@kubectl set image deployment/frontend frontend=$(REGISTRY)/frontend:$(VERSION)

# Clean
clean:
	@docker images -q $(REGISTRY)/*:* | xargs -r docker rmi -f
	@$(DOCKER_COMPOSE) down -v
"#;

// ============================================================================
// Benchmark Functions
// ============================================================================

fn benchmark_makefile_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("makefile_parsing");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.throughput(Throughput::Bytes(SIMPLE_MAKEFILE.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "simple"),
        &SIMPLE_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(makefile)),
    );

    group.throughput(Throughput::Bytes(MEDIUM_MAKEFILE.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "medium"),
        &MEDIUM_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(makefile)),
    );

    group.throughput(Throughput::Bytes(COMPLEX_MAKEFILE.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "complex"),
        &COMPLEX_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(makefile)),
    );

    group.throughput(Throughput::Bytes(REALWORLD_MAKEFILE.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "realworld"),
        &REALWORLD_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(makefile)),
    );

    group.finish();
}

fn benchmark_makefile_parsing_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("makefile_parsing_by_size");
    group.measurement_time(Duration::from_secs(10));

    let sizes = [("10_lines", generate_makefile(10)),
        ("50_lines", generate_makefile(50)),
        ("100_lines", generate_makefile(100)),
        ("200_lines", generate_makefile(200))];

    for (name, makefile) in sizes.iter() {
        group.throughput(Throughput::Bytes(makefile.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), makefile, |b, mf| {
            b.iter(|| parse_makefile(mf))
        });
    }

    group.finish();
}

fn benchmark_parsing_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_features");
    group.measurement_time(Duration::from_secs(10));

    // Variables
    let makefile_with_vars = r#"
VAR1 := value1
VAR2 := value2
VAR3 := $(VAR1)_$(VAR2)

all:
	echo $(VAR3)
"#;

    group.bench_with_input(
        BenchmarkId::new("parse", "variables"),
        &makefile_with_vars,
        |b, mf| b.iter(|| parse_makefile(mf)),
    );

    // Pattern rules
    let makefile_with_patterns = r#"
%.o: %.c
	gcc -c $< -o $@

%.o: %.cpp
	g++ -c $< -o $@

all: file1.o file2.o
"#;

    group.bench_with_input(
        BenchmarkId::new("parse", "patterns"),
        &makefile_with_patterns,
        |b, mf| b.iter(|| parse_makefile(mf)),
    );

    // Conditionals
    let makefile_with_conditionals = r#"
ifeq ($(OS),linux)
    PLATFORM := linux
else ifeq ($(OS),darwin)
    PLATFORM := macos
else
    PLATFORM := unknown
endif

all:
	echo $(PLATFORM)
"#;

    group.bench_with_input(
        BenchmarkId::new("parse", "conditionals"),
        &makefile_with_conditionals,
        |b, mf| b.iter(|| parse_makefile(mf)),
    );

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("makefile_memory");
    group.sample_size(50);

    group.bench_function("ast_size_simple", |b| {
        b.iter(|| {
            let ast = parse_makefile(SIMPLE_MAKEFILE).unwrap();
            std::mem::size_of_val(&ast)
        })
    });

    group.bench_function("ast_size_complex", |b| {
        b.iter(|| {
            let ast = parse_makefile(COMPLEX_MAKEFILE).unwrap();
            std::mem::size_of_val(&ast)
        })
    });

    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_makefile(num_targets: usize) -> String {
    let mut makefile = String::new();

    makefile.push_str("# Generated Makefile for benchmarking\n\n");
    makefile.push_str(".PHONY: all clean\n\n");

    // Variables
    makefile.push_str("CC := gcc\n");
    makefile.push_str("CFLAGS := -Wall -O2\n\n");

    // All target
    makefile.push_str("all:");
    for i in 0..num_targets {
        makefile.push_str(&format!(" target{}", i));
    }
    makefile.push_str("\n\n");

    // Individual targets
    for i in 0..num_targets {
        makefile.push_str(&format!("target{}: file{}.o\n\t$(CC) -o $@ $<\n\n", i, i));
        makefile.push_str(&format!(
            "file{}.o: file{}.c\n\t$(CC) $(CFLAGS) -c $< -o $@\n\n",
            i, i
        ));
    }

    // Clean target
    makefile.push_str("clean:\n\trm -f *.o");
    for i in 0..num_targets {
        makefile.push_str(&format!(" target{}", i));
    }
    makefile.push('\n');

    makefile
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    benchmark_makefile_parsing,
    benchmark_makefile_parsing_by_size,
    benchmark_parsing_features,
    benchmark_memory_usage
);

criterion_main!(benches);
