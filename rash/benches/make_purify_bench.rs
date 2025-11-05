// Makefile Purification Performance Benchmarks
// Sprint 73 Phase 4: Performance Benchmarking
//
// Target: <100ms for typical Makefile purification
// Memory: <10MB for purification process

use bashrs::make_parser::parser::parse_makefile;
use bashrs::make_parser::purify::purify_makefile;
use bashrs::make_parser::semantic::analyze_makefile;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// ============================================================================
// Test Makefiles with Non-Determinism
// ============================================================================

const MAKEFILE_WITH_TIMESTAMP: &str = r#"
# Makefile with timestamp-based versions
VERSION := $(shell date +%Y%m%d-%H%M%S)
BUILD_ID := build-$(VERSION)

.PHONY: all clean

all: release

release:
	mkdir /tmp/$(BUILD_ID)
	echo "Building version $(VERSION)"
	tar -czf release-$(VERSION).tar.gz src/

clean:
	rm -rf /tmp/build-*
"#;

const MAKEFILE_WITH_RANDOM: &str = r#"
# Makefile with random session IDs
SESSION_ID := $(shell echo $$RANDOM)
TMP_DIR := /tmp/build-$(SESSION_ID)

.PHONY: build test

build:
	mkdir $(TMP_DIR)
	cd $(TMP_DIR) && make -f ../Makefile.real

test:
	./run_tests.sh $(SESSION_ID)
"#;

const MAKEFILE_WITH_PROCESS_ID: &str = r#"
# Makefile using process ID
LOCK_FILE := /tmp/build.lock.$$
PID_FILE := /var/run/build.pid.$$

.PHONY: lock unlock build

lock:
	echo $$ > $(LOCK_FILE)

unlock:
	rm $(LOCK_FILE)

build: lock
	@echo "Building with PID $$"
	make compile
	$(MAKE) unlock
"#;

const COMPLEX_MAKEFILE_TO_PURIFY: &str = r#"
# Complex Makefile with multiple non-deterministic elements

# Version from timestamp
VERSION := $(shell date +%s)
BUILD_TIME := $(shell date '+%Y-%m-%d %H:%M:%S')

# Random session
SESSION := session-$$RANDOM

# Directories with timestamps
BUILD_DIR := build/$(VERSION)
DIST_DIR := dist-$(shell date +%Y%m%d)

# Non-idempotent operations
.PHONY: all clean deploy backup

all: build package

