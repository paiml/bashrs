// Node.js project bootstrap installer
// Demonstrates setting up a complete Node.js development environment

#[rash::main]
fn nodejs_project_bootstrap() {
    let project_name = env_var_or("PROJECT_NAME", "my-nodejs-app");
    let node_version = env_var_or("NODE_VERSION", "18.17.1");
    let package_manager = env_var_or("PACKAGE_MANAGER", "npm"); // npm, yarn, or pnpm
    let framework = env_var_or("FRAMEWORK", "express"); // express, fastify, nest, next
    let database = env_var_or("DATABASE", "none"); // mongodb, postgresql, mysql, sqlite, none
    let testing_framework = env_var_or("TESTING", "jest"); // jest, mocha, vitest
    
    echo("📦 Node.js Project Bootstrap Installer");
    echo(&format!("Project: {}, Node: {}, Package Manager: {}", project_name, node_version, package_manager));
    
    // Detect operating system
    let os_info = detect_operating_system();
    
    // Install Node.js via Node Version Manager (nvm)
    install_nodejs_via_nvm(&node_version, &os_info);
    
    // Install package manager
    install_package_manager(&package_manager);
    
    // Create project structure
    create_project_structure(&project_name);
    
    // Initialize package.json with proper configuration
    initialize_package_json(&project_name, &package_manager, &framework, &testing_framework);
    
    // Install framework and dependencies
    install_framework_dependencies(&framework, &package_manager);
    
    // Setup database integration
    if database != "none" {
        setup_database_integration(&database, &package_manager);
    }
    
    // Setup testing framework
    setup_testing_framework(&testing_framework, &package_manager);
    
    // Setup development tools
    setup_development_tools(&package_manager);
    
    // Setup CI/CD configuration
    setup_cicd_configuration(&testing_framework);
    
    // Setup Docker configuration
    setup_docker_configuration(&framework);
    
    // Create sample application code
    create_sample_application(&framework, &database);
    
    // Setup environment configuration
    setup_environment_configuration();
    
    // Install dependencies
    install_dependencies(&package_manager);
    
    // Create documentation
    create_project_documentation(&project_name, &framework, &database);
    
    echo("✅ Node.js project bootstrap completed successfully");
    echo(&format!("cd {} && {} run dev", project_name, package_manager));
}

fn detect_operating_system() -> String {
    if path_exists("/etc/os-release") {
        let os_release = read_file("/etc/os-release");
        if os_release.contains("Ubuntu") {
            return "linux".to_string();
        } else if os_release.contains("Debian") {
            return "linux".to_string();
        } else if os_release.contains("CentOS") || os_release.contains("Red Hat") {
            return "linux".to_string();
        }
    }
    
    if command_exists("sw_vers") {
        return "macos".to_string();
    }
    
    if command_exists("cmd") {
        return "windows".to_string();
    }
    
    "linux".to_string()
}

