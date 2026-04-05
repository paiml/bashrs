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


include!("nodejs_project_bootstrap_create.rs");
