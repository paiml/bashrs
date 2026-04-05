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