fn install_nodejs_via_nvm(version: &str, os: &str) {
    echo(&format!("🔧 Installing Node.js {} via NVM...", version));
    
    // Install nvm
    match os {
        "linux" | "macos" => {
            // Download and install nvm
            exec("curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash");
            
            // Source nvm in current shell
            let nvm_dir = env_var_or("NVM_DIR", &format!("{}/.nvm", env_var("HOME")));
            exec(&format!("export NVM_DIR={} && [ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"", nvm_dir));
            
            // Install and use specific Node.js version
            exec(&format!("source ~/.bashrc && nvm install {} && nvm use {}", version, version));
            exec(&format!("source ~/.bashrc && nvm alias default {}", version));
        },
        "windows" => {
            // For Windows, we'll download Node.js directly
            let node_url = format!("https://nodejs.org/dist/v{}/node-v{}-win-x64.zip", version, version);
            echo("Downloading Node.js for Windows...");
            exec(&format!("curl -L {} -o node.zip", node_url));
            exec("unzip node.zip");
            exec(&format!("mv node-v{}-win-x64 /c/nodejs", version));
            exec("export PATH=/c/nodejs:$PATH");
        },
        _ => {
            panic!("Unsupported operating system for Node.js installation");
        }
    }
    
    // Verify installation
    exec("node --version");
    exec("npm --version");
    
    echo("✅ Node.js installed successfully");
}

fn install_package_manager(manager: &str) {
    echo(&format!("📦 Setting up package manager: {}", manager));
    
    match manager {
        "npm" => {
            // npm comes with Node.js, just update it
            exec("npm install -g npm@latest");
        },
        "yarn" => {
            exec("npm install -g yarn");
            exec("yarn --version");
        },
        "pnpm" => {
            exec("npm install -g pnpm");
            exec("pnpm --version");
        },
        _ => panic!("Unsupported package manager: {}", manager),
    }
    
    echo(&format!("✅ {} installed successfully", manager));
}

fn create_project_structure(project_name: &str) {
    echo(&format!("📁 Creating project structure for {}...", project_name));
    
    // Create main project directory
    mkdir_p(project_name);
    cd(project_name);
    
    // Create standard Node.js project structure
    let directories = [
        "src",
        "src/controllers",
        "src/models",
        "src/routes",
        "src/middleware",
        "src/services",
        "src/utils",
        "tests",
        "tests/unit",
        "tests/integration",
        "tests/e2e",
        "docs",
        "scripts",
        "config",
        "public",
        "public/css",
        "public/js",
        "public/images",
        "logs",
    ];
    
    for dir in &directories {
        mkdir_p(dir);
    }
    
    // Create essential files
    touch(".env");
    touch(".env.example");
    touch(".gitignore");
    touch("README.md");
    touch("CHANGELOG.md");
    touch("LICENSE");
    
    echo("✅ Project structure created");
}

fn initialize_package_json(name: &str, manager: &str, framework: &str, testing: &str) {
    echo("📋 Initializing package.json...");
    
    let package_json = format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "A {} application built with Node.js",
  "main": "src/index.js",
  "scripts": {{
    "start": "node src/index.js",
    "dev": "nodemon src/index.js",
    "build": "babel src -d dist",
    "test": "{} test",
    "test:watch": "{} test --watch",
    "test:coverage": "{} test --coverage",
    "lint": "eslint src/ tests/",
    "lint:fix": "eslint src/ tests/ --fix",
    "format": "prettier --write src/ tests/",
    "prepare": "husky install",
    "docker:build": "docker build -t {} .",
    "docker:run": "docker run -p 3000:3000 {}",
    "migrate": "node scripts/migrate.js",
    "seed": "node scripts/seed.js"
  }},
  "keywords": [
    "nodejs",
    "{}",
    "api",
    "backend"
  ],
  "author": "Your Name <your.email@example.com>",
  "license": "MIT",
  "engines": {{
    "node": ">=18.0.0",
    "npm": ">=8.0.0"
  }},
  "dependencies": {{}},
  "devDependencies": {{}},
  "lint-staged": {{
    "*.{{js,ts}}": [
      "eslint --fix",
      "prettier --write"
    ]
  }},
  "husky": {{
    "hooks": {{
      "pre-commit": "lint-staged",
      "pre-push": "npm test"
    }}
  }}
}}"#, name, framework, testing, testing, testing, name, name, framework);
    
    write_file("package.json", &package_json);
    
    echo("✅ package.json initialized");
}

fn install_framework_dependencies(framework: &str, manager: &str) {
    echo(&format!("🚀 Installing {} framework dependencies...", framework));
    
    let (dependencies, dev_dependencies) = match framework {
        "express" => {
            let deps = vec![
                "express",
                "helmet",
                "cors",
                "morgan",
                "compression",
                "express-rate-limit",
                "express-validator",
                "bcryptjs",
                "jsonwebtoken",
                "dotenv",
                "winston",
            ];
            let dev_deps = vec![
                "nodemon",
                "supertest",
            ];
            (deps, dev_deps)
        },
        "fastify" => {
            let deps = vec![
                "fastify",
                "@fastify/helmet",
                "@fastify/cors",
                "@fastify/rate-limit",
                "@fastify/compress",
                "@fastify/env",
                "@fastify/jwt",
                "bcryptjs",
                "pino",
                "pino-pretty",
            ];
            let dev_deps = vec![
                "nodemon",
            ];
            (deps, dev_deps)
        },
        "nest" => {
            let deps = vec![
                "@nestjs/core",
                "@nestjs/common",
                "@nestjs/platform-express",
                "@nestjs/config",
                "@nestjs/jwt",
                "@nestjs/passport",
                "passport",
                "passport-local",
                "passport-jwt",
                "bcryptjs",
                "class-validator",
                "class-transformer",
                "rxjs",
                "reflect-metadata",
            ];
            let dev_deps = vec![
                "@nestjs/cli",
                "@nestjs/testing",
                "nodemon",
                "ts-node",
                "typescript",
                "@types/node",
                "@types/express",
                "@types/bcryptjs",
                "@types/passport-local",
                "@types/passport-jwt",
            ];
            (deps, dev_deps)
        },
        "next" => {
            let deps = vec![
                "next",
                "react",
                "react-dom",
                "@next/font",
            ];
            let dev_deps = vec![
                "@types/node",
                "@types/react",
                "@types/react-dom",
                "typescript",
                "eslint",
                "eslint-config-next",
            ];
            (deps, dev_deps)
        },
        _ => panic!("Unsupported framework: {}", framework),
    };
    
    // Install dependencies
    let deps_str = dependencies.join(" ");
    exec(&format!("{} install {}", manager, deps_str));
    
    // Install dev dependencies
    let dev_deps_str = dev_dependencies.join(" ");
    let dev_flag = match manager {
        "npm" => "--save-dev",
        "yarn" => "--dev",
        "pnpm" => "--save-dev",
        _ => "--save-dev",
    };
    exec(&format!("{} install {} {}", manager, dev_flag, dev_deps_str));
    
    echo(&format!("✅ {} dependencies installed", framework));
}

