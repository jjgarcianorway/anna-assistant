//! User Identity - User identification and profile management
//!
//! v6.54.0: Identity, Persistence, and Multi-User Awareness
//!
//! This module manages user identities and profiles, allowing Anna to
//! maintain separate preferences for different users on the same system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sha2::{Sha256, Digest};

/// User identity on this system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserIdentity {
    /// Username
    pub username: String,

    /// User ID (UID)
    pub uid: u32,

    /// Home directory path
    pub home_dir: PathBuf,
}

/// Greeting preferences for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingPreferences {
    /// Preferred greeting style (casual, professional, minimal)
    pub style: String,

    /// Topics to include in greetings
    pub topics: Vec<String>,

    /// Whether to show system alerts
    pub show_system_alerts: bool,

    /// Whether to show user-specific alerts
    pub show_user_alerts: bool,
}

impl Default for GreetingPreferences {
    fn default() -> Self {
        Self {
            style: "professional".to_string(),
            topics: vec![],
            show_system_alerts: true,
            show_user_alerts: true,
        }
    }
}

/// Watch visibility rules for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchVisibilityRules {
    /// Whether this user can see system-wide watches
    pub see_system_watches: bool,

    /// Whether this user can see other users' watches
    pub see_other_user_watches: bool,

    /// Whether this user's watches are visible to others
    pub my_watches_visible_to_others: bool,
}

impl Default for WatchVisibilityRules {
    fn default() -> Self {
        Self {
            see_system_watches: true,
            see_other_user_watches: false,
            my_watches_visible_to_others: false,
        }
    }
}

/// Complete user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Stable profile ID (hash of primary identity)
    pub id: String,

    /// Primary user identity
    pub primary_identity: UserIdentity,

    /// Other identities for the same human (for future cross-machine linking)
    pub other_identities: Vec<UserIdentity>,

    /// Optional personality type (e.g., "INFJ-A")
    pub personality_type: Option<String>,

    /// Greeting preferences
    pub greeting_preferences: GreetingPreferences,

    /// Watch visibility rules
    pub watch_visibility: WatchVisibilityRules,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserIdentity {
    /// Get the current user's identity
    pub fn current() -> Result<Self, std::io::Error> {
        let username = Self::get_username()?;
        let uid = Self::get_uid();
        let home_dir = Self::get_home_dir()?;

        Ok(Self {
            username,
            uid,
            home_dir,
        })
    }

    fn get_username() -> Result<String, std::io::Error> {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine username"
            ))
    }

    fn get_uid() -> u32 {
        #[cfg(unix)]
        {
            unsafe { libc::getuid() }
        }
        #[cfg(not(unix))]
        {
            0
        }
    }

    fn get_home_dir() -> Result<PathBuf, std::io::Error> {
        std::env::var("HOME")
            .map(PathBuf::from)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine home directory"
            ))
    }

    /// Generate a stable profile ID from this identity
    pub fn generate_profile_id(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.username.as_bytes());
        hasher.update(&self.uid.to_le_bytes());
        hasher.update(self.home_dir.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    }
}

