// Python project bootstrap installer
// Demonstrates setting up a complete Python development environment

mod components;
use components::*;

#[bashrs::main]
fn python_project_bootstrap() {
    let config = ProjectConfig::from_env();
    
    echo("🐍 Python Project Bootstrap Installer");
    echo(&format!("Project: {}, Python: {}, Framework: {}", 
        config.project_name, config.python_version, config.framework));
    
    // Install Python environment
    let installer = PythonInstaller::new(&config);
    installer.install();
    
    // Setup project structure
    let project = ProjectSetup::new(&config);
    project.create_structure();
    project.setup_environment();
    project.initialize_config();
    
    // Install dependencies and tools
    let deps = DependencyManager::new(&config);
    deps.install_framework();
    deps.install_database();
    deps.install_testing();
    deps.install_dev_tools();
    deps.install_quality_tools();
    
    // Generate project files
    let generator = FileGenerator::new(&config);
    generator.create_app();
    generator.create_configs();
    generator.create_documentation();
    
    // Final setup
    deps.install_all();
    
    echo("✅ Python project bootstrap completed successfully");
    echo(&format!("cd {} && source venv/bin/activate && python -m src.main", 
        config.project_name));
}

// Moved all the implementation to separate modules for better organization
mod components {
    use std::env;
    
    #[derive(Clone)]
    pub struct ProjectConfig {
        pub project_name: String,
        pub python_version: String,
        pub package_manager: String,
        pub framework: String,
        pub database: String,
        pub testing_framework: String,
        pub async_support: bool,
    }
    
    impl ProjectConfig {
        pub fn from_env() -> Self {
            Self {
                project_name: env_var_or("PROJECT_NAME", "my-python-app"),
                python_version: env_var_or("PYTHON_VERSION", "3.11"),
                package_manager: env_var_or("PACKAGE_MANAGER", "pip"),
                framework: env_var_or("FRAMEWORK", "fastapi"),
                database: env_var_or("DATABASE", "none"),
                testing_framework: env_var_or("TESTING", "pytest"),
                async_support: env_var_or("ASYNC", "true") == "true",
            }
        }
    }
    