fn setup_database_integration(database: &str, manager: &str) {
    echo(&format!("🗄️ Setting up {} database integration...", database));
    
    let (deps, dev_deps) = get_database_dependencies(database);
    
    install_database_deps(&deps, &dev_deps, manager);
    
    // Create database configuration
    create_database_config(database);
    
    echo(&format!("✅ {} integration configured", database));
}

fn get_database_dependencies(database: &str) -> (Vec<&str>, Vec<&str>) {
    match database {
        "mongodb" => (vec!["mongoose", "mongodb-memory-server"], vec![]),
        "postgresql" => (vec!["pg", "pg-pool"], vec!["@types/pg"]),
        "mysql" => (vec!["mysql2"], vec![]),
        "sqlite" => (vec!["sqlite3"], vec![]),
        _ => panic!("Unsupported database: {}", database),
    }
}

fn install_database_deps(deps: &[&str], dev_deps: &[&str], manager: &str) {
    if !deps.is_empty() {
        let deps_str = deps.join(" ");
        exec(&format!("{} install {}", manager, deps_str));
    }
    
    if !dev_deps.is_empty() {
        let dev_deps_str = dev_deps.join(" ");
        let dev_flag = get_dev_flag(manager);
        exec(&format!("{} install {} {}", manager, dev_flag, dev_deps_str));
    }
}

fn get_dev_flag(manager: &str) -> &str {
    match manager {
        "yarn" => "--dev",
        _ => "--save-dev",
    }
}

fn create_database_config(database: &str) {
    match database {
        "mongodb" => create_mongodb_config(),
        "postgresql" => create_postgresql_config(),
        "mysql" => create_mysql_config(),
        "sqlite" => create_sqlite_config(),
        _ => {},
    }
}

fn create_mongodb_config() {
    let config = get_mongodb_config_template();
    write_file("src/config/database.js", &config);
    append_to_file(".env.example", "MONGODB_URI=mongodb://localhost:27017/myapp\n");
}

fn create_postgresql_config() {
    let config = get_postgresql_config_template();
    write_file("src/config/database.js", &config);
    append_to_file(".env.example", "DATABASE_URL=postgresql://localhost:5432/myapp\n");
}

fn create_mysql_config() {
    let config = get_mysql_config_template();
    write_file("src/config/database.js", &config);
    append_to_file(".env.example", "DB_HOST=localhost\nDB_USER=root\nDB_PASSWORD=\nDB_NAME=myapp\n");
}

fn create_sqlite_config() {
    let config = get_sqlite_config_template();
    write_file("src/config/database.js", &config);
    mkdir_p("data");
    append_to_file(".env.example", "DATABASE_PATH=./data/database.sqlite\n");
}

fn get_mongodb_config_template() -> &'static str {
    r#"const mongoose = require('mongoose');

const connectDB = async () => {
  try {
    const conn = await mongoose.connect(process.env.MONGODB_URI || 'mongodb://localhost:27017/myapp', {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    });
    
    console.log(`MongoDB Connected: ${conn.connection.host}`);
    
    // Handle connection events
    mongoose.connection.on('error', (err) => {
      console.error('MongoDB connection error:', err);
    });
    
    mongoose.connection.on('disconnected', () => {
      console.log('MongoDB disconnected');
    });
    
    // Graceful shutdown
    process.on('SIGINT', async () => {
      await mongoose.connection.close();
      console.log('MongoDB connection closed.');
      process.exit(0);
    });
    
  } catch (error) {
    console.error('Error connecting to MongoDB:', error);
    process.exit(1);
  }
};