build:
	mkdir $(BUILD_DIR)
	echo "Build started at $(BUILD_TIME)" > $(BUILD_DIR)/info.txt
	gcc -DVERSION=$(VERSION) -o $(BUILD_DIR)/app src/*.c

package:
	mkdir $(DIST_DIR)
	tar -czf $(DIST_DIR)/app-$(VERSION).tar.gz $(BUILD_DIR)

deploy: package
	scp $(DIST_DIR)/*.tar.gz server:/releases/$(VERSION)/
	ssh server "ln -s /releases/$(VERSION) /releases/current"
	echo "Deployed at $(BUILD_TIME)" > deploy.log

backup:
	mkdir /backup/$(VERSION)
	cp -r $(BUILD_DIR) /backup/$(VERSION)/

clean:
	rm -rf build/*
	rm -rf dist-*
	rm deploy.log
"#;

// Production-like Makefile
const PRODUCTION_MAKEFILE: &str = r#"
# Production deployment Makefile

TIMESTAMP := $(shell date +%Y%m%d%H%M%S)
VERSION ?= $(TIMESTAMP)
RELEASE := release-$(VERSION)
SESSION := deploy-$$

SERVICES := api worker scheduler
DOCKER_REGISTRY := myregistry.com

.PHONY: all build tag push deploy rollback clean

all: build tag push

build:
	@for service in $(SERVICES); do \
		docker build -t $$service:$(VERSION) services/$$service/ ; \
	done

tag:
	@for service in $(SERVICES); do \
		docker tag $$service:$(VERSION) $(DOCKER_REGISTRY)/$$service:$(VERSION) ; \
		docker tag $$service:$(VERSION) $(DOCKER_REGISTRY)/$$service:latest ; \
	done

push:
	@for service in $(SERVICES); do \
		docker push $(DOCKER_REGISTRY)/$$service:$(VERSION) ; \
		docker push $(DOCKER_REGISTRY)/$$service:latest ; \
	done

deploy:
	@echo "Deploying $(RELEASE) at $(TIMESTAMP)"
	@mkdir -p /releases/$(RELEASE)
	@kubectl apply -f k8s/
	@kubectl set image deployment/api api=$(DOCKER_REGISTRY)/api:$(VERSION)
	@kubectl set image deployment/worker worker=$(DOCKER_REGISTRY)/worker:$(VERSION)
	@ln -s /releases/$(RELEASE) /releases/current
	@echo "$(TIMESTAMP): Deployed $(VERSION)" >> /var/log/deploys.log

rollback:
	@kubectl rollout undo deployment/api
	@kubectl rollout undo deployment/worker

clean:
	@rm -rf /releases/release-*
	@docker system prune -af
"#;

// ============================================================================
// Benchmark Functions
// ============================================================================

fn benchmark_purify_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify_simple");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.throughput(Throughput::Bytes(MAKEFILE_WITH_TIMESTAMP.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("purify", "timestamp"),
        &MAKEFILE_WITH_TIMESTAMP,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(makefile).unwrap();
                let _issues = analyze_makefile(&ast);
                purify_makefile(&ast)
            })
        },
    );

    group.throughput(Throughput::Bytes(MAKEFILE_WITH_RANDOM.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("purify", "random"),
        &MAKEFILE_WITH_RANDOM,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(makefile).unwrap();
                let _issues = analyze_makefile(&ast);
                purify_makefile(&ast)
            })
        },
    );

    group.throughput(Throughput::Bytes(MAKEFILE_WITH_PROCESS_ID.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("purify", "process_id"),
        &MAKEFILE_WITH_PROCESS_ID,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(makefile).unwrap();
                let _issues = analyze_makefile(&ast);
                purify_makefile(&ast)
            })
        },
    );

    group.finish();
}

fn benchmark_purify_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify_complex");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    group.throughput(Throughput::Bytes(COMPLEX_MAKEFILE_TO_PURIFY.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("purify", "complex"),
        &COMPLEX_MAKEFILE_TO_PURIFY,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(makefile).unwrap();
                let _issues = analyze_makefile(&ast);
                purify_makefile(&ast)
            })
        },
    );

    group.throughput(Throughput::Bytes(PRODUCTION_MAKEFILE.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("purify", "production"),
        &PRODUCTION_MAKEFILE,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(makefile).unwrap();
                let _issues = analyze_makefile(&ast);
                purify_makefile(&ast)
            })
        },
    );

    group.finish();
}

fn benchmark_purify_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify_end_to_end");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    // Full pipeline: parse → analyze → purify
    group.bench_with_input(
        BenchmarkId::new("full_pipeline", "complex"),
        &COMPLEX_MAKEFILE_TO_PURIFY,
        |b, makefile| {
            b.iter(|| {
                // Step 1: Parse
                let ast = parse_makefile(makefile).unwrap();

                // Step 2: Analyze
                let issues = analyze_makefile(&ast);

                // Step 3: Purify
                let purified = purify_makefile(&ast);

                (issues.len(), purified.report.len())
            })
        },
    );

    group.finish();
}

fn benchmark_analysis_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_analysis");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Pre-parse the Makefiles
    let timestamp_ast = parse_makefile(MAKEFILE_WITH_TIMESTAMP).unwrap();
    let complex_ast = parse_makefile(COMPLEX_MAKEFILE_TO_PURIFY).unwrap();
    let production_ast = parse_makefile(PRODUCTION_MAKEFILE).unwrap();

    group.bench_with_input(
        BenchmarkId::new("analyze", "timestamp"),
        &timestamp_ast,
        |b, ast| b.iter(|| analyze_makefile(ast)),
    );

    group.bench_with_input(
        BenchmarkId::new("analyze", "complex"),
        &complex_ast,
        |b, ast| b.iter(|| analyze_makefile(ast)),
    );

    group.bench_with_input(
        BenchmarkId::new("analyze", "production"),
        &production_ast,
        |b, ast| b.iter(|| analyze_makefile(ast)),
    );

    group.finish();
}

fn benchmark_purify_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify_scalability");
    group.measurement_time(Duration::from_secs(15));

    // Test with different numbers of non-deterministic elements
    for num_issues in [5, 10, 20, 50].iter() {
        let makefile = generate_makefile_with_issues(*num_issues);

        group.throughput(Throughput::Elements(*num_issues as u64));
        group.bench_with_input(
            BenchmarkId::new("purify_by_issues", num_issues),
            &makefile,
            |b, mf| {
                b.iter(|| {
                    let ast = parse_makefile(mf).unwrap();
                    let _issues = analyze_makefile(&ast);
                    purify_makefile(&ast)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify_memory");
    group.sample_size(30);

    group.bench_function("memory_complex_ast", |b| {
        b.iter(|| {
            let ast = parse_makefile(COMPLEX_MAKEFILE_TO_PURIFY).unwrap();
            std::mem::size_of_val(&ast)
        })
    });

    group.bench_function("memory_issues", |b| {
        b.iter(|| {
            let ast = parse_makefile(COMPLEX_MAKEFILE_TO_PURIFY).unwrap();
            let issues = analyze_makefile(&ast);
            std::mem::size_of_val(&issues)
        })
    });

    group.bench_function("memory_purified", |b| {
        b.iter(|| {
            let ast = parse_makefile(COMPLEX_MAKEFILE_TO_PURIFY).unwrap();
            let purified = purify_makefile(&ast);
            purified.report.len()
        })
    });

    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_makefile_with_issues(num_issues: usize) -> String {
    let mut makefile = String::new();

    makefile.push_str("# Generated Makefile with non-deterministic elements\n\n");

    // Add timestamp-based variables
    for i in 0..num_issues / 3 {
        makefile.push_str(&format!("VERSION_{} := $(shell date +%s)\n", i));
    }

    makefile.push_str("\n.PHONY: all clean\n\n");

    // Add targets with non-deterministic operations
    makefile.push_str("all:");
    for i in 0..num_issues {
        makefile.push_str(&format!(" target{}", i));
    }
    makefile.push_str("\n\n");

    for i in 0..num_issues {
        let operation = match i % 3 {
            0 => format!("mkdir /tmp/build-$(shell date +%s)-{}", i),
            1 => format!("echo $$ > /tmp/pid-{}.txt", i),
            _ => format!("tar -czf release-$(shell date +%s).tar.gz src/"),
        };

        makefile.push_str(&format!("target{}:\n\t{}\n\n", i, operation));
    }

    makefile.push_str("clean:\n\trm -rf /tmp/build-* /tmp/pid-*\n");

    makefile
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    benchmark_purify_simple,
    benchmark_purify_complex,
    benchmark_purify_end_to_end,
    benchmark_analysis_only,
    benchmark_purify_scalability,
    benchmark_memory_usage
);

criterion_main!(benches);