    pub struct PythonInstaller<'a> {
        config: &'a ProjectConfig,
    }
    
    impl<'a> PythonInstaller<'a> {
        pub fn new(config: &'a ProjectConfig) -> Self {
            Self { config }
        }
        
        pub fn install(&self) {
            let os_info = self.detect_os();
            self.install_pyenv(&os_info);
            self.install_package_manager();
        }
        
        fn detect_os(&self) -> String {
            if path_exists("/etc/os-release") {
                let content = read_file("/etc/os-release");
                if content.contains("Ubuntu") { return "ubuntu".to_string(); }
                if content.contains("Debian") { return "debian".to_string(); }
                if content.contains("CentOS") || content.contains("Red Hat") { 
                    return "rhel".to_string(); 
                }
            }
            if command_exists("sw_vers") { return "macos".to_string(); }
            "linux".to_string()
        }
        
        fn install_pyenv(&self, os: &str) {
            echo(&format!("🔧 Installing Python {} via pyenv...", self.config.python_version));
            
            self.install_dependencies(os);
            
            if !command_exists("pyenv") {
                exec("curl https://pyenv.run | bash");
                self.configure_shell();
            }
            
            exec(&format!("pyenv install {}", self.config.python_version));
            exec(&format!("pyenv global {}", self.config.python_version));
            exec("python --version");
            exec("pip --version");
            
            echo("✅ Python installed successfully");
        }
        
        fn install_dependencies(&self, os: &str) {
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
                    if command_exists("brew") {
                        exec("brew install openssl readline sqlite3 xz zlib tcl-tk");
                    }
                },
                _ => echo("Warning: Unknown OS, skipping dependency installation"),
            }
        }
        
        fn configure_shell(&self) {
            let shell_rc = if path_exists(&format!("{}/.zshrc", env_var("HOME"))) {
                "~/.zshrc"
            } else {
                "~/.bashrc"
            };
            
            let config = r#"
# Pyenv configuration
export PYENV_ROOT="$HOME/.pyenv"
command -v pyenv >/dev/null || export PATH="$PYENV_ROOT/bin:$PATH"
eval "$(pyenv init -)"
"#;
            
            append_to_file(shell_rc, config);
            exec(&format!("source {}", shell_rc));
        }
        
        fn install_package_manager(&self) {
            echo(&format!("📦 Setting up package manager: {}", self.config.package_manager));
            
            match self.config.package_manager.as_str() {
                "pip" => exec("python -m pip install --upgrade pip"),
                "poetry" => {
                    exec("curl -sSL https://install.python-poetry.org | python3 -");
                    let poetry_path = format!("{}/.local/bin", env_var("HOME"));
                    exec(&format!("export PATH={}:$PATH", poetry_path));
                    exec("poetry --version");
                },
                "pipenv" => {
                    exec("python -m pip install pipenv");
                    exec("pipenv --version");
                },
                _ => panic!("Unsupported package manager: {}", self.config.package_manager),
            }
            
            echo(&format!("✅ {} installed successfully", self.config.package_manager));
        }
    }
    
    pub struct ProjectSetup<'a> {
        config: &'a ProjectConfig,
    }
    
    impl<'a> ProjectSetup<'a> {
        pub fn new(config: &'a ProjectConfig) -> Self {
            Self { config }
        }
        
        pub fn create_structure(&self) {
            echo(&format!("📁 Creating project structure for {}...", self.config.project_name));
            
            mkdir_p(&self.config.project_name);
            cd(&self.config.project_name);
            
            let dirs = vec![
                "src", "src/config", "src/models", "src/services", "src/utils",
                "src/api", "src/api/routes", "src/api/middleware",
                "src/database", "src/database/migrations",
                "tests", "tests/unit", "tests/integration", "tests/e2e", "tests/fixtures",
                "docs", "scripts", "static", "static/css", "static/js", "static/images",
                "templates", "logs", "data",
            ];
            
            for dir in dirs {
                mkdir_p(dir);
                if dir.starts_with("src") || dir.starts_with("tests") {
                    touch(&format!("{}/__init__.py", dir));
                }
            }
            
            let files = vec![
                ".env", ".env.example", ".gitignore", "README.md",
                "CHANGELOG.md", "LICENSE", "requirements.txt", "requirements-dev.txt",
            ];
            
            for file in files {
                touch(file);
            }
            
            echo("✅ Project structure created");
        }
        
        pub fn setup_environment(&self) {
            echo("🏠 Setting up virtual environment...");
            
            match self.config.package_manager.as_str() {
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
                _ => panic!("Unsupported package manager"),
            }
            
            echo("✅ Virtual environment created");
        }
        
        pub fn initialize_config(&self) {
            echo("📋 Initializing project configuration...");
            
            match self.config.package_manager.as_str() {
                "pip" => self.create_setup_py(),
                "poetry" => self.create_pyproject_toml(),
                "pipenv" => self.create_pipfile(),
                _ => {},
            }
            
            echo("✅ Project configuration initialized");
        }
        
        fn create_setup_py(&self) {
            let content = format!(
                include_str!("templates/setup.py.template"),
                name = self.config.project_name,
                framework = self.config.framework,
                testing = self.config.testing_framework,
            );
            write_file("setup.py", &content);
        }
        
        fn create_pyproject_toml(&self) {
            let content = format!(
                include_str!("templates/pyproject.toml.template"),
                name = self.config.project_name,
                framework = self.config.framework,
                testing = self.config.testing_framework,
            );
            write_file("pyproject.toml", &content);
        }
        
        fn create_pipfile(&self) {
            let content = format!(
                include_str!("templates/Pipfile.template"),
                testing = self.config.testing_framework,
            );
            write_file("Pipfile", &content);
        }
    }
    
    include!("python_project_bootstrap_deps.rs");






include!("python_project_bootstrap_cont.rs");