module.exports = connectDB;"#
}

fn get_postgresql_config_template() -> &'static str {
    r#"const { Pool } = require('pg');

const pool = new Pool({
  connectionString: process.env.DATABASE_URL || 'postgresql://localhost:5432/myapp',
  ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false,
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
});

// Handle pool events
pool.on('connect', () => {
  console.log('Connected to PostgreSQL database');
});

pool.on('error', (err) => {
  console.error('Unexpected error on idle client', err);
  process.exit(-1);
});

// Graceful shutdown
process.on('SIGINT', async () => {
  await pool.end();
  console.log('PostgreSQL pool has ended');
  process.exit(0);
});

module.exports = pool;"#
}

fn get_mysql_config_template() -> &'static str {
    r#"const mysql = require('mysql2/promise');

const createConnection = async () => {
  const connection = await mysql.createConnection({
    host: process.env.DB_HOST || 'localhost',
    user: process.env.DB_USER || 'root',
    password: process.env.DB_PASSWORD || '',
    database: process.env.DB_NAME || 'myapp',
    waitForConnections: true,
    connectionLimit: 10,
    queueLimit: 0,
  });
  
  console.log('Connected to MySQL database');
  return connection;
};

module.exports = { createConnection };"#
}

fn get_sqlite_config_template() -> &'static str {
    r#"const sqlite3 = require('sqlite3').verbose();
const path = require('path');

const dbPath = process.env.DATABASE_PATH || path.join(__dirname, '../../data/database.sqlite');

const db = new sqlite3.Database(dbPath, (err) => {
  if (err) {
    console.error('Error opening database:', err);
  } else {
    console.log('Connected to SQLite database');
  }
});

// Graceful shutdown
process.on('SIGINT', () => {
  db.close((err) => {
    if (err) {
      console.error('Error closing database:', err);
    } else {
      console.log('Database connection closed.');
    }
    process.exit(0);
  });
});

module.exports = db;"#
}

fn setup_testing_framework(testing: &str, manager: &str) {
    echo(&format!("🧪 Setting up {} testing framework...", testing));
    
    let (deps, config) = match testing {
        "jest" => {
            let deps = vec!["jest", "supertest", "@types/jest"];
            let config = r#"{
  "testEnvironment": "node",
  "testMatch": [
    "**/tests/**/*.test.js",
    "**/tests/**/*.spec.js"
  ],
  "collectCoverageFrom": [
    "src/**/*.js",
    "!src/index.js"
  ],
  "coverageDirectory": "coverage",
  "coverageReporters": ["text", "lcov", "html"],
  "setupFilesAfterEnv": ["<rootDir>/tests/setup.js"]
}"#;
            (deps, config)
        },
        "mocha" => {
            let deps = vec!["mocha", "chai", "supertest", "@types/mocha", "@types/chai"];
            let config = r#"{
  "require": ["tests/setup.js"],
  "recursive": true,
  "timeout": 5000,
  "spec": "tests/**/*.test.js",
  "reporter": "spec"
}"#;
            (deps, config)
        },
        "vitest" => {
            let deps = vec!["vitest", "supertest"];
            let config = r#"{
  "test": {
    "globals": true,
    "environment": "node",
    "coverage": {
      "provider": "c8",
      "reporter": ["text", "lcov", "html"]
    }
  }
}"#;
            (deps, config)
        },
        _ => panic!("Unsupported testing framework: {}", testing),
    };
    
    // Install testing dependencies
    let deps_str = deps.join(" ");
    let dev_flag = match manager {
        "npm" => "--save-dev",
        "yarn" => "--dev",
        "pnpm" => "--save-dev", 
        _ => "--save-dev",
    };
    exec(&format!("{} install {} {}", manager, dev_flag, deps_str));
    
    // Create test configuration
    match testing {
        "jest" => write_file("jest.config.json", config),
        "mocha" => write_file(".mocharc.json", config),
        "vitest" => {
            // vitest config goes in vite.config.js or package.json
            let package_json = read_file("package.json");
            let updated = package_json.replace("\"devDependencies\": {}", &format!("\"vitest\": {}, \"devDependencies\": {{}}", config));
            write_file("package.json", &updated);
        },
        _ => {},
    }
    
    // Create test setup file
    let test_setup = r#"// Test setup and global configurations
process.env.NODE_ENV = 'test';

// Global test utilities
global.expect = require('chai').expect; // For mocha/chai
"#;
    
    write_file("tests/setup.js", test_setup);
    
    // Create sample test file
    create_sample_test(testing);
    
    echo(&format!("✅ {} testing framework configured", testing));
}

