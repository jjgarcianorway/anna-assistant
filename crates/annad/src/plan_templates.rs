//! Plan Templates - Pre-defined plans for common system operations.
//! Phase 16: Template plans for known operations.
//! Phase 17: Preflight checks, verification, and rollback.

use anna_shared::action_plan::{ActionPlan, ActionStep};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Generate a plan for common system configuration tasks.
/// This uses pre-defined templates for well-known operations.
pub fn generate_template_plan(question: &str) -> Option<ActionPlan> {
    let q = question.to_lowercase();

    // GDM resolution
    if q.contains("gdm") && q.contains("resolution") {
        return Some(gdm_resolution_plan(question));
    }

    // Disable sleep/suspend
    if (q.contains("disable") || q.contains("prevent") || q.contains("stop"))
        && (q.contains("sleep") || q.contains("suspend"))
    {
        return Some(disable_sleep_plan(question));
    }

    // Lid close behavior
    if q.contains("lid") && (q.contains("close") || q.contains("closing")) {
        return Some(lid_close_plan(question));
    }

    None
}

// --- GDM Resolution Plan ---

fn gdm_resolution_plan(question: &str) -> ActionPlan {
    let resolution = extract_resolution(question).unwrap_or_else(|| "1920x1080".to_string());
    let (width, height) = parse_resolution(&resolution);
    let config_path = "/var/lib/gdm/.config/monitors.xml";

    let mut plan = ActionPlan::new(
        question,
        &format!("Set GDM login screen resolution to {}", resolution),
        "This configures the display resolution for the GDM login screen by setting up \
         a custom Wayland configuration. The change takes effect on next login.",
    );

    // Preflight: Check if already configured
    if gdm_already_configured(&width, &height) {
        plan.mark_no_changes(&format!("GDM already configured for {}x{}", width, height));
        return plan;
    }

    // Step 1: Create monitors.xml config
    let config_content = format!(
        r#"<monitors version="2">
  <configuration>
    <logicalmonitor>
      <x>0</x>
      <y>0</y>
      <primary>yes</primary>
      <monitor>
        <monitorspec>
          <connector>*</connector>
          <vendor>unknown</vendor>
          <product>unknown</product>
          <serial>unknown</serial>
        </monitorspec>
        <mode>
          <width>{}</width>
          <height>{}</height>
          <rate>60</rate>
        </mode>
      </monitor>
    </logicalmonitor>
  </configuration>
</monitors>"#,
        width, height
    );

    let step1 = ActionStep::new(
        "Create GDM monitor configuration",
        &format!(
            "mkdir -p /var/lib/gdm/.config && cat > {} << 'EOF'\n{}\nEOF",
            config_path, config_content
        ),
        true,
    )
    .with_files(&[config_path])
    .with_verify(
        &format!("grep -q '<width>{}</width>' {}", width, config_path),
        "",
    )
    .with_rollback(&format!("rm -f {}", config_path));

    plan.add_step_full(step1);

    // Step 2: Set permissions
    let step2 = ActionStep::new(
        "Set proper ownership",
        "chown -R gdm:gdm /var/lib/gdm/.config",
        true,
    )
    .with_verify(&format!("stat -c '%U:%G' {}", config_path), "gdm:gdm");

    plan.add_step_full(step2);

    plan.set_verification(
        &format!(
            "test -f {} && grep -q '<width>{}</width>' {}",
            config_path, width, config_path
        ),
        "",
        "Verify monitor configuration exists with correct resolution",
    );

    plan
}

fn gdm_already_configured(width: &str, height: &str) -> bool {
    let path = Path::new("/var/lib/gdm/.config/monitors.xml");
    if !path.exists() {
        return false;
    }
    if let Ok(content) = fs::read_to_string(path) {
        let has_width = content.contains(&format!("<width>{}</width>", width));
        let has_height = content.contains(&format!("<height>{}</height>", height));
        return has_width && has_height;
    }
    false
}

// --- Disable Sleep Plan ---

fn disable_sleep_plan(question: &str) -> ActionPlan {
    let mut plan = ActionPlan::new(
        question,
        "Disable automatic sleep and suspend",
        "This masks the systemd sleep targets and configures logind to ignore idle timeouts. \
         The system will no longer automatically sleep or suspend.",
    );

    // Preflight: Check if already disabled
    if sleep_already_disabled() {
        plan.mark_no_changes("Sleep targets already masked and idle action set to ignore");
        return plan;
    }

    let sleep_targets = [
        "sleep.target",
        "suspend.target",
        "hibernate.target",
        "hybrid-sleep.target",
    ];

    // Step 1: Mask sleep targets
    let step1 = ActionStep::new(
        "Mask systemd sleep targets",
        "systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target",
        true,
    )
    .with_units(&sleep_targets)
    .with_verify("systemctl is-enabled sleep.target 2>&1", "masked")
    .with_rollback(
        "systemctl unmask sleep.target suspend.target hibernate.target hybrid-sleep.target",
    );

    plan.add_step_full(step1);

    // Step 2: Configure logind
    let logind_conf = "/etc/systemd/logind.conf.d/no-idle.conf";
    let step2 = ActionStep::new(
        "Configure logind to ignore idle",
        &format!(
            r#"mkdir -p /etc/systemd/logind.conf.d && cat > {} << 'EOF'
[Login]
IdleAction=ignore
IdleActionSec=0
EOF"#,
            logind_conf
        ),
        true,
    )
    .with_files(&[logind_conf])
    .with_verify(&format!("grep -q 'IdleAction=ignore' {}", logind_conf), "")
    .with_rollback(&format!("rm -f {}", logind_conf));

    plan.add_step_full(step2);

    // Step 3: Restart logind
    let step3 = ActionStep::new(
        "Apply logind changes",
        "systemctl restart systemd-logind",
        true,
    )
    .with_units(&["systemd-logind.service"]);

    plan.add_step_full(step3);

    plan.set_verification(
        "systemctl is-enabled sleep.target 2>&1 | grep -q masked && loginctl show -p IdleAction | grep -q ignore",
        "",
        "Verify sleep disabled and logind configured",
    );

    plan
}

