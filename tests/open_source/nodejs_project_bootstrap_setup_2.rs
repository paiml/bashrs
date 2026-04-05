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


include!("nodejs_project_bootstrap_setup.rs");