fn create_sample_test(testing: &str) {
    let test_content = match testing {
        "jest" => r#"const request = require('supertest');
const app = require('../src/index');

describe('API Tests', () => {
  test('GET / should return 200', async () => {
    const response = await request(app).get('/');
    expect(response.status).toBe(200);
  });

  test('GET /health should return health status', async () => {
    const response = await request(app).get('/health');
    expect(response.status).toBe(200);
    expect(response.body).toHaveProperty('status', 'OK');
  });
});
"#,
        "mocha" => r#"const request = require('supertest');
const { expect } = require('chai');
const app = require('../src/index');

describe('API Tests', () => {
  it('GET / should return 200', async () => {
    const response = await request(app).get('/');
    expect(response.status).to.equal(200);
  });

  it('GET /health should return health status', async () => {
    const response = await request(app).get('/health');
    expect(response.status).to.equal(200);
    expect(response.body).to.have.property('status', 'OK');
  });
});
"#,
        "vitest" => r#"import { describe, it, expect } from 'vitest';
import request from 'supertest';
import app from '../src/index.js';

describe('API Tests', () => {
  it('GET / should return 200', async () => {
    const response = await request(app).get('/');
    expect(response.status).toBe(200);
  });

  it('GET /health should return health status', async () => {
    const response = await request(app).get('/health');
    expect(response.status).toBe(200);
    expect(response.body).toHaveProperty('status', 'OK');
  });
});
"#,
        _ => "",
    };
    
    write_file("tests/unit/api.test.js", test_content);
}

fn setup_development_tools(manager: &str) {
    echo("🛠️ Setting up development tools...");
    
    let dev_tools = vec![
        "eslint",
        "prettier",
        "husky",
        "lint-staged",
        "@babel/core",
        "@babel/preset-env",
        "@babel/cli",
    ];
    
    let dev_tools_str = dev_tools.join(" ");
    let dev_flag = match manager {
        "npm" => "--save-dev",
        "yarn" => "--dev",
        "pnpm" => "--save-dev",
        _ => "--save-dev",
    };
    exec(&format!("{} install {} {}", manager, dev_flag, dev_tools_str));
    
    // Create ESLint configuration
    let eslint_config = r#"{
  "env": {
    "node": true,
    "es2021": true,
    "jest": true
  },
  "extends": [
    "eslint:recommended"
  ],
  "parserOptions": {
    "ecmaVersion": 12,
    "sourceType": "module"
  },
  "rules": {
    "no-console": "warn",
    "no-unused-vars": "error",
    "semi": ["error", "always"],
    "quotes": ["error", "single"]
  }
}"#;
    
    write_file(".eslintrc.json", eslint_config);
    
    // Create Prettier configuration
    let prettier_config = r#"{
  "semi": true,
  "trailingComma": "es5",
  "singleQuote": true,
  "printWidth": 100,
  "tabWidth": 2,
  "useTabs": false
}"#;
    
    write_file(".prettierrc", prettier_config);
    
    // Create .prettierignore
    let prettier_ignore = "node_modules/\ndist/\ncoverage/\n*.log\n";
    write_file(".prettierignore", prettier_ignore);
    
    // Create Babel configuration
    let babel_config = r#"{
  "presets": [
    [
      "@babel/preset-env",
      {
        "targets": {
          "node": "18"
        }
      }
    ]
  ]
}"#;
    
    write_file(".babelrc", babel_config);
    
    echo("✅ Development tools configured");
}

