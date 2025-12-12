//! Tests for web server recipes (v0.0.460).

use super::*;

#[test]
fn test_detect_install_nginx() {
    assert_eq!(
        detect_feature("install nginx on my server"),
        Some(WebServerFeature::InstallNginx)
    );
    assert_eq!(
        detect_feature("setup nginx web server"),
        Some(WebServerFeature::InstallNginx)
    );
}

#[test]
fn test_detect_install_apache() {
    // Keywords: "install apache", "install httpd", "setup apache"
    assert_eq!(
        detect_feature("install apache server"),
        Some(WebServerFeature::InstallApache)
    );
    assert_eq!(
        detect_feature("install httpd on server"),
        Some(WebServerFeature::InstallApache)
    );
}

#[test]
fn test_detect_virtual_host() {
    assert_eq!(
        detect_feature("create nginx virtual host"),
        Some(WebServerFeature::CreateVirtualHost)
    );
    assert_eq!(
        detect_feature("add server block for nginx"),
        Some(WebServerFeature::CreateVirtualHost)
    );
    assert_eq!(
        detect_feature("configure apache vhost"),
        Some(WebServerFeature::CreateVirtualHost)
    );
}

#[test]
fn test_detect_enable_site() {
    assert_eq!(
        detect_feature("nginx enable site"),
        Some(WebServerFeature::EnableSite)
    );
    assert_eq!(
        detect_feature("apache a2ensite command"),
        Some(WebServerFeature::EnableSite)
    );
}

#[test]
fn test_detect_ssl() {
    assert_eq!(
        detect_feature("configure ssl for nginx"),
        Some(WebServerFeature::ConfigureSsl)
    );
    assert_eq!(
        detect_feature("setup https certificate"),
        Some(WebServerFeature::ConfigureSsl)
    );
    assert_eq!(
        detect_feature("install certbot for web server"),
        Some(WebServerFeature::ConfigureSsl)
    );
    assert_eq!(
        detect_feature("letsencrypt nginx setup"),
        Some(WebServerFeature::ConfigureSsl)
    );
}

#[test]
fn test_detect_reverse_proxy() {
    assert_eq!(
        detect_feature("setup nginx reverse proxy"),
        Some(WebServerFeature::ReverseProxy)
    );
    assert_eq!(
        detect_feature("configure proxy_pass nginx"),
        Some(WebServerFeature::ReverseProxy)
    );
}

#[test]
fn test_detect_load_balancing() {
    assert_eq!(
        detect_feature("nginx load balancer setup"),
        Some(WebServerFeature::LoadBalancing)
    );
    assert_eq!(
        detect_feature("configure upstream servers nginx"),
        Some(WebServerFeature::LoadBalancing)
    );
}

#[test]
fn test_detect_logs() {
    assert_eq!(
        detect_feature("view nginx access log"),
        Some(WebServerFeature::ViewLogs)
    );
    assert_eq!(
        detect_feature("check apache error log"),
        Some(WebServerFeature::ViewLogs)
    );
}

#[test]
fn test_detect_test_config() {
    assert_eq!(
        detect_feature("nginx -t syntax check"),
        Some(WebServerFeature::TestConfig)
    );
    assert_eq!(
        detect_feature("apache configtest run"),
        Some(WebServerFeature::TestConfig)
    );
}

#[test]
fn test_detect_restart() {
    assert_eq!(
        detect_feature("restart nginx service"),
        Some(WebServerFeature::RestartServer)
    );
    assert_eq!(
        detect_feature("reload apache after changes"),
        Some(WebServerFeature::RestartServer)
    );
}

#[test]
fn test_detect_status() {
    assert_eq!(
        detect_feature("check nginx status"),
        Some(WebServerFeature::CheckStatus)
    );
    assert_eq!(
        detect_feature("apache status check"),
        Some(WebServerFeature::CheckStatus)
    );
}

#[test]
fn test_detect_caching() {
    assert_eq!(
        detect_feature("nginx cache configuration"),
        Some(WebServerFeature::ConfigureCaching)
    );
    assert_eq!(
        detect_feature("set expires header nginx"),
        Some(WebServerFeature::ConfigureCaching)
    );
}

#[test]
fn test_detect_basic_auth() {
    assert_eq!(
        detect_feature("nginx htpasswd setup"),
        Some(WebServerFeature::BasicAuth)
    );
    assert_eq!(
        detect_feature("password protect nginx directory"),
        Some(WebServerFeature::BasicAuth)
    );
}

