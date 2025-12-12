//! Web server builtin recipes (v0.0.460).

use super::types::{WebServerFeature, WebServerRecipe};

/// Get all builtin web server recipes
pub fn builtin_recipes() -> Vec<WebServerRecipe> {
    vec![
        // Installation
        WebServerRecipe::new(WebServerFeature::InstallNginx, "Install and start Nginx")
            .for_nginx()
            .with_command("sudo pacman -S nginx")
            .with_command("sudo apt install nginx")
            .with_command("sudo dnf install nginx")
            .with_command("sudo systemctl enable --now nginx")
            .with_answer(
                "Install Nginx with your package manager: `pacman -S nginx` (Arch), \
                 `apt install nginx` (Debian/Ubuntu), or `dnf install nginx` (Fedora). \
                 Then enable and start with `systemctl enable --now nginx`.",
            )
            .with_note("Config location: /etc/nginx/nginx.conf"),
        WebServerRecipe::new(WebServerFeature::InstallApache, "Install and start Apache")
            .for_apache()
            .with_command("sudo pacman -S apache")
            .with_command("sudo apt install apache2")
            .with_command("sudo dnf install httpd")
            .with_command("sudo systemctl enable --now httpd")
            .with_answer(
                "Install Apache: `pacman -S apache` (Arch), `apt install apache2` (Debian), \
                 or `dnf install httpd` (Fedora). Enable with `systemctl enable --now httpd`.",
            )
            .with_note("Config location: /etc/httpd/conf/httpd.conf or /etc/apache2/"),
        // Virtual hosts
        WebServerRecipe::new(
            WebServerFeature::CreateVirtualHost,
            "Create Nginx server block",
        )
        .for_nginx()
        .with_command("sudo nano /etc/nginx/sites-available/example.com")
        .with_command("sudo ln -s /etc/nginx/sites-available/example.com /etc/nginx/sites-enabled/")
        .with_config(
            r#"server {
    listen 80;
    listen [::]:80;
    server_name example.com www.example.com;
    root /var/www/example.com;
    index index.html index.htm;

    location / {
        try_files $uri $uri/ =404;
    }

    access_log /var/log/nginx/example.com.access.log;
    error_log /var/log/nginx/example.com.error.log;
}"#,
        )
        .with_answer(
            "Create a server block in `/etc/nginx/sites-available/`, then symlink to \
             `sites-enabled/`. Set server_name, root, and location directives.",
        )
        .with_note("Test with nginx -t before reloading"),
        // Enable/disable sites
        WebServerRecipe::new(WebServerFeature::EnableSite, "Enable or disable website")
            .with_command("sudo ln -s /etc/nginx/sites-available/site /etc/nginx/sites-enabled/")
            .with_command("sudo rm /etc/nginx/sites-enabled/site")
            .with_command("sudo a2ensite site.conf")
            .with_command("sudo a2dissite site.conf")
            .with_answer(
                "Nginx: symlink to sites-enabled to enable, remove symlink to disable. \
                 Apache: use `a2ensite` and `a2dissite` commands.",
            ),
        // SSL/TLS
        WebServerRecipe::new(WebServerFeature::ConfigureSsl, "Configure SSL/TLS with Let's Encrypt")
            .with_command("sudo pacman -S certbot certbot-nginx")
            .with_command("sudo apt install certbot python3-certbot-nginx")
            .with_command("sudo certbot --nginx -d example.com")
            .with_command("sudo certbot renew --dry-run")
            .with_config(
                r#"server {
    listen 443 ssl http2;
    server_name example.com;

    ssl_certificate /etc/letsencrypt/live/example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # ... rest of config
}"#,
            )
            .with_answer(
                "Use Certbot for free Let's Encrypt certificates: \
                 `certbot --nginx -d example.com`. Certbot auto-configures Nginx. \
                 Set up auto-renewal with `certbot renew`.",
            )
            .with_note("Certificates auto-renew via systemd timer"),
        // Reverse proxy
        WebServerRecipe::new(WebServerFeature::ReverseProxy, "Set up reverse proxy")
            .for_nginx()
            .with_config(
                r#"server {
    listen 80;
    server_name app.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}"#,
            )
            .with_answer(
                "Use `proxy_pass` to forward requests to a backend server. Include headers \
                 like X-Real-IP and X-Forwarded-For for proper client identification.",
            )
            .with_note("Add proxy_read_timeout for slow backends"),
        // Load balancing
        WebServerRecipe::new(WebServerFeature::LoadBalancing, "Configure load balancing")
            .for_nginx()
            .with_config(
                r#"upstream backend {
    least_conn;  # or: round_robin, ip_hash
    server backend1.example.com:8080 weight=3;
    server backend2.example.com:8080;
    server backend3.example.com:8080 backup;
}