fn setup_cicd_configuration(testing: &str) {
    echo("🔄 Setting up CI/CD configuration...");
    
    // GitHub Actions workflow
    mkdir_p(".github/workflows");
    
    let github_workflow = format!(r#"name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        node-version: [16.x, 18.x, 20.x]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Use Node.js ${{{{ matrix.node-version }}}}
      uses: actions/setup-node@v3
      with:
        node-version: ${{{{ matrix.node-version }}}}
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run linter
      run: npm run lint
    
    - name: Run tests
      run: npm run test:coverage
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage/lcov.info
        fail_ci_if_error: true

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Use Node.js 18.x
      uses: actions/setup-node@v3
      with:
        node-version: 18.x
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Build application
      run: npm run build
    
    - name: Build Docker image
      run: docker build -t myapp:latest .
"#);
    
    write_file(".github/workflows/ci.yml", &github_workflow);
    
    echo("✅ CI/CD configuration created");
}

fn setup_docker_configuration(framework: &str) {
    echo("🐳 Setting up Docker configuration...");
    
    // Create Dockerfile
    let dockerfile = format!(r#"# Use official Node.js runtime as base image
FROM node:18-alpine

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy application code
COPY src/ ./src/
COPY public/ ./public/

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodeuser -u 1001

# Change ownership of the app directory
RUN chown -R nodeuser:nodejs /app
USER nodeuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Start application
CMD ["npm", "start"]
"#);
    
    write_file("Dockerfile", &dockerfile);
    
    // Create .dockerignore
    let dockerignore = r#"node_modules/
npm-debug.log
coverage/
.git/
.gitignore
README.md
.env
.env.local
.env.*.local
tests/
docs/
*.md
.vscode/
.idea/
"#;
    
    write_file(".dockerignore", dockerignore);
    
    // Create docker-compose.yml
    let docker_compose = r#"version: '3.8'

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    volumes:
      - ./logs:/app/logs
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=myapp
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    restart: unless-stopped

volumes:
  postgres_data:
"#;
    
    write_file("docker-compose.yml", docker_compose);
    
    echo("✅ Docker configuration created");
}

fn create_sample_application(framework: &str, database: &str) {
    echo("📝 Creating sample application code...");
    
    match framework {
        "express" => create_express_app(database),
        "fastify" => create_fastify_app(database),
        "nest" => create_nest_app(database),
        "next" => create_next_app(),
        _ => panic!("Unsupported framework: {}", framework),
    }
}

fn create_express_app(database: &str) {
    let main_app = format!(r#"const express = require('express');
const helmet = require('helmet');
const cors = require('cors');
const morgan = require('morgan');
const compression = require('compression');
const rateLimit = require('express-rate-limit');
require('dotenv').config();

const app = express();
const PORT = process.env.PORT || 3000;

// Security middleware - helmet for headers, CORS protection
app.use(helmet());
app.use(cors());

// Rate limiting
const limiter = rateLimit({{
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
}});
app.use(limiter);

// General middleware
app.use(compression());
app.use(morgan('combined'));
app.use(express.json({{ limit: '10mb' }}));
app.use(express.urlencoded({{ extended: true }}));

// Serve static files
app.use(express.static('public'));

{}

// Routes
app.get('/', (req, res) => {{
  res.json({{
    message: 'Welcome to the API',
    version: '1.0.0',
    environment: process.env.NODE_ENV || 'development'
  }});
}});

app.get('/health', (req, res) => {{
  res.json({{
    status: 'OK',
    timestamp: new Date().toISOString(),
    uptime: process.uptime()
  }});
}});

// API routes
const apiRoutes = require('./routes');
app.use('/api/v1', apiRoutes);

// Error handling middleware
app.use((err, req, res, next) => {{
  console.error(err.stack);
  res.status(500).json({{
    error: 'Something went wrong!',
    message: process.env.NODE_ENV === 'development' ? err.message : 'Internal server error'
  }});
}});

// 404 handler
app.use('*', (req, res) => {{
  res.status(404).json({{
    error: 'Route not found',
    path: req.originalUrl
  }});
}});

// Graceful shutdown
process.on('SIGTERM', () => {{
  console.log('SIGTERM received, shutting down gracefully');
  process.exit(0);
}});

process.on('SIGINT', () => {{
  console.log('SIGINT received, shutting down gracefully');
  process.exit(0);
}});

if (require.main === module) {{
  app.listen(PORT, () => {{
    console.log(`Server running on port ${{PORT}} in ${{process.env.NODE_ENV || 'development'}} mode`);
  }});
}}

module.exports = app;
"#, if database != "none" { "// Database connection\nconst connectDB = require('./config/database');\nconnectDB();" } else { "" });
    
    write_file("src/index.js", &main_app);
    
    // Create routes
    let routes = r#"const express = require('express');
const router = express.Router();

// Example routes
router.get('/users', (req, res) => {
  res.json({ message: 'Get all users' });
});

router.post('/users', (req, res) => {
  res.json({ message: 'Create user', data: req.body });
});

router.get('/users/:id', (req, res) => {
  res.json({ message: `Get user ${req.params.id}` });
});

module.exports = router;
"#;
    
    write_file("src/routes/index.js", routes);
}

fn create_fastify_app(database: &str) {
    let main_app = format!(r#"const fastify = require('fastify')({{
  logger: {{
    level: 'info',
    prettyPrint: process.env.NODE_ENV === 'development'
  }}
}});

require('dotenv').config();

// Register plugins
fastify.register(require('@fastify/helmet'), {{
  contentSecurityPolicy: false
}});

fastify.register(require('@fastify/cors'), {{
  origin: true
}});

fastify.register(require('@fastify/compress'));

fastify.register(require('@fastify/rate-limit'), {{
  max: 100,
  timeWindow: '1 minute'
}});

fastify.register(require('@fastify/static'), {{
  root: require('path').join(__dirname, '../public'),
  prefix: '/public/'
}});

{}

// Routes
fastify.get('/', async (request, reply) => {{
  return {{
    message: 'Welcome to the Fastify API',
    version: '1.0.0',
    environment: process.env.NODE_ENV || 'development'
  }};
}});

fastify.get('/health', async (request, reply) => {{
  return {{
    status: 'OK',
    timestamp: new Date().toISOString(),
    uptime: process.uptime()
  }};
}});

// API routes
fastify.register(require('./routes'), {{ prefix: '/api/v1' }});

// Error handler
fastify.setErrorHandler((error, request, reply) => {{
  fastify.log.error(error);
  reply.status(500).send({{
    error: 'Something went wrong!',
    message: process.env.NODE_ENV === 'development' ? error.message : 'Internal server error'
  }});
}});

// 404 handler
fastify.setNotFoundHandler((request, reply) => {{
  reply.status(404).send({{
    error: 'Route not found',
    path: request.url
  }});
}});

const start = async () => {{
  try {{
    const PORT = process.env.PORT || 3000;
    await fastify.listen({{ port: PORT, host: '0.0.0.0' }});
    fastify.log.info(`Server running on port ${{PORT}} in ${{process.env.NODE_ENV || 'development'}} mode`);
  }} catch (err) {{
    fastify.log.error(err);
    process.exit(1);
  }}
}};

if (require.main === module) {{
  start();
}}

module.exports = fastify;
"#, if database != "none" { "// Database connection\nconst connectDB = require('./config/database');\nconnectDB();" } else { "" });
    
    write_file("src/index.js", &main_app);
    
    // Create routes for Fastify
    let routes = r#"async function routes(fastify, options) {
  // Example routes
  fastify.get('/users', async (request, reply) => {
    return { message: 'Get all users' };
  });

  fastify.post('/users', async (request, reply) => {
    return { message: 'Create user', data: request.body };
  });

  fastify.get('/users/:id', async (request, reply) => {
    return { message: `Get user ${request.params.id}` };
  });
}

module.exports = routes;
"#;
    
    write_file("src/routes/index.js", routes);
}

