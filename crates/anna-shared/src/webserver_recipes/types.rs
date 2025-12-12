//! Web server recipe types (v0.0.460).

use serde::{Deserialize, Serialize};

/// Web server features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebServerFeature {
    /// Install Nginx
    InstallNginx,
    /// Install Apache
    InstallApache,
    /// Create virtual host
    CreateVirtualHost,
    /// Enable/disable site
    EnableSite,
    /// Configure SSL/TLS
    ConfigureSsl,
    /// Set up reverse proxy
    ReverseProxy,
    /// Configure load balancing
    LoadBalancing,
    /// View access logs
    ViewLogs,
    /// Test configuration
    TestConfig,
    /// Restart server
    RestartServer,
    /// Check status
    CheckStatus,
    /// Configure caching
    ConfigureCaching,
    /// Set up basic auth
    BasicAuth,
    /// Configure CORS
    ConfigureCors,
    /// Optimize performance
    OptimizePerformance,
}

/// Web server type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebServerType {
    Nginx,
    Apache,
}

impl std::fmt::Display for WebServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebServerType::Nginx => write!(f, "nginx"),
            WebServerType::Apache => write!(f, "apache"),
        }
    }
}

impl WebServerFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            WebServerFeature::InstallNginx => "install nginx",
            WebServerFeature::InstallApache => "install apache",
            WebServerFeature::CreateVirtualHost => "create virtual host",
            WebServerFeature::EnableSite => "enable/disable site",
            WebServerFeature::ConfigureSsl => "configure SSL/TLS",
            WebServerFeature::ReverseProxy => "set up reverse proxy",
            WebServerFeature::LoadBalancing => "configure load balancing",
            WebServerFeature::ViewLogs => "view access logs",
            WebServerFeature::TestConfig => "test configuration",
            WebServerFeature::RestartServer => "restart server",
            WebServerFeature::CheckStatus => "check status",
            WebServerFeature::ConfigureCaching => "configure caching",
            WebServerFeature::BasicAuth => "set up basic auth",
            WebServerFeature::ConfigureCors => "configure CORS",
            WebServerFeature::OptimizePerformance => "optimize performance",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            WebServerFeature::InstallNginx => &["install nginx", "setup nginx", "get nginx"],
            WebServerFeature::InstallApache => {
                &["install apache", "install httpd", "setup apache"]
            }
            WebServerFeature::CreateVirtualHost => {
                &["virtual host", "vhost", "server block", "site config"]
            }
            WebServerFeature::EnableSite => {
                &["enable site", "disable site", "a2ensite", "a2dissite", "sites-enabled"]
            }
            WebServerFeature::ConfigureSsl => {
                &["ssl", "tls", "https", "certificate", "certbot", "let's encrypt", "letsencrypt"]
            }
            WebServerFeature::ReverseProxy => {
                &["reverse proxy", "proxy_pass", "proxypass", "backend server"]
            }
            WebServerFeature::LoadBalancing => {
                &["load balance", "load balancer", "upstream", "round robin"]
            }
            WebServerFeature::ViewLogs => {
                &["access log", "error log", "web log", "nginx log", "apache log"]
            }
            WebServerFeature::TestConfig => {
                &["test config", "configtest", "nginx -t", "apachectl -t", "syntax check"]
            }
            WebServerFeature::RestartServer => {
                &["restart nginx", "restart apache", "reload nginx", "reload apache"]
            }
            WebServerFeature::CheckStatus => {
                &["nginx status", "apache status", "httpd status", "web server status"]
            }
            WebServerFeature::ConfigureCaching => {
                &["cache", "caching", "expires", "cache-control", "etag"]
            }
            WebServerFeature::BasicAuth => {
                &["basic auth", "htpasswd", "authentication", "password protect"]
            }
            WebServerFeature::ConfigureCors => {
                &["cors", "cross-origin", "access-control-allow"]
            }
            WebServerFeature::OptimizePerformance => {
                &["optimize", "performance", "gzip", "compression", "worker_processes"]
            }
        }
    }
}

impl std::fmt::Display for WebServerFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A web server recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServerRecipe {
    pub feature: WebServerFeature,
    pub server_type: Option<WebServerType>,
    pub description: String,
    pub commands: Vec<String>,
    pub config_example: Option<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
    /// Required tools
    pub requires: Vec<String>,
}

impl WebServerRecipe {
    pub fn new(feature: WebServerFeature, description: &str) -> Self {
        Self {
            feature,
            server_type: None,
            description: description.to_string(),
            commands: Vec::new(),
            config_example: None,
            answer_template: String::new(),
            notes: Vec::new(),
            requires: Vec::new(),
        }
    }

    pub fn for_nginx(mut self) -> Self {
        self.server_type = Some(WebServerType::Nginx);
        self.requires.push("nginx".to_string());
        self
    }

    pub fn for_apache(mut self) -> Self {
        self.server_type = Some(WebServerType::Apache);
        self.requires.push("apache".to_string());
        self
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_config(mut self, config: &str) -> Self {
        self.config_example = Some(config.to_string());
        self
    }

    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer_template = answer.to_string();
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
}
