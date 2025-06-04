// Kubernetes cluster bootstrap installer
// Demonstrates setting up a production-ready Kubernetes cluster

#[rash::main]
fn kubernetes_cluster_bootstrap() {
    let cluster_name = env_var_or("CLUSTER_NAME", "production-k8s");
    let kubernetes_version = env_var_or("K8S_VERSION", "1.28.0");
    let container_runtime = env_var_or("CONTAINER_RUNTIME", "containerd");
    let cni_plugin = env_var_or("CNI_PLUGIN", "calico");
    
    echo("🚀 Kubernetes Cluster Bootstrap Installer");
    echo(&format!("Cluster: {}, Version: {}", cluster_name, kubernetes_version));
    
    // Detect operating system and architecture
    let os_info = detect_operating_system();
    let arch = detect_architecture();
    
    // Validate system requirements
    validate_system_requirements(&os_info, &arch);
    
    // Install container runtime
    install_container_runtime(&container_runtime, &os_info);
    
    // Install kubeadm, kubelet, kubectl
    install_kubernetes_tools(&kubernetes_version, &os_info, &arch);
    
    // Configure system for Kubernetes
    configure_system_for_kubernetes();
    
    // Initialize control plane
    initialize_control_plane(&cluster_name, &kubernetes_version, &cni_plugin);
    
    // Install CNI plugin
    install_cni_plugin(&cni_plugin);
    
    // Install essential cluster components
    install_cluster_components();
    
    // Configure kubectl for current user
    configure_kubectl_access();
    
    // Verify cluster installation
    verify_cluster_installation();
    
    echo("✅ Kubernetes cluster bootstrap completed successfully");
    echo("Run 'kubectl get nodes' to verify your cluster");
}

fn detect_operating_system() -> String {
    if path_exists("/etc/os-release") {
        let os_release = read_file("/etc/os-release");
        if os_release.contains("Ubuntu") {
            return "ubuntu".to_string();
        } else if os_release.contains("CentOS") || os_release.contains("Red Hat") {
            return "rhel".to_string();
        } else if os_release.contains("Debian") {
            return "debian".to_string();
        } else if os_release.contains("SUSE") {
            return "suse".to_string();
        }
    }
    
    if command_exists("sw_vers") {
        return "macos".to_string();
    }
    
    "unknown".to_string()
}

fn detect_architecture() -> String {
    let arch_output = exec_output("uname -m");
    match arch_output.trim() {
        "x86_64" => "amd64".to_string(),
        "aarch64" | "arm64" => "arm64".to_string(),
        "armv7l" => "arm".to_string(),
        _ => arch_output.trim().to_string(),
    }
}

fn validate_system_requirements(os: &str, arch: &str) {
    echo("🔍 Validating system requirements...");
    
    // Check minimum memory (2GB)
    let memory_kb = exec_output("grep MemTotal /proc/meminfo | awk '{print $2}'");
    let memory_gb = memory_kb.trim().parse::<u64>().unwrap_or(0) / 1024 / 1024;
    
    if memory_gb < 2 {
        panic!("Insufficient memory: {}GB found, 2GB minimum required", memory_gb);
    }
    
    // Check disk space (20GB)
    let disk_space = exec_output("df / | tail -1 | awk '{print $4}'");
    let disk_gb = disk_space.trim().parse::<u64>().unwrap_or(0) / 1024 / 1024;
    
    if disk_gb < 20 {
        panic!("Insufficient disk space: {}GB available, 20GB minimum required", disk_gb);
    }
    
    // Check CPU cores (2 minimum)
    let cpu_cores = exec_output("nproc");
    let cores = cpu_cores.trim().parse::<u32>().unwrap_or(0);
    
    if cores < 2 {
        panic!("Insufficient CPU cores: {} found, 2 minimum required", cores);
    }
    
    // Check supported architecture
    if !["amd64", "arm64"].contains(&arch) {
        panic!("Unsupported architecture: {}", arch);
    }
    
    echo(&format!("✅ System requirements validated: {}GB RAM, {}GB disk, {} CPU cores", memory_gb, disk_gb, cores));
}

fn install_container_runtime(runtime: &str, os: &str) {
    echo(&format!("📦 Installing container runtime: {}", runtime));
    
    match runtime {
        "containerd" => install_containerd(os),
        "docker" => install_docker(os),
        "cri-o" => install_crio(os),
        _ => panic!("Unsupported container runtime: {}", runtime),
    }
}

