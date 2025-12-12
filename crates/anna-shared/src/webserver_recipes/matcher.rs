//! Web server query matching (v0.0.460).

use super::recipes::builtin_recipes;
use super::types::{WebServerFeature, WebServerRecipe};

/// Detect if a query is about web servers
pub fn detect_feature(query: &str) -> Option<WebServerFeature> {
    let lower = query.to_lowercase();

    // First check if it's even a web server query
    if !is_webserver_query(&lower) {
        return None;
    }

    // Find all matching keywords and return the feature with the longest match
    let mut best_match: Option<(WebServerFeature, usize)> = None;

    for feature in all_features() {
        for keyword in feature.keywords() {
            if lower.contains(keyword) {
                let keyword_len = keyword.len();
                if best_match.is_none() || keyword_len > best_match.unwrap().1 {
                    best_match = Some((feature, keyword_len));
                }
            }
        }
    }

    best_match.map(|(f, _)| f)
}

/// Match a query to a recipe
pub fn match_query(query: &str) -> Option<WebServerRecipe> {
    let feature = detect_feature(query)?;

    builtin_recipes()
        .into_iter()
        .find(|r| r.feature == feature)
}

/// Check if query is about web servers
fn is_webserver_query(query: &str) -> bool {
    let webserver_indicators = [
        "nginx",
        "apache",
        "httpd",
        "web server",
        "webserver",
        "virtual host",
        "vhost",
        "server block",
        "reverse proxy",
        "load balanc",
        "ssl",
        "tls",
        "https",
        "certbot",
        "let's encrypt",
        "letsencrypt",
        "htpasswd",
        "sites-available",
        "sites-enabled",
        "proxy_pass",
        "upstream",
    ];

    webserver_indicators.iter().any(|k| query.contains(k))
}

/// Get all web server features
fn all_features() -> Vec<WebServerFeature> {
    vec![
        WebServerFeature::InstallNginx,
        WebServerFeature::InstallApache,
        WebServerFeature::CreateVirtualHost,
        WebServerFeature::EnableSite,
        WebServerFeature::ConfigureSsl,
        WebServerFeature::ReverseProxy,
        WebServerFeature::LoadBalancing,
        WebServerFeature::ViewLogs,
        WebServerFeature::TestConfig,
        WebServerFeature::RestartServer,
        WebServerFeature::CheckStatus,
        WebServerFeature::ConfigureCaching,
        WebServerFeature::BasicAuth,
        WebServerFeature::ConfigureCors,
        WebServerFeature::OptimizePerformance,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_install_nginx() {
        assert_eq!(
            detect_feature("install nginx on my server"),
            Some(WebServerFeature::InstallNginx)
        );
    }

    #[test]
    fn test_detect_ssl() {
        assert_eq!(
            detect_feature("configure ssl for nginx"),
            Some(WebServerFeature::ConfigureSsl)
        );
        assert_eq!(
            detect_feature("setup letsencrypt certificate"),
            Some(WebServerFeature::ConfigureSsl)
        );
    }

    #[test]
    fn test_detect_reverse_proxy() {
        assert_eq!(
            detect_feature("setup reverse proxy for nginx"),
            Some(WebServerFeature::ReverseProxy)
        );
    }

    #[test]
    fn test_not_webserver_query() {
        assert_eq!(detect_feature("how much disk space"), None);
        assert_eq!(detect_feature("restart docker"), None);
    }

    #[test]
    fn test_match_query() {
        let recipe = match_query("install nginx");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, WebServerFeature::InstallNginx);
    }
}
