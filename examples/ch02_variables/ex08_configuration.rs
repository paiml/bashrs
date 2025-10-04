// Chapter 2, Example 8: Configuration Variables
// Bootstrap installer pattern with multiple configuration values

fn main() {
    let repo_url = "https://github.com/user/repo";
    let branch = "main";
    let deploy_user = "appuser";
    let deploy_group = "appgroup";
    let port = 8080;
    let enable_ssl = true;

    echo("Deployment configuration:");
    echo(repo_url);
    echo(branch);
    echo(deploy_user);
    echo(deploy_group);
    echo("Port: 8080");
    echo("SSL: enabled");
}

fn echo(msg: &str) {}