fn install_containerd(os: &str) {
    match os {
        "ubuntu" | "debian" => {
            // Update package index
            exec("apt-get update");
            
            // Install prerequisites
            exec("apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release");
            
            // Add Docker repository (for containerd)
            exec("curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg");
            echo("deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable") > "/etc/apt/sources.list.d/docker.list";
            
            exec("apt-get update");
            exec("apt-get install -y containerd.io");
            
            // Configure containerd
            exec("mkdir -p /etc/containerd");
            exec("containerd config default > /etc/containerd/config.toml");
            
            // Enable systemd cgroup driver
            exec("sed -i 's/SystemdCgroup = false/SystemdCgroup = true/' /etc/containerd/config.toml");
        },
        "rhel" => {
            // Install containerd on RHEL/CentOS
            exec("yum install -y yum-utils");
            exec("yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo");
            exec("yum install -y containerd.io");
            
            exec("mkdir -p /etc/containerd");
            exec("containerd config default > /etc/containerd/config.toml");
            exec("sed -i 's/SystemdCgroup = false/SystemdCgroup = true/' /etc/containerd/config.toml");
        },
        _ => panic!("Unsupported OS for containerd installation: {}", os),
    }
    
    // Start and enable containerd
    exec("systemctl daemon-reload");
    exec("systemctl enable containerd");
    exec("systemctl start containerd");
    
    echo("✅ containerd installed and configured");
}

fn install_docker(os: &str) {
    match os {
        "ubuntu" | "debian" => {
            exec("apt-get update");
            exec("apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release");
            
            exec("curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg");
            echo("deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable") > "/etc/apt/sources.list.d/docker.list";
            
            exec("apt-get update");
            exec("apt-get install -y docker-ce docker-ce-cli containerd.io");
        },
        "rhel" => {
            exec("yum install -y yum-utils");
            exec("yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo");
            exec("yum install -y docker-ce docker-ce-cli containerd.io");
        },
        _ => panic!("Unsupported OS for Docker installation: {}", os),
    }
    
    // Configure Docker daemon for Kubernetes
    let docker_daemon_config = r#"{
  "exec-opts": ["native.cgroupdriver=systemd"],
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "100m"
  },
  "storage-driver": "overlay2"
}"#;
    
    exec("mkdir -p /etc/docker");
    write_file("/etc/docker/daemon.json", docker_daemon_config);
    
    exec("systemctl daemon-reload");
    exec("systemctl enable docker");
    exec("systemctl start docker");
    
    echo("✅ Docker installed and configured");
}

fn install_kubernetes_tools(version: &str, os: &str, arch: &str) {
    echo(&format!("⚙️ Installing Kubernetes tools version {}", version));
    
    match os {
        "ubuntu" | "debian" => {
            // Add Kubernetes repository
            exec("curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/kubernetes-archive-keyring.gpg");
            echo("deb [signed-by=/usr/share/keyrings/kubernetes-archive-keyring.gpg] https://apt.kubernetes.io/ kubernetes-xenial main") > "/etc/apt/sources.list.d/kubernetes.list";
            
            exec("apt-get update");
            exec(&format!("apt-get install -y kubelet={}-00 kubeadm={}-00 kubectl={}-00", version, version, version));
            exec("apt-mark hold kubelet kubeadm kubectl");
        },
        "rhel" => {
            // Add Kubernetes repository
            let k8s_repo = r#"[kubernetes]
name=Kubernetes
baseurl=https://packages.cloud.google.com/yum/repos/kubernetes-el7-$basearch
enabled=1
gpgcheck=1
gpgkey=https://packages.cloud.google.com/yum/doc/yum-key.gpg https://packages.cloud.google.com/yum/doc/rpm-package-key.gpg
exclude=kubelet kubeadm kubectl"#;
            
            write_file("/etc/yum.repos.d/kubernetes.repo", k8s_repo);
            
            exec("setenforce 0");
            exec("sed -i 's/^SELINUX=enforcing$/SELINUX=permissive/' /etc/selinux/config");
            
            exec(&format!("yum install -y kubelet-{} kubeadm-{} kubectl-{} --disableexcludes=kubernetes", version, version, version));
        },
        _ => {
            // Manual installation for other systems
            install_kubernetes_tools_manual(version, arch);
        },
    }
    
    exec("systemctl enable kubelet");
    echo("✅ Kubernetes tools installed");
}

fn install_kubernetes_tools_manual(version: &str, arch: &str) {
    let tools = ["kubeadm", "kubelet", "kubectl"];
    
    for tool in &tools {
        let url = format!("https://dl.k8s.io/release/v{}/bin/linux/{}/{}", version, arch, tool);
        curl(&url, &format!("/usr/local/bin/{}", tool));
        chmod("+x", &format!("/usr/local/bin/{}", tool));
    }
    
    // Create kubelet systemd service
    let kubelet_service = r#"[Unit]
Description=kubelet: The Kubernetes Node Agent
Documentation=https://kubernetes.io/docs/
Wants=network-online.target
After=network-online.target

[Service]
ExecStart=/usr/local/bin/kubelet
Restart=always
StartLimitInterval=0
RestartSec=10

[Install]
WantedBy=multi-user.target"#;
    
    write_file("/etc/systemd/system/kubelet.service", kubelet_service);
    exec("systemctl daemon-reload");
}

