// Node.js project bootstrap installer
// Demonstrates setting up a complete Node.js development environment


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


include!("nodejs_project_bootstrap_part3_incl2.rs");
