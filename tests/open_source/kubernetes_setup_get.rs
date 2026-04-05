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