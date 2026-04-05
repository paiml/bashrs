// Node.js project bootstrap installer
// Demonstrates setting up a complete Node.js development environment

#[bashrs::main]
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


include!("nodejs_project_bootstrap_setup_2.rs");
