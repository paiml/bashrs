fn generate_dockerfile_templates(count: usize, seed: u64) -> Vec<String> {
    let mut scripts = Vec::with_capacity(count);
    let mut idx = seed;

    let base_images = [
        "alpine:3.18",
        "ubuntu:22.04",
        "debian:bookworm-slim",
        "fedora:39",
        "python:3.12-slim",
        "node:20-alpine",
        "golang:1.22-alpine",
        "rust:1.77-slim",
        "ruby:3.3-slim",
        "openjdk:21-slim",
    ];
    let safe_run = [
        "RUN apk add --no-cache curl git",
        "RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*",
        "RUN pip install --no-cache-dir flask gunicorn",
        "RUN npm ci --production",
        "RUN go build -o /app ./cmd/server",
        "RUN cargo build --release",
        "RUN adduser -D -s /bin/sh appuser",
        "RUN chmod 755 /app/entrypoint.sh",
    ];
    let safe_copy = [
        "COPY --chown=appuser:appuser . /app/",
        "COPY requirements.txt /app/",
        "COPY package.json package-lock.json /app/",
        "COPY go.mod go.sum /app/",
        "COPY Cargo.toml Cargo.lock /app/",
    ];
    let safe_workdir = ["WORKDIR /app", "WORKDIR /opt/app", "WORKDIR /home/appuser"];
    let safe_expose = ["EXPOSE 8080", "EXPOSE 3000", "EXPOSE 5000", "EXPOSE 80"];
    let safe_entrypoint = [
        "ENTRYPOINT [\"/app/server\"]",
        "ENTRYPOINT [\"python\", \"-m\", \"flask\", \"run\"]",
        "ENTRYPOINT [\"node\", \"server.js\"]",
    ];
    let safe_cmd = [
        "CMD [\"--port\", \"8080\"]",
        "CMD [\"serve\", \"--host\", \"0.0.0.0\"]",
        "CMD [\"run\"]",
    ];
    let safe_user = ["USER appuser", "USER nobody", "USER 1000:1000"];
    let safe_env = [
        "ENV APP_ENV=production",
        "ENV NODE_ENV=production",
        "ENV PYTHONUNBUFFERED=1",
        "ENV LANG=C.UTF-8",
    ];
    let safe_label = [
        "LABEL maintainer=\"team@example.com\"",
        "LABEL version=\"1.0.0\"",
        "LABEL description=\"Production service\"",
    ];

    // Unsafe patterns
    let unsafe_run_root = [
        "RUN chmod 777 /app",
        "RUN chmod -R 777 /etc",
        "RUN chmod 666 /etc/shadow",
    ];
    let unsafe_run_curl = [
        "RUN curl https://example.com/setup.sh | sh",
        "RUN wget -O- https://example.com/install | bash",
        "RUN curl -sSL https://get.example.com | sh -s --",
    ];
    let unsafe_run_eval = ["RUN eval $BUILD_CMD", "RUN sh -c \"eval $SCRIPT\""];
    let unsafe_env = [
        "ENV DB_PASSWORD=secret123",
        "ENV API_KEY=sk-1234567890",
        "ENV AWS_SECRET_KEY=wJalrXUtnFEMI",
    ];
    let unsafe_add = ["ADD https://example.com/file.tar.gz /app/", "ADD . /app/"];

    while scripts.len() < count {
        idx = idx
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let variant = (idx >> 33) as usize;

        let script = match variant % 14 {
            // Safe: minimal Dockerfile
            0 => {
                let base = base_images[variant % base_images.len()];
                let run = safe_run[variant % safe_run.len()];
                let copy = safe_copy[variant % safe_copy.len()];
                format!(
                    "FROM {base}\n{run}\n{copy}\n{}",
                    safe_cmd[variant % safe_cmd.len()]
                )
            }
            // Safe: multi-stage build
            1 => {
                let base = base_images[variant % base_images.len()];
                let run_base = base_images[(variant + 1) % base_images.len()];
                format!(
                    "FROM {base} AS builder\n{}\nCOPY . /src/\nRUN cargo build --release\n\nFROM {run_base}\nCOPY --from=builder /src/target/release/app /app/\n{}",
                    safe_workdir[variant % safe_workdir.len()],
                    safe_entrypoint[variant % safe_entrypoint.len()]
                )
            }
            // Safe: with user + expose
            2 => {
                let base = base_images[variant % base_images.len()];
                format!(
                    "FROM {base}\n{}\n{}\n{}\n{}\n{}",
                    safe_run[variant % safe_run.len()],
                    safe_workdir[variant % safe_workdir.len()],
                    safe_user[variant % safe_user.len()],
                    safe_expose[variant % safe_expose.len()],
                    safe_entrypoint[variant % safe_entrypoint.len()]
                )
            }
            // Safe: with env + label
            3 => {
                let base = base_images[variant % base_images.len()];
                format!(
                    "FROM {base}\n{}\n{}\n{}\n{}",
                    safe_label[variant % safe_label.len()],
                    safe_env[variant % safe_env.len()],
                    safe_copy[variant % safe_copy.len()],
                    safe_cmd[variant % safe_cmd.len()]
                )
            }
            // Safe: healthcheck
            4 => {
                let base = base_images[variant % base_images.len()];
                let port = ["8080", "3000", "5000"][variant % 3];
                format!(
                    "FROM {base}\n{}\nHEALTHCHECK --interval=30s --timeout=3s CMD curl -f http://localhost:{port}/health || exit 1\n{}",
                    safe_run[variant % safe_run.len()],
                    safe_expose[variant % safe_expose.len()]
                )
            }
            // Safe: arg + env combo
            5 => {
                let base = base_images[variant % base_images.len()];
                let arg = ["APP_VERSION", "BUILD_DATE", "GIT_SHA"][variant % 3];
                format!(
                    "FROM {base}\nARG {arg}=unknown\n{}\n{}\n{}",
                    safe_env[variant % safe_env.len()],
                    safe_workdir[variant % safe_workdir.len()],
                    safe_entrypoint[variant % safe_entrypoint.len()]
                )
            }
            // Safe: volume + copy
            6 => {
                let base = base_images[variant % base_images.len()];
                let vol = ["/data", "/app/logs", "/var/lib/app"][variant % 3];
                format!(
                    "FROM {base}\nVOLUME [\"{vol}\"]\n{}\n{}",
                    safe_copy[variant % safe_copy.len()],
                    safe_cmd[variant % safe_cmd.len()]
                )
            }
            // Safe: onbuild
            7 => {
                let base = base_images[variant % base_images.len()];
                format!(
                    "FROM {base}\nONBUILD COPY . /app/\nONBUILD RUN pip install -r requirements.txt"
                )
            }
            // Unsafe: curl | sh
            8 => {
                let base = base_images[variant % base_images.len()];
                let curl_cmd = unsafe_run_curl[variant % unsafe_run_curl.len()];
                format!("FROM {base}\n{curl_cmd}")
            }
            // Unsafe: chmod 777
            9 => {
                let base = base_images[variant % base_images.len()];
                let chmod = unsafe_run_root[variant % unsafe_run_root.len()];
                format!(
                    "FROM {base}\n{}\n{chmod}",
                    safe_copy[variant % safe_copy.len()]
                )
            }
            // Unsafe: secrets in ENV
            10 => {
                let base = base_images[variant % base_images.len()];
                let env = unsafe_env[variant % unsafe_env.len()];
                format!("FROM {base}\n{env}\n{}", safe_cmd[variant % safe_cmd.len()])
            }
            // Unsafe: eval
            11 => {
                let base = base_images[variant % base_images.len()];
                let eval_cmd = unsafe_run_eval[variant % unsafe_run_eval.len()];
                format!("FROM {base}\n{eval_cmd}")
            }
            // Unsafe: ADD remote URL
            12 => {
                let base = base_images[variant % base_images.len()];
                let add = unsafe_add[variant % unsafe_add.len()];
                format!("FROM {base}\n{add}")
            }
            // Unsafe: running as root (no USER)
            _ => {
                let base = base_images[variant % base_images.len()];
                format!(
                    "FROM {base}\n{}\nRUN apt-get update && apt-get install -y sudo\nENTRYPOINT [\"bash\"]",
                    safe_run[variant % safe_run.len()]
                )
            }
        };

        scripts.push(script);
    }

    scripts
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "expansion_generator_tests_pmat176_gene.rs"]
// FIXME(PMAT-238): mod tests_extracted;