fn create_next_app() {
    // Create Next.js app structure
    let app_layout = r#"import './globals.css'

export const metadata = {
  title: 'My Next.js App',
  description: 'Created with Node.js bootstrap installer',
}

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
"#;
    
    write_file("src/app/layout.js", app_layout);
    
    let app_page = r#"export default function Home() {
  return (
    <main>
      <h1>Welcome to My Next.js App</h1>
      <p>This app was created with the Node.js bootstrap installer.</p>
    </main>
  )
}
"#;
    
    write_file("src/app/page.js", app_page);
    
    let globals_css = r#"html,
body {
  padding: 0;
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Oxygen,
    Ubuntu, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif;
}

a {
  color: inherit;
  text-decoration: none;
}

* {
  box-sizing: border-box;
}
"#;
    
    write_file("src/app/globals.css", globals_css);
}

fn setup_environment_configuration() {
    echo("🔧 Setting up environment configuration...");
    
    let env_example = r#"# Application Configuration
NODE_ENV=development
PORT=3000

# Security Configuration - Replace with secure random values in production
JWT_SECRET=your-super-secret-jwt-key-here
SESSION_SECRET=your-session-secret-here

# CORS
CORS_ORIGIN=http://localhost:3000

# Rate Limiting
RATE_LIMIT_WINDOW_MS=900000
RATE_LIMIT_MAX_REQUESTS=100

# Logging
LOG_LEVEL=info
LOG_FILE=./logs/app.log

# API Keys (if needed)
API_KEY=your-api-key-here
"#;
    
    write_file(".env.example", env_example);
    
    // Create .gitignore
    let gitignore = r#"# Dependencies
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Environment variables
.env
.env.local
.env.development.local
.env.test.local
.env.production.local

# Runtime data
pids
*.pid
*.seed
*.pid.lock

# Coverage directory used by tools like istanbul
coverage/
*.lcov

# Build outputs
dist/
build/

# Logs
logs/
*.log

# Runtime data
tmp/
temp/

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# IDE files
.vscode/
.idea/
*.swp
*.swo

# Optional npm cache directory
.npm

# Optional eslint cache
.eslintcache

# Microbundle cache
.rpt2_cache/
.rts2_cache_cjs/
.rts2_cache_es/
.rts2_cache_umd/

# Optional REPL history
.node_repl_history

# Output of 'npm pack'
*.tgz

# Yarn Integrity file
.yarn-integrity

# parcel-bundler cache (https://parceljs.org/)
.cache
.parcel-cache

# next.js build output
.next

# nuxt.js build output
.nuxt

# Storybook build outputs
.out
.storybook-out

# Database
*.sqlite
*.sqlite3
data/
"#;
    
    write_file(".gitignore", gitignore);
    
    echo("✅ Environment configuration created");
}