#[test]
fn test_detect_cors() {
    assert_eq!(
        detect_feature("nginx cors configuration"),
        Some(WebServerFeature::ConfigureCors)
    );
    assert_eq!(
        detect_feature("add access-control-allow headers nginx"),
        Some(WebServerFeature::ConfigureCors)
    );
}

#[test]
fn test_detect_performance() {
    assert_eq!(
        detect_feature("optimize nginx performance"),
        Some(WebServerFeature::OptimizePerformance)
    );
    assert_eq!(
        detect_feature("nginx gzip compression"),
        Some(WebServerFeature::OptimizePerformance)
    );
}

#[test]
fn test_not_webserver_query() {
    assert_eq!(detect_feature("how much disk space"), None);
    assert_eq!(detect_feature("install htop"), None);
    assert_eq!(detect_feature("restart docker"), None);
    assert_eq!(detect_feature("kubernetes pods"), None);
}

#[test]
fn test_match_query_returns_recipe() {
    let recipe = match_query("install nginx");
    assert!(recipe.is_some());
    let recipe = recipe.unwrap();
    assert_eq!(recipe.feature, WebServerFeature::InstallNginx);
    assert!(!recipe.commands.is_empty());
    assert!(!recipe.answer_template.is_empty());
}

#[test]
fn test_all_features_have_recipes() {
    let recipes = builtin_recipes();
    let features: Vec<WebServerFeature> = recipes.iter().map(|r| r.feature).collect();

    assert!(features.contains(&WebServerFeature::InstallNginx));
    assert!(features.contains(&WebServerFeature::InstallApache));
    assert!(features.contains(&WebServerFeature::CreateVirtualHost));
    assert!(features.contains(&WebServerFeature::EnableSite));
    assert!(features.contains(&WebServerFeature::ConfigureSsl));
    assert!(features.contains(&WebServerFeature::ReverseProxy));
    assert!(features.contains(&WebServerFeature::LoadBalancing));
    assert!(features.contains(&WebServerFeature::ViewLogs));
    assert!(features.contains(&WebServerFeature::TestConfig));
    assert!(features.contains(&WebServerFeature::RestartServer));
    assert!(features.contains(&WebServerFeature::CheckStatus));
    assert!(features.contains(&WebServerFeature::ConfigureCaching));
    assert!(features.contains(&WebServerFeature::BasicAuth));
    assert!(features.contains(&WebServerFeature::ConfigureCors));
    assert!(features.contains(&WebServerFeature::OptimizePerformance));
}

#[test]
fn test_feature_display_names() {
    assert_eq!(WebServerFeature::InstallNginx.display_name(), "install nginx");
    assert_eq!(WebServerFeature::ConfigureSsl.display_name(), "configure SSL/TLS");
    assert_eq!(WebServerFeature::ReverseProxy.display_name(), "set up reverse proxy");
}

#[test]
fn test_server_type_display() {
    assert_eq!(WebServerType::Nginx.to_string(), "nginx");
    assert_eq!(WebServerType::Apache.to_string(), "apache");
}

#[test]
fn test_recipe_builder() {
    let recipe = WebServerRecipe::new(WebServerFeature::InstallNginx, "Test")
        .for_nginx()
        .with_command("test command")
        .with_config("test config")
        .with_answer("test answer")
        .with_note("test note");

    assert_eq!(recipe.feature, WebServerFeature::InstallNginx);
    assert_eq!(recipe.server_type, Some(WebServerType::Nginx));
    assert!(recipe.requires.contains(&"nginx".to_string()));
    assert_eq!(recipe.commands, vec!["test command"]);
    assert_eq!(recipe.config_example, Some("test config".to_string()));
    assert_eq!(recipe.answer_template, "test answer");
    assert_eq!(recipe.notes, vec!["test note"]);
}

#[test]
fn test_ssl_recipe_has_config() {
    let recipes = builtin_recipes();
    let ssl_recipe = recipes
        .iter()
        .find(|r| r.feature == WebServerFeature::ConfigureSsl)
        .unwrap();

    assert!(ssl_recipe.config_example.is_some());
    let config = ssl_recipe.config_example.as_ref().unwrap();
    assert!(config.contains("ssl_certificate"));
    assert!(config.contains("443"));
}

#[test]
fn test_reverse_proxy_recipe_has_config() {
    let recipes = builtin_recipes();
    let proxy_recipe = recipes
        .iter()
        .find(|r| r.feature == WebServerFeature::ReverseProxy)
        .unwrap();

    assert!(proxy_recipe.config_example.is_some());
    let config = proxy_recipe.config_example.as_ref().unwrap();
    assert!(config.contains("proxy_pass"));
    assert!(config.contains("X-Real-IP"));
}