server {
    listen 80;
    server_name lb.example.com;

    location / {
        proxy_pass http://backend;
    }
}"#,
            )
            .with_answer(
                "Define an `upstream` block with backend servers. Methods: round_robin (default), \
                 least_conn, ip_hash. Use weight for unequal distribution, backup for failover.",
            )
            .with_note("Add health checks with nginx-plus or external tools"),
        // Logs
        WebServerRecipe::new(WebServerFeature::ViewLogs, "View web server logs")
            .with_command("sudo tail -f /var/log/nginx/access.log")
            .with_command("sudo tail -f /var/log/nginx/error.log")
            .with_command("sudo tail -f /var/log/apache2/access.log")
            .with_command("sudo tail -f /var/log/httpd/access_log")
            .with_command("journalctl -u nginx -f")
            .with_answer(
                "Nginx logs: `/var/log/nginx/access.log` and `error.log`. \
                 Apache: `/var/log/apache2/` or `/var/log/httpd/`. \
                 Use `tail -f` to follow in real-time.",
            )
            .with_note("Use journalctl -u nginx for systemd journal logs"),
        // Test config
        WebServerRecipe::new(WebServerFeature::TestConfig, "Test configuration syntax")
            .with_command("sudo nginx -t")
            .with_command("sudo apachectl configtest")
            .with_command("sudo httpd -t")
            .with_answer(
                "Always test before reloading! Nginx: `nginx -t`. \
                 Apache: `apachectl configtest` or `httpd -t`. \
                 This validates syntax without affecting running server.",
            )
            .with_note("Fix any errors before reloading"),
        // Restart/reload
        WebServerRecipe::new(WebServerFeature::RestartServer, "Restart or reload web server")
            .with_command("sudo systemctl reload nginx")
            .with_command("sudo systemctl restart nginx")
            .with_command("sudo systemctl reload apache2")
            .with_command("sudo systemctl reload httpd")
            .with_command("sudo nginx -s reload")
            .with_answer(
                "Use `systemctl reload` for graceful reload (no downtime). \
                 Use `restart` only when reload fails. \
                 Nginx: `nginx -s reload` also works.",
            )
            .with_note("Always test config before reloading"),
        // Status
        WebServerRecipe::new(WebServerFeature::CheckStatus, "Check web server status")
            .with_command("systemctl status nginx")
            .with_command("systemctl status apache2")
            .with_command("systemctl status httpd")
            .with_command("curl -I http://localhost")
            .with_answer(
                "Check service status with `systemctl status nginx/apache2/httpd`. \
                 Test response with `curl -I http://localhost` for headers only.",
            ),
        // Caching
        WebServerRecipe::new(WebServerFeature::ConfigureCaching, "Configure browser caching")
            .for_nginx()
            .with_config(
                r#"# Static file caching
location ~* \.(jpg|jpeg|png|gif|ico|css|js|woff2)$ {
    expires 30d;
    add_header Cache-Control "public, no-transform";
}

# Disable caching for dynamic content
location ~ \.php$ {
    add_header Cache-Control "no-store, no-cache, must-revalidate";
}"#,
            )
            .with_answer(
                "Use `expires` directive for static files. Add Cache-Control headers. \
                 Common: 30d for images/CSS/JS, no-cache for dynamic content.",
            )
            .with_note("Use versioned filenames for cache busting"),
        // Basic auth
        WebServerRecipe::new(WebServerFeature::BasicAuth, "Set up HTTP basic authentication")
            .with_command("sudo htpasswd -c /etc/nginx/.htpasswd username")
            .with_command("sudo htpasswd /etc/nginx/.htpasswd another_user")
            .with_config(
                r#"location /admin {
    auth_basic "Restricted Area";
    auth_basic_user_file /etc/nginx/.htpasswd;
}"#,
            )
            .with_answer(
                "Create password file with `htpasswd -c /etc/nginx/.htpasswd username`. \
                 Add `auth_basic` directive to location block. \
                 Use -c only for first user (creates file).",
            )
            .with_note("Always use HTTPS with basic auth"),
        // CORS
        WebServerRecipe::new(WebServerFeature::ConfigureCors, "Configure CORS headers")
            .for_nginx()
            .with_config(
                r#"location /api {
    add_header 'Access-Control-Allow-Origin' '*' always;
    add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS, PUT, DELETE' always;
    add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,Content-Type,Authorization' always;

    if ($request_method = 'OPTIONS') {
        add_header 'Access-Control-Max-Age' 1728000;
        add_header 'Content-Type' 'text/plain; charset=utf-8';
        add_header 'Content-Length' 0;
        return 204;
    }
}"#,
            )
            .with_answer(
                "Add Access-Control-Allow-* headers for CORS. Handle OPTIONS preflight requests. \
                 Replace '*' with specific origins in production for security.",
            )
            .with_note("Be specific with allowed origins in production"),
        // Performance
        WebServerRecipe::new(WebServerFeature::OptimizePerformance, "Optimize Nginx performance")
            .for_nginx()
            .with_config(
                r#"# nginx.conf optimizations
worker_processes auto;
worker_connections 1024;

http {
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript;

    # Open file cache
    open_file_cache max=1000 inactive=20s;
    open_file_cache_valid 30s;
    open_file_cache_min_uses 2;
}"#,
            )
            .with_answer(
                "Key optimizations: `worker_processes auto`, enable sendfile and tcp_nopush, \
                 enable gzip compression, use open_file_cache for static files.",
            )
            .with_note("Adjust worker_connections based on expected load"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipes_exist() {
        let recipes = builtin_recipes();
        assert!(!recipes.is_empty());
    }

    #[test]
    fn test_recipes_have_commands_or_config() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.commands.is_empty() || recipe.config_example.is_some(),
                "Recipe {:?} has no commands or config",
                recipe.feature
            );
        }
    }

    #[test]
    fn test_recipes_have_answers() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.answer_template.is_empty(),
                "Recipe {:?} has no answer",
                recipe.feature
            );
        }
    }
}