fn configure_system_for_kubernetes() {
    echo("🔧 Configuring system for Kubernetes...");
    
    // Disable swap
    exec("swapoff -a");
    exec("sed -i '/ swap / s/^\(.*\)$/#\1/g' /etc/fstab");
    
    // Configure iptables to see bridged traffic
    let k8s_conf = r#"net.bridge.bridge-nf-call-iptables  = 1
net.bridge.bridge-nf-call-ip6tables = 1
net.ipv4.ip_forward                 = 1"#;
    
    write_file("/etc/sysctl.d/k8s.conf", k8s_conf);
    exec("sysctl --system");
    
    // Load required kernel modules
    let modules = r#"overlay
br_netfilter"#;
    
    write_file("/etc/modules-load.d/k8s.conf", modules);
    exec("modprobe overlay");
    exec("modprobe br_netfilter");
    
    echo("✅ System configured for Kubernetes");
}

fn initialize_control_plane(cluster_name: &str, version: &str, cni: &str) {
    echo("🎛️ Initializing Kubernetes control plane...");
    
    let pod_subnet = match cni {
        "calico" => "192.168.0.0/16",
        "flannel" => "10.244.0.0/16",
        "weave" => "10.32.0.0/12",
        _ => "10.244.0.0/16",
    };
    
    let kubeadm_config = format!(r#"apiVersion: kubeadm.k8s.io/v1beta3
kind: InitConfiguration
localAPIEndpoint:
  advertiseAddress: {}
  bindPort: 6443
nodeRegistration:
  criSocket: unix:///var/run/containerd/containerd.sock
---
apiVersion: kubeadm.k8s.io/v1beta3
kind: ClusterConfiguration
clusterName: {}
kubernetesVersion: v{}
networking:
  serviceSubnet: 10.96.0.0/12
  podSubnet: {}
  dnsDomain: cluster.local
apiServer:
  certSANs:
  - localhost
  - 127.0.0.1
  extraArgs:
    authorization-mode: RBAC
controllerManager:
  extraArgs:
    bind-address: 0.0.0.0
scheduler:
  extraArgs:
    bind-address: 0.0.0.0
etcd:
  local:
    dataDir: /var/lib/etcd
---
apiVersion: kubelet.config.k8s.io/v1beta1
kind: KubeletConfiguration
cgroupDriver: systemd
"#, get_primary_ip(), cluster_name, version, pod_subnet);
    
    write_file("/tmp/kubeadm-config.yaml", &kubeadm_config);
    
    exec("kubeadm init --config=/tmp/kubeadm-config.yaml --upload-certs");
    
    echo("✅ Control plane initialized");
}

fn get_primary_ip() -> String {
    // Try to get the primary network interface IP
    let ip_output = exec_output("ip route get 8.8.8.8 | grep -oP 'src \\K\\S+'");
    if !ip_output.trim().is_empty() {
        return ip_output.trim().to_string();
    }
    
    // Fallback to hostname -I
    let hostname_output = exec_output("hostname -I | awk '{print $1}'");
    hostname_output.trim().to_string()
}

fn install_cni_plugin(cni: &str) {
    echo(&format!("🌐 Installing CNI plugin: {}", cni));
    
    match cni {
        "calico" => install_calico(),
        "flannel" => install_flannel(),
        "weave" => install_weave(),
        _ => panic!("Unsupported CNI plugin: {}", cni),
    }
}

fn install_calico() {
    exec("kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.26.1/manifests/tigera-operator.yaml");
    
    let calico_config = r#"apiVersion: operator.tigera.io/v1
kind: Installation
metadata:
  name: default
spec:
  calicoNetwork:
    ipPools:
    - blockSize: 26
      cidr: 192.168.0.0/16
      encapsulation: VXLANCrossSubnet
      natOutgoing: Enabled
      nodeSelector: all()
---
apiVersion: operator.tigera.io/v1
kind: APIServer
metadata:
  name: default
spec: {}"#;
    
    write_file("/tmp/calico-config.yaml", calico_config);
    exec("kubectl create -f /tmp/calico-config.yaml");
    
    // Wait for Calico to be ready
    exec("kubectl wait --for=condition=available --timeout=300s deployment/calico-kube-controllers -n calico-system");
    
    echo("✅ Calico CNI installed");
}

fn install_flannel() {
    exec("kubectl apply -f https://github.com/flannel-io/flannel/releases/latest/download/kube-flannel.yml");
    
    // Wait for flannel to be ready
    exec("kubectl wait --for=condition=ready --timeout=300s pod -l app=flannel -n kube-flannel");
    
    echo("✅ Flannel CNI installed");
}

fn install_weave() {
    exec("kubectl apply -f https://github.com/weaveworks/weave/releases/download/v2.8.1/weave-daemonset-k8s.yaml");
    
    // Wait for weave to be ready
    exec("kubectl wait --for=condition=ready --timeout=300s pod -l name=weave-net -n kube-system");
    
    echo("✅ Weave CNI installed");
}

fn install_cluster_components() {
    echo("🔧 Installing essential cluster components...");
    
    // Install metrics server
    install_metrics_server();
    
    // Install ingress controller
    install_ingress_controller();
    
    // Install local storage provisioner
    install_local_storage_provisioner();
    
    // Install dashboard (optional)
    if env_var_or("INSTALL_DASHBOARD", "true") == "true" {
        install_kubernetes_dashboard();
    }
}

fn install_metrics_server() {
    exec("kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml");
    
    // Patch metrics server for local clusters
    exec("kubectl patch deployment metrics-server -n kube-system --type='json' -p='[{\"op\": \"add\", \"path\": \"/spec/template/spec/containers/0/args/-\", \"value\": \"--kubelet-insecure-tls\"}]'");
    
    exec("kubectl wait --for=condition=available --timeout=300s deployment/metrics-server -n kube-system");
    echo("✅ Metrics server installed");
}

fn install_ingress_controller() {
    // Install NGINX Ingress Controller
    exec("kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/baremetal/deploy.yaml");
    
    exec("kubectl wait --for=condition=ready --timeout=300s pod -l app.kubernetes.io/component=controller -n ingress-nginx");
    echo("✅ NGINX Ingress Controller installed");
}

fn install_local_storage_provisioner() {
    let storage_class = r#"apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: local-storage
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer"#;
    
    write_file("/tmp/local-storage-class.yaml", storage_class);
    exec("kubectl apply -f /tmp/local-storage-class.yaml");
    
    echo("✅ Local storage provisioner configured");
}

fn install_kubernetes_dashboard() {
    exec("kubectl apply -f https://raw.githubusercontent.com/kubernetes/dashboard/v2.7.0/aio/deploy/recommended.yaml");
    
    // Create admin user for dashboard
    let dashboard_admin = r#"apiVersion: v1
kind: ServiceAccount
metadata:
  name: admin-user
  namespace: kubernetes-dashboard
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: admin-user
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: cluster-admin
subjects:
- kind: ServiceAccount
  name: admin-user
  namespace: kubernetes-dashboard"#;
    
    write_file("/tmp/dashboard-admin.yaml", dashboard_admin);
    exec("kubectl apply -f /tmp/dashboard-admin.yaml");
    
    echo("✅ Kubernetes Dashboard installed");
    echo("Access token can be retrieved with: kubectl -n kubernetes-dashboard create token admin-user");
}

fn configure_kubectl_access() {
    echo("🔑 Configuring kubectl access...");
    
    let user_home = env_var("HOME");
    let kube_dir = format!("{}/.kube", user_home);
    
    exec(&format!("mkdir -p {}", kube_dir));
    exec(&format!("cp -i /etc/kubernetes/admin.conf {}/config", kube_dir));
    
    // Change ownership if not root
    let current_user = exec_output("whoami");
    if current_user.trim() != "root" {
        exec(&format!("chown $(id -u):$(id -g) {}/config", kube_dir));
    }
    
    echo("✅ kubectl configured for current user");
}

fn verify_cluster_installation() {
    echo("🔍 Verifying cluster installation...");
    
    // Wait for all system pods to be ready
    exec("kubectl wait --for=condition=ready --timeout=600s pod --all -n kube-system");
    
    // Check node status
    let nodes = exec_output("kubectl get nodes --no-headers | wc -l");
    echo(&format!("✅ Cluster nodes: {}", nodes.trim()));
    
    // Check system pods
    let system_pods = exec_output("kubectl get pods -n kube-system --no-headers | grep Running | wc -l");
    echo(&format!("✅ Running system pods: {}", system_pods.trim()));
    
    // Verify CNI is working
    exec("kubectl run test-pod --image=busybox --rm -it --restart=Never -- nslookup kubernetes.default");
    
    echo("✅ Cluster verification completed");
}

// Utility functions
fn command_exists(command: &str) -> bool {
    exec_output(&format!("command -v {}", command)).trim() != ""
}

fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

fn env_var_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}