fn sleep_already_disabled() -> bool {
    let masked = Command::new("systemctl")
        .args(["is-enabled", "sleep.target"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("masked"))
        .unwrap_or(false);

    if !masked {
        return false;
    }

    Command::new("loginctl")
        .args(["show", "-p", "IdleAction"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("ignore"))
        .unwrap_or(false)
}

// --- Lid Close Plan ---

fn lid_close_plan(question: &str) -> ActionPlan {
    let action = if question.to_lowercase().contains("nothing")
        || question.to_lowercase().contains("ignore")
    {
        "ignore"
    } else if question.to_lowercase().contains("lock") {
        "lock"
    } else if question.to_lowercase().contains("suspend") {
        "suspend"
    } else {
        "ignore"
    };

    let mut plan = ActionPlan::new(
        question,
        &format!("Configure lid close to {}", action),
        &format!(
            "This configures the system to {} when the laptop lid is closed. \
             Applies to both AC power and battery.",
            action
        ),
    );

    // Preflight: Check if already configured
    if lid_already_configured(action) {
        plan.mark_no_changes(&format!("Lid close already set to {}", action));
        return plan;
    }

    let lid_conf = "/etc/systemd/logind.conf.d/lid.conf";
    let step1 = ActionStep::new(
        "Configure lid close behavior",
        &format!(
            r#"mkdir -p /etc/systemd/logind.conf.d && cat > {} << 'EOF'
[Login]
HandleLidSwitch={}
HandleLidSwitchExternalPower={}
HandleLidSwitchDocked={}
EOF"#,
            lid_conf, action, action, action
        ),
        true,
    )
    .with_files(&[lid_conf])
    .with_verify(
        &format!("grep -q 'HandleLidSwitch={}' {}", action, lid_conf),
        "",
    )
    .with_rollback(&format!("rm -f {}", lid_conf));

    plan.add_step_full(step1);

    let step2 = ActionStep::new("Apply changes", "systemctl restart systemd-logind", true)
        .with_units(&["systemd-logind.service"]);

    plan.add_step_full(step2);

    plan.set_verification(
        &format!("loginctl show -p HandleLidSwitch | grep -q {}", action),
        "",
        "Verify lid switch configuration",
    );

    plan
}

fn lid_already_configured(action: &str) -> bool {
    Command::new("loginctl")
        .args(["show", "-p", "HandleLidSwitch"])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.contains(&format!("HandleLidSwitch={}", action))
        })
        .unwrap_or(false)
}

// --- Helpers ---

fn extract_resolution(question: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d{3,4})[xX×](\d{3,4})").ok()?;
    re.captures(question)
        .map(|c| format!("{}x{}", &c[1], &c[2]))
}

fn parse_resolution(resolution: &str) -> (String, String) {
    let parts: Vec<&str> = resolution.split('x').collect();
    (
        parts.first().unwrap_or(&"1920").to_string(),
        parts.get(1).unwrap_or(&"1080").to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resolution() {
        assert_eq!(
            extract_resolution("set to 1920x1080"),
            Some("1920x1080".to_string())
        );
        assert_eq!(
            extract_resolution("change to 2560X1440"),
            Some("2560x1440".to_string())
        );
        assert_eq!(extract_resolution("no resolution here"), None);
    }

    #[test]
    fn test_generate_template_plan_gdm() {
        let plan = generate_template_plan("change GDM resolution to 1920x1080");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(plan.summary.contains("GDM"));
        // Plan requires sudo unless already configured (idempotent)
        assert!(plan.requires_sudo() || !plan.changes_needed);
    }

    #[test]
    fn test_generate_template_plan_sleep() {
        let plan = generate_template_plan("disable sleep when idle");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(plan.summary.contains("sleep"));
    }

    #[test]
    fn test_generate_template_plan_lid() {
        let plan = generate_template_plan("do nothing when lid closes");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(plan.summary.contains("lid"));
    }

    #[test]
    fn test_parse_resolution() {
        let (w, h) = parse_resolution("1920x1080");
        assert_eq!(w, "1920");
        assert_eq!(h, "1080");
    }
}