impl UserProfile {
    /// Create a new user profile from current identity
    pub fn from_current_user() -> Result<Self, std::io::Error> {
        let identity = UserIdentity::current()?;
        let id = identity.generate_profile_id();

        Ok(Self {
            id,
            primary_identity: identity,
            other_identities: Vec::new(),
            personality_type: None,
            greeting_preferences: GreetingPreferences::default(),
            watch_visibility: WatchVisibilityRules::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Create a profile from a specific identity
    pub fn from_identity(identity: UserIdentity) -> Self {
        let id = identity.generate_profile_id();

        Self {
            id,
            primary_identity: identity,
            other_identities: Vec::new(),
            personality_type: None,
            greeting_preferences: GreetingPreferences::default(),
            watch_visibility: WatchVisibilityRules::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Add an alternate identity (for future cross-machine support)
    pub fn add_identity(&mut self, identity: UserIdentity) {
        if identity != self.primary_identity
            && !self.other_identities.contains(&identity) {
            self.other_identities.push(identity);
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Update personality type
    pub fn set_personality_type(&mut self, personality: String) {
        self.personality_type = Some(personality);
        self.updated_at = chrono::Utc::now();
    }

    /// Update greeting preferences
    pub fn update_greeting_preferences(&mut self, prefs: GreetingPreferences) {
        self.greeting_preferences = prefs;
        self.updated_at = chrono::Utc::now();
    }

    /// Update watch visibility
    pub fn update_watch_visibility(&mut self, rules: WatchVisibilityRules) {
        self.watch_visibility = rules;
        self.updated_at = chrono::Utc::now();
    }

    /// Check if this profile matches a given identity
    pub fn matches_identity(&self, identity: &UserIdentity) -> bool {
        &self.primary_identity == identity
            || self.other_identities.contains(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_identity_current() {
        // This test will succeed if running in a normal environment
        let result = UserIdentity::current();
        assert!(result.is_ok());

        if let Ok(identity) = result {
            assert!(!identity.username.is_empty());
            assert!(identity.home_dir.exists() || cfg!(target_os = "windows"));
        }
    }

    #[test]
    fn test_profile_id_generation() {
        let identity = UserIdentity {
            username: "testuser".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/testuser"),
        };

        let id1 = identity.generate_profile_id();
        let id2 = identity.generate_profile_id();

        // Same identity should generate same ID
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16);
    }

    #[test]
    fn test_different_identities_different_ids() {
        let identity1 = UserIdentity {
            username: "user1".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/user1"),
        };

        let identity2 = UserIdentity {
            username: "user2".to_string(),
            uid: 1001,
            home_dir: PathBuf::from("/home/user2"),
        };

        let id1 = identity1.generate_profile_id();
        let id2 = identity2.generate_profile_id();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_profile_creation() {
        let identity = UserIdentity {
            username: "testuser".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/testuser"),
        };

        let profile = UserProfile::from_identity(identity.clone());

        assert_eq!(profile.primary_identity, identity);
        assert!(profile.other_identities.is_empty());
        assert!(profile.personality_type.is_none());
        assert_eq!(profile.greeting_preferences.style, "professional");
    }

    #[test]
    fn test_add_alternate_identity() {
        let primary = UserIdentity {
            username: "user1".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/user1"),
        };

        let alternate = UserIdentity {
            username: "user1-alt".to_string(),
            uid: 1001,
            home_dir: PathBuf::from("/home/user1-alt"),
        };

        let mut profile = UserProfile::from_identity(primary.clone());
        profile.add_identity(alternate.clone());

        assert_eq!(profile.other_identities.len(), 1);
        assert!(profile.matches_identity(&primary));
        assert!(profile.matches_identity(&alternate));
    }

    #[test]
    fn test_add_duplicate_identity_ignored() {
        let identity = UserIdentity {
            username: "testuser".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/testuser"),
        };

        let mut profile = UserProfile::from_identity(identity.clone());
        profile.add_identity(identity.clone());

        // Should not add primary identity to other_identities
        assert!(profile.other_identities.is_empty());
    }

    #[test]
    fn test_personality_update() {
        let identity = UserIdentity {
            username: "testuser".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/testuser"),
        };

        let mut profile = UserProfile::from_identity(identity);
        assert!(profile.personality_type.is_none());

        profile.set_personality_type("INTJ-A".to_string());
        assert_eq!(profile.personality_type, Some("INTJ-A".to_string()));
    }

    #[test]
    fn test_greeting_preferences_update() {
        let identity = UserIdentity {
            username: "testuser".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/testuser"),
        };

        let mut profile = UserProfile::from_identity(identity);

        let new_prefs = GreetingPreferences {
            style: "casual".to_string(),
            topics: vec!["weather".to_string(), "news".to_string()],
            show_system_alerts: false,
            show_user_alerts: true,
        };

        profile.update_greeting_preferences(new_prefs);

        assert_eq!(profile.greeting_preferences.style, "casual");
        assert_eq!(profile.greeting_preferences.topics.len(), 2);
        assert!(!profile.greeting_preferences.show_system_alerts);
    }

    #[test]
    fn test_watch_visibility_defaults() {
        let identity = UserIdentity {
            username: "testuser".to_string(),
            uid: 1000,
            home_dir: PathBuf::from("/home/testuser"),
        };

        let profile = UserProfile::from_identity(identity);

        assert!(profile.watch_visibility.see_system_watches);
        assert!(!profile.watch_visibility.see_other_user_watches);
        assert!(!profile.watch_visibility.my_watches_visible_to_others);
    }
}
