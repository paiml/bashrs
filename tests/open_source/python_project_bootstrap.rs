// Python project bootstrap installer
// Demonstrates setting up a complete Python development environment

#[rash::main]
fn python_project_bootstrap() {
    let project_name = env_var_or("PROJECT_NAME", "my-python-app");
    let python_version = env_var_or("PYTHON_VERSION", "3.11");
    let package_manager = env_var_or("PACKAGE_MANAGER", "pip"); // pip, poetry, pipenv
    let framework = env_var_or("FRAMEWORK", "fastapi"); // django, flask, fastapi, none
    let database = env_var_or("DATABASE", "none"); // postgresql, mysql, sqlite, mongodb, none
    let testing_framework = env_var_or("TESTING", "pytest"); // pytest, unittest, nose2
    let async_support = env_var_or("ASYNC", "true") == "true";
    
    echo("🐍 Python Project Bootstrap Installer");
    echo(&format!("Project: {}, Python: {}, Framework: {}", project_name, python_version, framework));
    
    // Detect operating system
    let os_info = detect_operating_system();
    
    // Install Python via pyenv
    install_python_via_pyenv(&python_version, &os_info);
    
    // Install package manager
    install_package_manager(&package_manager);
    
    // Create project structure
    create_project_structure(&project_name);
    
    // Setup virtual environment
    setup_virtual_environment(&project_name, &package_manager);
    
    // Initialize project configuration
    initialize_project_config(&project_name, &package_manager, &framework, &testing_framework);
    
    // Install framework and dependencies
    install_framework_dependencies(&framework, &package_manager, async_support);
    
    // Setup database integration
    if database != "none" {
        setup_database_integration(&database, &package_manager, async_support);
    }
    
    // Setup testing framework
    setup_testing_framework(&testing_framework, &package_manager);
    
    // Setup development tools
    setup_development_tools(&package_manager);
    
    // Setup type checking and linting
    setup_code_quality_tools(&package_manager);
    
    // Setup CI/CD configuration
    setup_cicd_configuration(&testing_framework);
    
    // Setup Docker configuration
    setup_docker_configuration(&framework, &python_version);
    
    // Create sample application code
    create_sample_application(&framework, &database, async_support);
    
    // Setup environment configuration
    setup_environment_configuration();
    
    // Install dependencies
    install_dependencies(&package_manager);
    
    // Create documentation
    create_project_documentation(&project_name, &framework, &database);
    
    echo("✅ Python project bootstrap completed successfully");
    echo(&format!("cd {} && source venv/bin/activate && python -m src.main", project_name));
}

fn detect_operating_system() -> String {
    if path_exists("/etc/os-release") {
        let os_release = read_file("/etc/os-release");
        if os_release.contains("Ubuntu") {
            return "ubuntu".to_string();
        } else if os_release.contains("Debian") {
            return "debian".to_string();
        } else if os_release.contains("CentOS") || os_release.contains("Red Hat") {
            return "rhel".to_string();
        }
    }
    
    if command_exists("sw_vers") {
        return "macos".to_string();
    }
    
    "linux".to_string()
}

fn install_python_via_pyenv(version: &str, os: &str) {
    echo(&format!("🔧 Installing Python {} via pyenv...", version));
    
    // Install pyenv dependencies
    match os {
        "ubuntu" | "debian" => {
            exec("apt-get update");
            exec("apt-get install -y make build-essential libssl-dev zlib1g-dev libbz2-dev libreadline-dev libsqlite3-dev wget curl llvm libncursesw5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev");
        },
        "rhel" => {
            exec("yum groupinstall -y \"Development Tools\"");
            exec("yum install -y gcc make patch zlib-devel bzip2 bzip2-devel readline-devel sqlite sqlite-devel openssl-devel tk-devel libffi-devel xz-devel");
        },
        "macos" => {
            // Assume Homebrew is available
            if command_exists("brew") {
                exec("brew install openssl readline sqlite3 xz zlib tcl-tk");
            }
        },
        _ => {
            echo("Warning: Unknown OS, skipping dependency installation");
        }
    }
    
    // Install pyenv
    if !command_exists("pyenv") {
        exec("curl https://pyenv.run | bash");
        
        // Add pyenv to PATH
        let shell_rc = if path_exists(&format!("{}/.zshrc", env_var("HOME"))) {
            "~/.zshrc"
        } else {
            "~/.bashrc"
        };
        
        let pyenv_config = r#"
# Pyenv configuration
export PYENV_ROOT="$HOME/.pyenv"
command -v pyenv >/dev/null || export PATH="$PYENV_ROOT/bin:$PATH"
eval "$(pyenv init -)"
"#;
        
        append_to_file(shell_rc, pyenv_config);
        
        // Source the configuration
        exec(&format!("source {}", shell_rc));
    }
    
    // Install specific Python version
    exec(&format!("pyenv install {}", version));
    exec(&format!("pyenv global {}", version));
    
    // Verify installation
    exec("python --version");
    exec("pip --version");
    
    echo("✅ Python installed successfully");
}