fn install_dependencies(manager: &str) {
    echo(&format!("📦 Installing all dependencies with {}...", manager));
    
    match manager {
        "npm" => exec("npm install"),
        "yarn" => exec("yarn install"),
        "pnpm" => exec("pnpm install"),
        _ => panic!("Unknown package manager: {}", manager),
    }
    
    echo("✅ Dependencies installed successfully");
}

fn create_project_documentation(project_name: &str, framework: &str, database: &str) {
    echo("📚 Creating project documentation...");
    
    let readme = format!(r#"# {}

A {} application built with Node.js{}

## Features

- 🚀 {} web framework
- 🔒 Security with Helmet, CORS, and rate limiting
- 🧪 Testing with Jest/Mocha
- 📦 Package management with npm/yarn/pnpm
- 🐳 Docker support
- 🔄 CI/CD with GitHub Actions
- 📝 Code quality with ESLint and Prettier
- 🎯 Environment-based configuration
{}

## Prerequisites

- Node.js 18.0.0 or higher
- npm 8.0.0 or higher

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd {}
```

2. Install dependencies:
```bash
npm install
```

3. Copy environment variables:
```bash
cp .env.example .env
```

4. Update the `.env` file with your configuration

## Development

Start the development server:
```bash
npm run dev
```

The application will be available at `http://localhost:3000`

## Testing

Run tests:
```bash
npm test
```

Run tests with coverage:
```bash
npm run test:coverage
```

## Production

Build the application:
```bash
npm run build
```

Start the production server:
```bash
npm start
```

## Docker

Build and run with Docker:
```bash
npm run docker:build
npm run docker:run
```

Or use Docker Compose:
```bash
docker-compose up
```

## API Documentation

### Health Check
- **GET** `/health` - Returns application health status

### API Routes
- **GET** `/api/v1/users` - Get all users
- **POST** `/api/v1/users` - Create a new user
- **GET** `/api/v1/users/:id` - Get user by ID

## Project Structure

```
{}
├── src/
│   ├── controllers/     # Route controllers
│   ├── models/         # Data models
│   ├── routes/         # API routes
│   ├── middleware/     # Custom middleware
│   ├── services/       # Business logic
│   ├── utils/          # Utility functions
│   ├── config/         # Configuration files
│   └── index.js        # Application entry point
├── tests/
│   ├── unit/           # Unit tests
│   ├── integration/    # Integration tests
│   └── e2e/           # End-to-end tests
├── public/             # Static files
├── docs/              # Documentation
├── scripts/           # Build and deployment scripts
└── docker-compose.yml # Docker composition
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
"#, 
        project_name,
        framework,
        if database != "none" { &format!(" and {} database", database) } else { "" },
        framework,
        if database != "none" { &format!("- 🗄️ {} database integration", database) } else { "" },
        project_name,
        project_name
    );
    
    write_file("README.md", &readme);
    
    // Create CHANGELOG.md
    let changelog = r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2024-01-01

### Added
- Initial project setup
- Basic API structure
- Testing framework
- Docker configuration
- CI/CD pipeline
- Documentation

[Unreleased]: https://github.com/username/project/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/username/project/releases/tag/v1.0.0
"#;
    
    write_file("CHANGELOG.md", changelog);
    
    // Create LICENSE
    let license = r#"MIT License

Copyright (c) 2024

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;
    
    write_file("LICENSE", license);
    
    echo("✅ Project documentation created");
}

// Utility functions
fn command_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

fn env_var_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn append_to_file(path: &str, content: &str) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Failed to open file");
    
    std::io::Write::write_all(&mut file, content.as_bytes())
        .expect("Failed to write to file");
}

fn touch(path: &str) {
    std::fs::File::create(path).expect("Failed to create file");
}