fn install_package_manager(manager: &str) {
    echo(&format!("📦 Setting up package manager: {}", manager));
    
    match manager {
        "pip" => {
            // pip comes with Python, just upgrade it
            exec("python -m pip install --upgrade pip");
        },
        "poetry" => {
            exec("curl -sSL https://install.python-poetry.org | python3 -");
            // Add poetry to PATH
            let poetry_path = format!("{}/.local/bin", env_var("HOME"));
            exec(&format!("export PATH={}:$PATH", poetry_path));
            exec("poetry --version");
        },
        "pipenv" => {
            exec("python -m pip install pipenv");
            exec("pipenv --version");
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
    
    // Create standard Python project structure
    let directories = [
        "src",
        "src/config",
        "src/models",
        "src/services",
        "src/utils",
        "src/api",
        "src/api/routes",
        "src/api/middleware",
        "src/database",
        "src/database/migrations",
        "tests",
        "tests/unit",
        "tests/integration",
        "tests/e2e",
        "tests/fixtures",
        "docs",
        "scripts",
        "static",
        "static/css",
        "static/js",
        "static/images",
        "templates",
        "logs",
        "data",
    ];
    
    for dir in &directories {
        mkdir_p(dir);
        // Create __init__.py files for Python packages
        if dir.starts_with("src") || dir.starts_with("tests") {
            touch(&format!("{}/__init__.py", dir));
        }
    }
    
    // Create essential files
    touch(".env");
    touch(".env.example");
    touch(".gitignore");
    touch("README.md");
    touch("CHANGELOG.md");
    touch("LICENSE");
    touch("requirements.txt");
    touch("requirements-dev.txt");
    
    echo("✅ Project structure created");
}

fn setup_virtual_environment(project_name: &str, manager: &str) {
    echo("🏠 Setting up virtual environment...");
    
    match manager {
        "pip" => {
            exec("python -m venv venv");
            exec("source venv/bin/activate && pip install --upgrade pip");
        },
        "poetry" => {
            exec("poetry init --no-interaction");
            exec("poetry env use python");
        },
        "pipenv" => {
            exec("pipenv --python 3.11");
        },
        _ => panic!("Unsupported package manager for venv: {}", manager),
    }
    
    echo("✅ Virtual environment created");
}

fn initialize_project_config(name: &str, manager: &str, framework: &str, testing: &str) {
    echo("📋 Initializing project configuration...");
    
    match manager {
        "pip" => {
            // Create setup.py
            let setup_py = format!(r#"#!/usr/bin/env python3
"""Setup script for {}."""

from setuptools import setup, find_packages
import os

# Read README
def read_readme():
    with open('README.md', 'r', encoding='utf-8') as f:
        return f.read()

# Read requirements
def read_requirements(filename):
    with open(filename, 'r', encoding='utf-8') as f:
        return [line.strip() for line in f if line.strip() and not line.startswith('#')]

setup(
    name="{}",
    version="1.0.0",
    description="A {} application built with Python",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    author="Your Name",
    author_email="your.email@example.com",
    url="https://github.com/yourusername/{}",
    packages=find_packages(where="src"),
    package_dir={{"": "src"}},
    python_requires=">=3.8",
    install_requires=read_requirements("requirements.txt"),
    extras_require={{
        "dev": read_requirements("requirements-dev.txt"),
        "test": [
            "{}",
            "pytest-cov",
            "pytest-asyncio",
        ],
    }},
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    entry_points={{
        "console_scripts": [
            "{}=src.main:main",
        ],
    }},
    include_package_data=True,
    zip_safe=False,
)
"#, name, name, framework, name, testing, name);
            
            write_file("setup.py", &setup_py);
        },
        "poetry" => {
            // Poetry will create pyproject.toml, but we'll enhance it
            let pyproject_toml = format!(r#"[tool.poetry]
name = "{}"
version = "1.0.0"
description = "A {} application built with Python"
authors = ["Your Name <your.email@example.com>"]
readme = "README.md"
packages = [{{include = "src", from = "."}}]

[tool.poetry.dependencies]
python = "^3.8"

[tool.poetry.group.dev.dependencies]
{} = "*"
pytest-cov = "*"
pytest-asyncio = "*"
black = "*"
isort = "*"
flake8 = "*"
mypy = "*"
pre-commit = "*"

[tool.poetry.scripts]
{} = "src.main:main"

[build-system]
requires = ["poetry-core"]
build-backend = "poetry.core.masonry.api"

[tool.black]
line-length = 88
target-version = ["py38", "py39", "py310", "py311"]
include = '\.pyi?$'
extend-exclude = '''
/(
  # directories
  \.eggs
  | \.git
  | \.hg
  | \.mypy_cache
  | \.tox
  | \.venv
  | build
  | dist
)/
'''

[tool.isort]
profile = "black"
multi_line_output = 3
line_length = 88
known_first_party = ["src"]

[tool.mypy]
python_version = "3.8"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true
disallow_incomplete_defs = true
check_untyped_defs = true
disallow_untyped_decorators = true
no_implicit_optional = true
warn_redundant_casts = true
warn_unused_ignores = true
warn_no_return = true
warn_unreachable = true
strict_equality = true

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py", "*_test.py"]
python_classes = ["Test*"]
python_functions = ["test_*"]
addopts = [
    "--strict-markers",
    "--strict-config",
    "--verbose",
    "--cov=src",
    "--cov-report=term-missing",
    "--cov-report=html",
    "--cov-report=xml",
]
markers = [
    "slow: marks tests as slow (deselect with '-m \"not slow\"')",
    "integration: marks tests as integration tests",
    "unit: marks tests as unit tests",
]

[tool.coverage.run]
source = ["src"]
omit = [
    "*/tests/*",
    "*/migrations/*",
    "*/__init__.py",
]

[tool.coverage.report]
exclude_lines = [
    "pragma: no cover",
    "def __repr__",
    "if self.debug:",
    "if settings.DEBUG",
    "raise AssertionError",
    "raise NotImplementedError",
    "if 0:",
    "if __name__ == .__main__.:",
    "class .*\\bProtocol\\):",
    "@(abc\\.)?abstractmethod",
]
"#, name, framework, testing, name);
            
            write_file("pyproject.toml", &pyproject_toml);
        },
        "pipenv" => {
            // Pipenv creates Pipfile automatically, but we'll enhance it
            let pipfile = format!(r#"[[source]]
url = "https://pypi.org/simple"
verify_ssl = true
name = "pypi"

[packages]

[dev-packages]
{} = "*"
pytest-cov = "*"
pytest-asyncio = "*"
black = "*"
isort = "*"
flake8 = "*"
mypy = "*"
pre-commit = "*"

[requires]
python_version = "3.11"

[scripts]
test = "{} tests/"
test-cov = "{} --cov=src tests/"
lint = "flake8 src tests"
format = "black src tests && isort src tests"
typecheck = "mypy src"
"#, testing, testing, testing);
            
            write_file("Pipfile", &pipfile);
        },
        _ => {},
    }
    
    echo("✅ Project configuration initialized");
}

fn install_framework_dependencies(framework: &str, manager: &str, async_support: bool) {
    echo(&format!("🚀 Installing {} framework dependencies...", framework));
    
    let dependencies = match framework {
        "django" => {
            let mut deps = vec![
                "Django>=4.2,<5.0",
                "djangorestframework",
                "django-cors-headers",
                "django-environ",
                "celery",
                "redis",
                "gunicorn",
                "whitenoise",
                "django-extensions",
            ];
            if async_support {
                deps.push("channels[daphne]");
                deps.push("channels-redis");
            }
            deps
        },
        "flask" => {
            let mut deps = vec![
                "Flask>=2.3.0",
                "Flask-SQLAlchemy",
                "Flask-Migrate",
                "Flask-CORS",
                "Flask-JWT-Extended",
                "Flask-Limiter",
                "python-dotenv",
                "gunicorn",
                "marshmallow",
                "webargs",
            ];
            if async_support {
                deps.push("Flask[async]");
                deps.push("asyncio");
            }
            deps
        },
        "fastapi" => {
            let mut deps = vec![
                "fastapi>=0.104.0",
                "uvicorn[standard]",
                "pydantic>=2.0.0",
                "python-multipart",
                "python-jose[cryptography]",
                "passlib[bcrypt]",
                "python-dotenv",
                "slowapi",
                "httpx",
            ];
            if async_support {
                deps.push("aiofiles");
                deps.push("asyncio");
            }
            deps
        },
        "none" => vec![
            "requests",
            "click",
            "rich",
            "python-dotenv",
        ],
        _ => panic!("Unsupported framework: {}", framework),
    };
    
    // Install dependencies based on package manager
    match manager {
        "pip" => {
            let deps_str = dependencies.join(" ");
            exec(&format!("source venv/bin/activate && pip install {}", deps_str));
            
            // Update requirements.txt
            for dep in dependencies {
                append_to_file("requirements.txt", &format!("{}\n", dep));
            }
        },
        "poetry" => {
            for dep in dependencies {
                exec(&format!("poetry add {}", dep));
            }
        },
        "pipenv" => {
            let deps_str = dependencies.join(" ");
            exec(&format!("pipenv install {}", deps_str));
        },
        _ => panic!("Unsupported package manager: {}", manager),
    }
    
    echo(&format!("✅ {} dependencies installed", framework));
}

fn setup_database_integration(database: &str, manager: &str, async_support: bool) {
    echo(&format!("🗄️ Setting up {} database integration...", database));
    
    let dependencies = match database {
        "postgresql" => {
            let mut deps = vec!["psycopg2-binary"];
            if async_support {
                deps.push("asyncpg");
            }
            deps.push("alembic"); // For migrations
            deps
        },
        "mysql" => {
            let mut deps = vec!["PyMySQL"];
            if async_support {
                deps.push("aiomysql");
            }
            deps.push("alembic");
            deps
        },
        "sqlite" => {
            let mut deps = vec![];
            if async_support {
                deps.push("aiosqlite");
            }
            deps.push("alembic");
            deps
        },
        "mongodb" => {
            let mut deps = vec!["pymongo"];
            if async_support {
                deps.push("motor");
            }
            deps
        },
        _ => panic!("Unsupported database: {}", database),
    };
    
    // Install database dependencies
    match manager {
        "pip" => {
            let deps_str = dependencies.join(" ");
            exec(&format!("source venv/bin/activate && pip install {}", deps_str));
            
            for dep in dependencies {
                append_to_file("requirements.txt", &format!("{}\n", dep));
            }
        },
        "poetry" => {
            for dep in dependencies {
                exec(&format!("poetry add {}", dep));
            }
        },
        "pipenv" => {
            let deps_str = dependencies.join(" ");
            exec(&format!("pipenv install {}", deps_str));
        },
        _ => {},
    }
    
    // Create database configuration
    create_database_config(database, async_support);
    
    echo(&format!("✅ {} integration configured", database));
}

fn create_database_config(database: &str, async_support: bool) {
    match database {
        "postgresql" => {
            let pg_config = if async_support {
                r#"""PostgreSQL database configuration with async support."""

import asyncpg
import asyncio
from typing import Optional
import os

class DatabaseManager:
    def __init__(self):
        self.pool: Optional[asyncpg.Pool] = None
        self.database_url = os.getenv(
            "DATABASE_URL", 
            "postgresql://postgres:password@localhost/myapp"
        )
    
    async def connect(self):
        """Create database connection pool."""
        try:
            self.pool = await asyncpg.create_pool(
                self.database_url,
                min_size=1,
                max_size=10,
                command_timeout=60,
            )
            print("Connected to PostgreSQL database")
        except Exception as e:
            print(f"Error connecting to database: {e}")
            raise
    
    async def disconnect(self):
        """Close database connection pool."""
        if self.pool:
            await self.pool.close()
            print("Database connection closed")
    
    async def execute_query(self, query: str, *args):
        """Execute a query and return results."""
        async with self.pool.acquire() as connection:
            return await connection.fetch(query, *args)
    
    async def execute_command(self, command: str, *args):
        """Execute a command (INSERT, UPDATE, DELETE)."""
        async with self.pool.acquire() as connection:
            return await connection.execute(command, *args)

# Global database manager instance
db = DatabaseManager()
"#
            } else {
                r#"""PostgreSQL database configuration."""

import psycopg2
from psycopg2.pool import ThreadedConnectionPool
import os
from contextlib import contextmanager

class DatabaseManager:
    def __init__(self):
        self.pool = None
        self.database_url = os.getenv(
            "DATABASE_URL",
            "postgresql://postgres:password@localhost/myapp"
        )
    
    def connect(self):
        """Create database connection pool."""
        try:
            self.pool = ThreadedConnectionPool(
                1, 20,  # min and max connections
                self.database_url
            )
            print("Connected to PostgreSQL database")
        except Exception as e:
            print(f"Error connecting to database: {e}")
            raise
    
    def disconnect(self):
        """Close all database connections."""
        if self.pool:
            self.pool.closeall()
            print("Database connections closed")
    
    @contextmanager
    def get_connection(self):
        """Context manager for database connections."""
        connection = self.pool.getconn()
        try:
            yield connection
        finally:
            self.pool.putconn(connection)
    
    def execute_query(self, query: str, params=None):
        """Execute a query and return results."""
        with self.get_connection() as conn:
            with conn.cursor() as cursor:
                cursor.execute(query, params)
                return cursor.fetchall()
    
    def execute_command(self, command: str, params=None):
        """Execute a command (INSERT, UPDATE, DELETE)."""
        with self.get_connection() as conn:
            with conn.cursor() as cursor:
                cursor.execute(command, params)
                conn.commit()
                return cursor.rowcount

# Global database manager instance
db = DatabaseManager()
"#
            };
            
            write_file("src/database/connection.py", pg_config);
            
            // Add to .env.example
            append_to_file(".env.example", "DATABASE_URL=postgresql://postgres:password@localhost/myapp\n");
        },
        "mysql" => {
            let mysql_config = if async_support {
                r#"""MySQL database configuration with async support."""

import aiomysql
import asyncio
from typing import Optional
import os

class DatabaseManager:
    def __init__(self):
        self.pool: Optional[aiomysql.Pool] = None
        self.config = {
            'host': os.getenv('DB_HOST', 'localhost'),
            'port': int(os.getenv('DB_PORT', 3306)),
            'user': os.getenv('DB_USER', 'root'),
            'password': os.getenv('DB_PASSWORD', ''),
            'db': os.getenv('DB_NAME', 'myapp'),
            'charset': 'utf8mb4',
            'autocommit': True,
        }
    
    async def connect(self):
        """Create database connection pool."""
        try:
            self.pool = await aiomysql.create_pool(
                minsize=1,
                maxsize=10,
                **self.config
            )
            print("Connected to MySQL database")
        except Exception as e:
            print(f"Error connecting to database: {e}")
            raise
    
    async def disconnect(self):
        """Close database connection pool."""
        if self.pool:
            self.pool.close()
            await self.pool.wait_closed()
            print("Database connection closed")
    
    async def execute_query(self, query: str, args=None):
        """Execute a query and return results."""
        async with self.pool.acquire() as conn:
            async with conn.cursor() as cursor:
                await cursor.execute(query, args)
                return await cursor.fetchall()
    
    async def execute_command(self, command: str, args=None):
        """Execute a command (INSERT, UPDATE, DELETE)."""
        async with self.pool.acquire() as conn:
            async with conn.cursor() as cursor:
                await cursor.execute(command, args)
                return cursor.rowcount

# Global database manager instance
db = DatabaseManager()
"#
            } else {
                r#"""MySQL database configuration."""

import PyMySQL
from PyMySQL.pooling import ConnectionPool
import os
from contextlib import contextmanager

class DatabaseManager:
    def __init__(self):
        self.pool = None
        self.config = {
            'host': os.getenv('DB_HOST', 'localhost'),
            'port': int(os.getenv('DB_PORT', 3306)),
            'user': os.getenv('DB_USER', 'root'),
            'password': os.getenv('DB_PASSWORD', ''),
            'database': os.getenv('DB_NAME', 'myapp'),
            'charset': 'utf8mb4',
            'autocommit': True,
        }
    
    def connect(self):
        """Create database connection pool."""
        try:
            self.pool = ConnectionPool(
                size=20,
                name='myapp_pool',
                **self.config
            )
            print("Connected to MySQL database")
        except Exception as e:
            print(f"Error connecting to database: {e}")
            raise
    
    def disconnect(self):
        """Close database connections."""
        if self.pool:
            self.pool.close()
            print("Database connections closed")
    
    @contextmanager
    def get_connection(self):
        """Context manager for database connections."""
        connection = self.pool.get_connection()
        try:
            yield connection
        finally:
            connection.close()
    
    def execute_query(self, query: str, args=None):
        """Execute a query and return results."""
        with self.get_connection() as conn:
            with conn.cursor() as cursor:
                cursor.execute(query, args)
                return cursor.fetchall()
    
    def execute_command(self, command: str, args=None):
        """Execute a command (INSERT, UPDATE, DELETE)."""
        with self.get_connection() as conn:
            with conn.cursor() as cursor:
                cursor.execute(command, args)
                return cursor.rowcount

# Global database manager instance
db = DatabaseManager()
"#
            };
            
            write_file("src/database/connection.py", mysql_config);
            
            append_to_file(".env.example", "DB_HOST=localhost\nDB_PORT=3306\nDB_USER=root\nDB_PASSWORD=\nDB_NAME=myapp\n");
        },
        "mongodb" => {
            let mongo_config = if async_support {
                r#"""MongoDB database configuration with async support."""

from motor.motor_asyncio import AsyncIOMotorClient
from typing import Optional
import os

class DatabaseManager:
    def __init__(self):
        self.client: Optional[AsyncIOMotorClient] = None
        self.database = None
        self.mongodb_url = os.getenv(
            "MONGODB_URL",
            "mongodb://localhost:27017/myapp"
        )
    
    async def connect(self):
        """Create database connection."""
        try:
            self.client = AsyncIOMotorClient(self.mongodb_url)
            # Extract database name from URL or use default
            db_name = self.mongodb_url.split('/')[-1] or 'myapp'
            self.database = self.client[db_name]
            
            # Test the connection
            await self.client.admin.command('ping')
            print("Connected to MongoDB database")
        except Exception as e:
            print(f"Error connecting to database: {e}")
            raise
    
    async def disconnect(self):
        """Close database connection."""
        if self.client:
            self.client.close()
            print("Database connection closed")
    
    def get_collection(self, collection_name: str):
        """Get a collection from the database."""
        return self.database[collection_name]

# Global database manager instance
db = DatabaseManager()
"#
            } else {
                r#"""MongoDB database configuration."""

from pymongo import MongoClient
from typing import Optional
import os

class DatabaseManager:
    def __init__(self):
        self.client: Optional[MongoClient] = None
        self.database = None
        self.mongodb_url = os.getenv(
            "MONGODB_URL",
            "mongodb://localhost:27017/myapp"
        )
    
    def connect(self):
        """Create database connection."""
        try:
            self.client = MongoClient(self.mongodb_url)
            # Extract database name from URL or use default
            db_name = self.mongodb_url.split('/')[-1] or 'myapp'
            self.database = self.client[db_name]
            
            # Test the connection
            self.client.admin.command('ping')
            print("Connected to MongoDB database")
        except Exception as e:
            print(f"Error connecting to database: {e}")
            raise
    
    def disconnect(self):
        """Close database connection."""
        if self.client:
            self.client.close()
            print("Database connection closed")
    
    def get_collection(self, collection_name: str):
        """Get a collection from the database."""
        return self.database[collection_name]

# Global database manager instance
db = DatabaseManager()
"#
            };
            
            write_file("src/database/connection.py", mongo_config);
            
            append_to_file(".env.example", "MONGODB_URL=mongodb://localhost:27017/myapp\n");
        },
        _ => {},
    }
}

fn setup_testing_framework(testing: &str, manager: &str) {
    echo(&format!("🧪 Setting up {} testing framework...", testing));
    
    let dependencies = match testing {
        "pytest" => vec![
            "pytest>=7.0.0",
            "pytest-cov",
            "pytest-asyncio",
            "pytest-mock",
            "pytest-xdist",
            "pytest-sugar",
            "pytest-html",
            "factory-boy",
            "faker",
        ],
        "unittest" => vec![
            "coverage",
            "mock",
            "faker",
        ],
        "nose2" => vec![
            "nose2",
            "coverage",
            "faker",
        ],
        _ => panic!("Unsupported testing framework: {}", testing),
    };
    
    // Install testing dependencies
    match manager {
        "pip" => {
            let deps_str = dependencies.join(" ");
            exec(&format!("source venv/bin/activate && pip install {}", deps_str));
            
            for dep in dependencies {
                append_to_file("requirements-dev.txt", &format!("{}\n", dep));
            }
        },
        "poetry" => {
            for dep in dependencies {
                exec(&format!("poetry add --group dev {}", dep));
            }
        },
        "pipenv" => {
            let deps_str = dependencies.join(" ");
            exec(&format!("pipenv install --dev {}", deps_str));
        },
        _ => {},
    }
    
    // Create test configuration and sample tests
    create_test_configuration(testing);
    create_sample_tests(testing);
    
    echo(&format!("✅ {} testing framework configured", testing));
}

fn create_test_configuration(testing: &str) {
    match testing {
        "pytest" => {
            let pytest_ini = r#"[tool:pytest]
testpaths = tests
python_files = test_*.py *_test.py
python_classes = Test*
python_functions = test_*
addopts = 
    --strict-markers
    --strict-config
    --verbose
    --tb=short
    --cov=src
    --cov-report=term-missing
    --cov-report=html:htmlcov
    --cov-report=xml:coverage.xml
    --cov-fail-under=80

markers =
    slow: marks tests as slow (deselect with '-m "not slow"')
    integration: marks tests as integration tests
    unit: marks tests as unit tests
    async_test: marks tests as async tests

filterwarnings =
    ignore::UserWarning
    ignore::DeprecationWarning
"#;
            
            write_file("pytest.ini", pytest_ini);
        },
        "unittest" => {
            let unittest_config = r#"""Unit test configuration."""

import unittest
import sys
import os

# Add src to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

class BaseTestCase(unittest.TestCase):
    """Base test case with common setup."""
    
    def setUp(self):
        """Set up test fixtures before each test method."""
        pass
    
    def tearDown(self):
        """Tear down test fixtures after each test method."""
        pass

if __name__ == '__main__':
    # Discover and run all tests
    loader = unittest.TestLoader()
    suite = loader.discover('tests', pattern='test_*.py')
    runner = unittest.TextTestRunner(verbosity=2)
    runner.run(suite)
"#;
            
            write_file("tests/config.py", unittest_config);
        },
        _ => {},
    }
}

fn create_sample_tests(testing: &str) {
    match testing {
        "pytest" => {
            let test_content = r#"""Sample tests using pytest."""

import pytest
import asyncio
from src.main import app, get_health_status

class TestHealthCheck:
    """Test health check functionality."""
    
    def test_health_status_returns_ok(self):
        """Test that health status returns OK."""
        status = get_health_status()
        assert status["status"] == "OK"
        assert "timestamp" in status
        assert "uptime" in status
    
    @pytest.mark.asyncio
    async def test_async_health_check(self):
        """Test async health check if applicable."""
        # This is a placeholder for async testing
        await asyncio.sleep(0.1)
        assert True

class TestApplication:
    """Test main application functionality."""
    
    def test_app_creation(self):
        """Test that the app can be created."""
        assert app is not None
    
    @pytest.mark.slow
    def test_slow_operation(self):
        """Test a slow operation."""
        # Simulate slow operation
        import time
        time.sleep(1)
        assert True

class TestUtilities:
    """Test utility functions."""
    
    def test_example_utility(self):
        """Test example utility function."""
        # Add actual utility tests here
        assert 1 + 1 == 2

# Fixtures
@pytest.fixture
def sample_data():
    """Provide sample data for tests."""
    return {
        "test_value": 42,
        "test_string": "hello world"
    }

@pytest.fixture
async def async_sample_data():
    """Provide async sample data for tests."""
    await asyncio.sleep(0.1)
    return {"async_value": True}

def test_with_fixture(sample_data):
    """Test using fixture data."""
    assert sample_data["test_value"] == 42
    assert sample_data["test_string"] == "hello world"
"#;
            
            write_file("tests/test_main.py", test_content);
        },
        "unittest" => {
            let test_content = r#"""Sample tests using unittest."""

import unittest
import sys
import os

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from main import get_health_status

class TestHealthCheck(unittest.TestCase):
    """Test health check functionality."""
    
    def test_health_status_returns_ok(self):
        """Test that health status returns OK."""
        status = get_health_status()
        self.assertEqual(status["status"], "OK")
        self.assertIn("timestamp", status)
        self.assertIn("uptime", status)

class TestApplication(unittest.TestCase):
    """Test main application functionality."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.test_data = {"value": 42}
    
    def test_example(self):
        """Test example functionality."""
        self.assertEqual(self.test_data["value"], 42)
    
    def tearDown(self):
        """Clean up after tests."""
        pass

if __name__ == '__main__':
    unittest.main()
"#;
            
            write_file("tests/test_main.py", test_content);
        },
        _ => {},
    }
}

fn setup_development_tools(manager: &str) {
    echo("🛠️ Setting up development tools...");
    
    let dev_tools = vec![
        "pre-commit",
        "commitizen",
        "bumpversion",
        "twine",
        "wheel",
        "setuptools-scm",
    ];
    
    // Install development tools
    match manager {
        "pip" => {
            let tools_str = dev_tools.join(" ");
            exec(&format!("source venv/bin/activate && pip install {}", tools_str));
            
            for tool in dev_tools {
                append_to_file("requirements-dev.txt", &format!("{}\n", tool));
            }
        },
        "poetry" => {
            for tool in dev_tools {
                exec(&format!("poetry add --group dev {}", tool));
            }
        },
        "pipenv" => {
            let tools_str = dev_tools.join(" ");
            exec(&format!("pipenv install --dev {}", tools_str));
        },
        _ => {},
    }
    
    // Create pre-commit configuration
    let precommit_config = r#"repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
      - id: check-merge-conflict
      - id: debug-statements
      - id: check-docstring-first
      
  - repo: https://github.com/psf/black
    rev: 23.7.0
    hooks:
      - id: black
        language_version: python3
        
  - repo: https://github.com/pycqa/isort
    rev: 5.12.0
    hooks:
      - id: isort
        args: ["--profile", "black"]
        
  - repo: https://github.com/pycqa/flake8
    rev: 6.0.0
    hooks:
      - id: flake8
        additional_dependencies: [flake8-docstrings]
        
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.5.1
    hooks:
      - id: mypy
        additional_dependencies: [types-requests]
        
  - repo: https://github.com/pycqa/bandit
    rev: 1.7.5
    hooks:
      - id: bandit
        args: ["-c", "pyproject.toml"]
        additional_dependencies: ["bandit[toml]"]
"#;
    
    write_file(".pre-commit-config.yaml", precommit_config);
    
    // Install pre-commit hooks
    exec("pre-commit install");
    
    echo("✅ Development tools configured");
}

fn setup_code_quality_tools(manager: &str) {
    echo("🔍 Setting up code quality tools...");
    
    let quality_tools = vec![
        "black",
        "isort",
        "flake8",
        "mypy",
        "bandit",
        "safety",
        "pylint",
    ];
    
    // Install code quality tools
    match manager {
        "pip" => {
            let tools_str = quality_tools.join(" ");
            exec(&format!("source venv/bin/activate && pip install {}", tools_str));
            
            for tool in quality_tools {
                append_to_file("requirements-dev.txt", &format!("{}\n", tool));
            }
        },
        "poetry" => {
            for tool in quality_tools {
                exec(&format!("poetry add --group dev {}", tool));
            }
        },
        "pipenv" => {
            let tools_str = quality_tools.join(" ");
            exec(&format!("pipenv install --dev {}", tools_str));
        },
        _ => {},
    }
    
    // Create configuration files
    create_code_quality_configs();
    
    echo("✅ Code quality tools configured");
}

fn create_code_quality_configs() {
    // Create .flake8 configuration
    let flake8_config = r#"[flake8]
max-line-length = 88
select = E,W,F,N,C,B
ignore = 
    E203,  # whitespace before ':'
    E501,  # line too long (handled by black)
    W503,  # line break before binary operator
exclude = 
    .git,
    __pycache__,
    .venv,
    venv,
    build,
    dist,
    *.egg-info,
    migrations
per-file-ignores =
    __init__.py:F401
    tests/*:S101
max-complexity = 10
"#;
    
    write_file(".flake8", flake8_config);
    
    // Create mypy configuration
    let mypy_config = r#"[mypy]
python_version = 3.8
warn_return_any = True
warn_unused_configs = True
disallow_untyped_defs = True
disallow_incomplete_defs = True
check_untyped_defs = True
disallow_untyped_decorators = True
no_implicit_optional = True
warn_redundant_casts = True
warn_unused_ignores = True
warn_no_return = True
warn_unreachable = True
strict_equality = True

[mypy-tests.*]
disallow_untyped_defs = False

[mypy-migrations.*]
ignore_errors = True
"#;
    
    write_file("mypy.ini", mypy_config);
    
    // Create bandit configuration
    let bandit_config = r#"[tool.bandit]
exclude_dirs = ["tests", "migrations"]
skips = ["B101", "B601"]
"#;
    
    append_to_file("pyproject.toml", bandit_config);
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
        python-version: ["3.8", "3.9", "3.10", "3.11", "3.12"]
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test_db
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    
    - name: Cache pip packages
      uses: actions/cache@v3
      with:
        path: ~/.cache/pip
        key: ${{{{ runner.os }}}}-pip-${{{{ hashFiles('**/requirements*.txt') }}}}
        restore-keys: |
          ${{{{ runner.os }}}}-pip-
    
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -r requirements.txt
        pip install -r requirements-dev.txt
    
    - name: Lint with flake8
      run: |
        flake8 src tests
    
    - name: Type check with mypy
      run: |
        mypy src
    
    - name: Security check with bandit
      run: |
        bandit -r src
    
    - name: Test with {}
      run: |
        {} tests/
      env:
        DATABASE_URL: postgresql://postgres:postgres@localhost/test_db
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage.xml
        fail_ci_if_error: true

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python 3.11
      uses: actions/setup-python@v4
      with:
        python-version: 3.11
    
    - name: Install build dependencies
      run: |
        python -m pip install --upgrade pip
        pip install build twine
    
    - name: Build package
      run: |
        python -m build
    
    - name: Build Docker image
      run: |
        docker build -t myapp:latest .

  security:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        scan-type: 'fs'
        format: 'sarif'
        output: 'trivy-results.sarif'
    
    - name: Upload Trivy scan results to GitHub Security tab
      uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: 'trivy-results.sarif'
"#, testing, testing);
    
    write_file(".github/workflows/ci.yml", &github_workflow);
    
    echo("✅ CI/CD configuration created");
}

fn setup_docker_configuration(framework: &str, python_version: &str) {
    echo("🐳 Setting up Docker configuration...");
    
    // Create Dockerfile
    let dockerfile = format!(r#"# Use official Python runtime as base image
FROM python:{}-slim

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PYTHONPATH=/app \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

# Set work directory
WORKDIR /app

# Install system dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        curl \
        && rm -rf /var/lib/apt/lists/*

# Copy and install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy project
COPY src/ ./src/
COPY static/ ./static/
COPY templates/ ./templates/

# Create non-root user
RUN adduser --disabled-password --gecos '' appuser && \
    chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Run application
CMD ["python", "-m", "uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
"#, python_version);
    
    write_file("Dockerfile", &dockerfile);
    
    // Create .dockerignore
    let dockerignore = r#"__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg
MANIFEST

.env
.env.*
!.env.example
.venv
venv/
ENV/
env/

.git/
.gitignore
README.md
*.md
.pytest_cache/
.coverage
htmlcov/
.tox/
.cache
nosetests.xml
coverage.xml
*.cover
.hypothesis/

.vscode/
.idea/
*.swp
*.swo
*~

tests/
docs/
*.log
"#;
    
    write_file(".dockerignore", dockerignore);
    
    // Create docker-compose.yml
    let docker_compose = r#"version: '3.8'

services:
  app:
    build: .
    ports:
      - "8000:8000"
    environment:
      - PYTHONPATH=/app
      - DATABASE_URL=postgresql://postgres:password@db:5432/myapp
    volumes:
      - ./logs:/app/logs
    depends_on:
      - db
      - redis
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

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
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 3

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./static:/var/www/static
    depends_on:
      - app
    restart: unless-stopped

volumes:
  postgres_data:
"#;
    
    write_file("docker-compose.yml", docker_compose);
    
    echo("✅ Docker configuration created");
}

fn create_sample_application(framework: &str, database: &str, async_support: bool) {
    echo("📝 Creating sample application code...");
    
    match framework {
        "django" => create_django_app(database),
        "flask" => create_flask_app(database, async_support),
        "fastapi" => create_fastapi_app(database),
        "none" => create_cli_app(),
        _ => panic!("Unsupported framework: {}", framework),
    }
}

fn create_fastapi_app(database: &str) {
    let main_app = format!(r#"""FastAPI application main module."""

from fastapi import FastAPI, HTTPException, Depends
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded
import uvicorn
import time
import os
from contextlib import asynccontextmanager
{}

# Rate limiting
limiter = Limiter(key_func=get_remote_address)

@asynccontextmanager
async def lifespan(app: FastAPI):
    \"\"\"Application lifespan events.\"\"\"
    # Startup
    print("🚀 Application starting up...")
    {}
    yield
    # Shutdown
    print("🛑 Application shutting down...")
    {}

# Create FastAPI app
app = FastAPI(
    title="My Python API",
    description="A FastAPI application created with Python bootstrap installer",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan
)

# Add middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.add_middleware(
    TrustedHostMiddleware,
    allowed_hosts=["localhost", "127.0.0.1", "*.example.com"]
)

# Add rate limiting
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# Health check endpoint
@app.get("/health")
@limiter.limit("10/minute")
async def health_check(request):
    \"\"\"Health check endpoint.\"\"\"
    return {{
        "status": "OK",
        "timestamp": time.time(),
        "uptime": time.time() - start_time
    }}

# Root endpoint
@app.get("/")
@limiter.limit("30/minute")
async def root(request):
    \"\"\"Root endpoint.\"\"\"
    return {{
        "message": "Welcome to the FastAPI application",
        "version": "1.0.0",
        "docs": "/docs"
    }}

# API routes
from src.api.routes import router as api_router
app.include_router(api_router, prefix="/api/v1", tags=["api"])

# Global exception handler
@app.exception_handler(Exception)
async def global_exception_handler(request, exc):
    \"\"\"Global exception handler.\"\"\"
    return HTTPException(
        status_code=500,
        detail="Internal server error"
    )

def get_health_status():
    \"\"\"Get application health status.\"\"\"
    return {{
        "status": "OK",
        "timestamp": time.time(),
        "uptime": time.time() - start_time
    }}

# Application startup time
start_time = time.time()

if __name__ == "__main__":
    uvicorn.run(
        "src.main:app",
        host="0.0.0.0",
        port=int(os.getenv("PORT", 8000)),
        reload=os.getenv("DEBUG", "false").lower() == "true",
        workers=1 if os.getenv("DEBUG") else 4,
        log_level="info"
    )
"#, 
    if database != "none" { "from src.database.connection import db" } else { "" },
    if database != "none" { "await db.connect()" } else { "pass" },
    if database != "none" { "await db.disconnect()" } else { "pass" }
);
    
    write_file("src/main.py", &main_app);
    
    // Create API routes
    let api_routes = r#"""API routes module."""

from fastapi import APIRouter, HTTPException, Depends
from pydantic import BaseModel
from typing import List, Optional
import time

router = APIRouter()

# Pydantic models
class User(BaseModel):
    id: Optional[int] = None
    name: str
    email: str
    created_at: Optional[float] = None

class UserCreate(BaseModel):
    name: str
    email: str

class UserResponse(BaseModel):
    id: int
    name: str
    email: str
    created_at: float

# In-memory storage (replace with database)
users_db = []
next_id = 1

@router.get("/users", response_model=List[UserResponse])
async def get_users():
    """Get all users."""
    return users_db

@router.post("/users", response_model=UserResponse)
async def create_user(user: UserCreate):
    """Create a new user."""
    global next_id
    
    # Check if email already exists
    for existing_user in users_db:
        if existing_user["email"] == user.email:
            raise HTTPException(status_code=400, detail="Email already exists")
    
    new_user = {
        "id": next_id,
        "name": user.name,
        "email": user.email,
        "created_at": time.time()
    }
    
    users_db.append(new_user)
    next_id += 1
    
    return new_user

@router.get("/users/{user_id}", response_model=UserResponse)
async def get_user(user_id: int):
    """Get a user by ID."""
    for user in users_db:
        if user["id"] == user_id:
            return user
    
    raise HTTPException(status_code=404, detail="User not found")

@router.put("/users/{user_id}", response_model=UserResponse)
async def update_user(user_id: int, user_update: UserCreate):
    """Update a user."""
    for i, user in enumerate(users_db):
        if user["id"] == user_id:
            users_db[i].update({
                "name": user_update.name,
                "email": user_update.email
            })
            return users_db[i]
    
    raise HTTPException(status_code=404, detail="User not found")

@router.delete("/users/{user_id}")
async def delete_user(user_id: int):
    """Delete a user."""
    for i, user in enumerate(users_db):
        if user["id"] == user_id:
            deleted_user = users_db.pop(i)
            return {"message": f"User {deleted_user['name']} deleted successfully"}
    
    raise HTTPException(status_code=404, detail="User not found")
"#;
    
    write_file("src/api/routes/__init__.py", api_routes);
}

fn create_flask_app(database: &str, async_support: bool) {
    let main_app = format!(r#"""Flask application main module."""

from flask import Flask, jsonify, request
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
import time
import os
{}

def create_app():
    \"\"\"Application factory pattern.\"\"\"
    app = Flask(__name__)
    
    # Configuration
    app.config['SECRET_KEY'] = os.getenv('SECRET_KEY', 'dev-secret-key')
    app.config['DEBUG'] = os.getenv('DEBUG', 'false').lower() == 'true'
    
    # Extensions
    CORS(app)
    
    # Rate limiting
    limiter = Limiter(
        app,
        key_func=get_remote_address,
        default_limits=["100 per hour"]
    )
    
    {}
    
    # Routes
    @app.route('/')
    @limiter.limit("30 per minute")
    def root():
        \"\"\"Root endpoint.\"\"\"
        return jsonify({{
            "message": "Welcome to the Flask application",
            "version": "1.0.0"
        }})
    
    @app.route('/health')
    @limiter.limit("10 per minute")  
    def health():
        \"\"\"Health check endpoint.\"\"\"
        return jsonify(get_health_status())
    
    # API Blueprint
    from src.api.routes import api_bp
    app.register_blueprint(api_bp, url_prefix='/api/v1')
    
    # Error handlers
    @app.errorhandler(404)
    def not_found(error):
        return jsonify({{"error": "Not found"}}), 404
    
    @app.errorhandler(500)
    def internal_error(error):
        return jsonify({{"error": "Internal server error"}}), 500
    
    return app

def get_health_status():
    \"\"\"Get application health status.\"\"\"
    return {{
        "status": "OK",
        "timestamp": time.time(),
        "uptime": time.time() - start_time
    }}

# Application startup time
start_time = time.time()

# Create app instance
app = create_app()

if __name__ == "__main__":
    app.run(
        host="0.0.0.0",
        port=int(os.getenv("PORT", 5000)),
        debug=os.getenv("DEBUG", "false").lower() == "true"
    )
"#,
    if database != "none" { "from src.database.connection import db" } else { "" },
    if database != "none" { "# Initialize database\n    db.connect()" } else { "pass" }
);
    
    write_file("src/main.py", &main_app);
    
    // Create Flask API routes
    let api_routes = r#"""Flask API routes."""

from flask import Blueprint, jsonify, request
import time

api_bp = Blueprint('api', __name__)

# In-memory storage (replace with database)
users_db = []
next_id = 1

@api_bp.route('/users', methods=['GET'])
def get_users():
    """Get all users."""
    return jsonify(users_db)

@api_bp.route('/users', methods=['POST'])
def create_user():
    """Create a new user."""
    global next_id
    
    data = request.get_json()
    
    if not data or 'name' not in data or 'email' not in data:
        return jsonify({"error": "Name and email are required"}), 400
    
    # Check if email already exists
    for user in users_db:
        if user['email'] == data['email']:
            return jsonify({"error": "Email already exists"}), 400
    
    new_user = {
        "id": next_id,
        "name": data['name'],
        "email": data['email'],
        "created_at": time.time()
    }
    
    users_db.append(new_user)
    next_id += 1
    
    return jsonify(new_user), 201

@api_bp.route('/users/<int:user_id>', methods=['GET'])
def get_user(user_id):
    """Get a user by ID."""
    for user in users_db:
        if user['id'] == user_id:
            return jsonify(user)
    
    return jsonify({"error": "User not found"}), 404

@api_bp.route('/users/<int:user_id>', methods=['PUT'])
def update_user(user_id):
    """Update a user."""
    data = request.get_json()
    
    for i, user in enumerate(users_db):
        if user['id'] == user_id:
            if 'name' in data:
                users_db[i]['name'] = data['name']
            if 'email' in data:
                users_db[i]['email'] = data['email']
            return jsonify(users_db[i])
    
    return jsonify({"error": "User not found"}), 404

@api_bp.route('/users/<int:user_id>', methods=['DELETE'])
def delete_user(user_id):
    """Delete a user."""
    for i, user in enumerate(users_db):
        if user['id'] == user_id:
            deleted_user = users_db.pop(i)
            return jsonify({"message": f"User {deleted_user['name']} deleted successfully"})
    
    return jsonify({"error": "User not found"}), 404
"#;
    
    write_file("src/api/routes.py", api_routes);
}

fn create_cli_app() {
    let cli_app = r#"""Command-line application main module."""

import click
import time
import os
from typing import Optional

@click.group()
@click.version_option(version="1.0.0")
@click.option('--verbose', '-v', is_flag=True, help='Enable verbose output')
@click.pass_context
def cli(ctx, verbose):
    """My Python CLI Application."""
    ctx.ensure_object(dict)
    ctx.obj['verbose'] = verbose
    
    if verbose:
        click.echo("🐍 Python CLI Application starting...")

@cli.command()
@click.option('--name', '-n', default='World', help='Name to greet')
@click.option('--count', '-c', default=1, help='Number of greetings')
@click.pass_context
def hello(ctx, name, count):
    """Say hello to NAME."""
    if ctx.obj['verbose']:
        click.echo(f"Greeting {name} {count} time(s)")
    
    for _ in range(count):
        click.echo(f"Hello, {name}!")

@cli.command()
@click.pass_context
def status(ctx):
    """Show application status."""
    if ctx.obj['verbose']:
        click.echo("Checking application status...")
    
    status_info = get_health_status()
    
    click.echo("📊 Application Status:")
    click.echo(f"  Status: {status_info['status']}")
    click.echo(f"  Uptime: {status_info['uptime']:.2f} seconds")
    click.echo(f"  Timestamp: {status_info['timestamp']}")

@cli.command()
@click.argument('filename', type=click.Path(exists=True))
@click.option('--output', '-o', type=click.Path(), help='Output file')
@click.pass_context
def process(ctx, filename, output):
    """Process a file."""
    if ctx.obj['verbose']:
        click.echo(f"Processing file: {filename}")
    
    # Simulate file processing
    with open(filename, 'r') as f:
        content = f.read()
    
    # Simple processing: count lines and words
    lines = len(content.splitlines())
    words = len(content.split())
    
    result = f"File: {filename}\nLines: {lines}\nWords: {words}\n"
    
    if output:
        with open(output, 'w') as f:
            f.write(result)
        click.echo(f"Results written to: {output}")
    else:
        click.echo(result)

def get_health_status():
    """Get application health status."""
    return {
        "status": "OK",
        "timestamp": time.time(),
        "uptime": time.time() - start_time
    }

def main():
    """Main entry point."""
    cli(obj={})

# Application startup time
start_time = time.time()

if __name__ == "__main__":
    main()
"#;
    
    write_file("src/main.py", cli_app);
}

fn setup_environment_configuration() {
    echo("🔧 Setting up environment configuration...");
    
    let env_example = r#"# Application Configuration
DEBUG=false
SECRET_KEY=your-super-secret-key-here
PORT=8000

# CORS Configuration
CORS_ORIGINS=http://localhost:3000,http://localhost:8080

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=3600

# Logging Configuration
LOG_LEVEL=INFO
LOG_FILE=./logs/app.log
LOG_FORMAT=%(asctime)s - %(name)s - %(levelname)s - %(message)s

# Security Configuration - Cookie protection settings
SESSION_COOKIE_SECURE=true
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Lax

# API Configuration
API_TITLE=My Python API
API_VERSION=1.0.0
API_DESCRIPTION=A Python API application

# Worker Configuration
WORKERS=4
WORKER_CLASS=uvicorn.workers.UvicornWorker
WORKER_TIMEOUT=30
KEEP_ALIVE=2

# Cache Configuration
CACHE_TYPE=redis
CACHE_REDIS_URL=redis://localhost:6379/0
CACHE_DEFAULT_TIMEOUT=300
"#;
    
    write_file(".env.example", env_example);
    
    // Create .gitignore
    let gitignore = r#"# Byte-compiled / optimized / DLL files
__pycache__/
*.py[cod]
*$py.class

# C extensions
*.so

# Distribution / packaging
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg
MANIFEST

# PyInstaller
*.manifest
*.spec

# Installer logs
pip-log.txt
pip-delete-this-directory.txt

# Unit test / coverage reports
htmlcov/
.tox/
.nox/
.coverage
.coverage.*
.cache
nosetests.xml
coverage.xml
*.cover
.hypothesis/
.pytest_cache/

# Translations
*.mo
*.pot

# Django stuff:
*.log
local_settings.py
db.sqlite3

# Flask stuff:
instance/
.webassets-cache

# Scrapy stuff:
.scrapy

# Sphinx documentation
docs/_build/

# PyBuilder
target/

# Jupyter Notebook
.ipynb_checkpoints

# IPython
profile_default/
ipython_config.py

# pyenv
.python-version

# celery beat schedule file
celerybeat-schedule

# SageMath parsed files
*.sage.py

# Environments
.env
.env.*
!.env.example
.venv
venv/
ENV/
env/
env.bak/
venv.bak/

# Spyder project settings
.spyderproject
.spyproject

# Rope project settings
.ropeproject

# mkdocs documentation
/site

# mypy
.mypy_cache/
.dmypy.json
dmypy.json

# Pyre type checker
.pyre/

# IDEs
.vscode/
.idea/
*.swp
*.swo
*~

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Database
*.db
*.sqlite
*.sqlite3
data/

# Logs
logs/
*.log

# Temporary files
tmp/
temp/
"#;
    
    write_file(".gitignore", gitignore);
    
    echo("✅ Environment configuration created");
}

fn install_dependencies(manager: &str) {
    echo(&format!("📦 Installing all dependencies with {}...", manager));
    
    match manager {
        "pip" => {
            exec("source venv/bin/activate && pip install -r requirements.txt");
            exec("source venv/bin/activate && pip install -r requirements-dev.txt");
        },
        "poetry" => {
            exec("poetry install");
        },
        "pipenv" => {
            exec("pipenv install --dev");
        },
        _ => panic!("Unknown package manager: {}", manager),
    }
    
    echo("✅ Dependencies installed successfully");
}

fn create_project_documentation(project_name: &str, framework: &str, database: &str) {
    echo("📚 Creating project documentation...");
    
    let readme = format!(r#"# {}

A {} application built with Python{}

## Features

- 🐍 Python 3.8+ support
- 🚀 {} web framework
- 🧪 Testing with pytest
- 🔒 Security best practices
- 🐳 Docker support
- 🔄 CI/CD with GitHub Actions
- 📝 Code quality with Black, isort, flake8, mypy
- 🎯 Environment-based configuration
{}

## Prerequisites

- Python 3.8 or higher
- pip or poetry

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd {}
```

2. Create and activate virtual environment:
```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

3. Install dependencies:
```bash
pip install -r requirements.txt
pip install -r requirements-dev.txt
```

4. Copy environment variables:
```bash
cp .env.example .env
```

5. Update the `.env` file with your configuration

## Development

Start the development server:
```bash
python -m src.main
```

## Testing

Run tests:
```bash
pytest
```

Run tests with coverage:
```bash
pytest --cov=src
```

## Code Quality

Format code:
```bash
black src tests
isort src tests
```

Lint code:
```bash
flake8 src tests
mypy src
```

Run all quality checks:
```bash
pre-commit run --all-files
```

## Production

### Using Docker

Build and run with Docker:
```bash
docker build -t {} .
docker run -p 8000:8000 {}
```

Or use Docker Compose:
```bash
docker-compose up
```

### Manual Deployment

1. Install production dependencies:
```bash
pip install -r requirements.txt
```

2. Set production environment variables

3. Run with a production WSGI server:
```bash
gunicorn src.main:app --workers 4 --bind 0.0.0.0:8000
```

## API Documentation

### Health Check
- **GET** `/health` - Returns application health status

### API Endpoints
- **GET** `/api/v1/users` - Get all users
- **POST** `/api/v1/users` - Create a new user
- **GET** `/api/v1/users/{{id}}` - Get user by ID
- **PUT** `/api/v1/users/{{id}}` - Update user by ID
- **DELETE** `/api/v1/users/{{id}}` - Delete user by ID

## Project Structure

```
{}
├── src/
│   ├── api/
│   │   ├── routes/         # API route handlers
│   │   └── middleware/     # Custom middleware
│   ├── models/            # Data models
│   ├── services/          # Business logic
│   ├── utils/             # Utility functions
│   ├── config/            # Configuration files
│   ├── database/          # Database connections and migrations
│   └── main.py            # Application entry point
├── tests/
│   ├── unit/              # Unit tests
│   ├── integration/       # Integration tests
│   ├── e2e/              # End-to-end tests
│   └── fixtures/         # Test fixtures
├── static/               # Static files
├── templates/            # Template files
├── scripts/              # Utility scripts
├── docs/                 # Documentation
├── requirements.txt      # Production dependencies
├── requirements-dev.txt  # Development dependencies
├── Dockerfile           # Docker configuration
├── docker-compose.yml   # Docker Compose configuration
└── pyproject.toml       # Project configuration
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and quality checks (`pre-commit run --all-files`)
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
"#,
        project_name,
        framework,
        if database != "none" { &format!(" and {} database", database) } else { "" },
        framework,
        if database != "none" { &format!("- 🗄️ {} database integration", database) } else { "" },
        project_name,
        project_name,
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

### Added
- Initial project setup
- Basic API structure
- Testing framework
- Docker configuration
- CI/CD pipeline
- Code quality tools
- Documentation

## [1.0.0] - 2024-01-01

### Added
- Initial release
- Core application functionality
- API endpoints
- Database integration
- Testing suite
- Docker support

[Unreleased]: https://github.com/username/project/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/username/project/releases/tag/v1.0.0
"#;
    
    write_file("CHANGELOG.md", changelog);
    
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