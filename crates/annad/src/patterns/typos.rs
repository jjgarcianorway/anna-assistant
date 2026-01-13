//! Typo corrections and fuzzy matching for queries.

/// Common misspellings of Linux/tech terms.
/// Format: (misspelling, correct_spelling)
pub const TYPO_CORRECTIONS: &[(&str, &str)] = &[
    // Package managers
    ("pacaman", "pacman"), ("pacmn", "pacman"), ("packman", "pacman"),
    ("pamcan", "pacman"), ("pacmam", "pacman"),
    ("systemclt", "systemctl"), ("sytemctl", "systemctl"), ("systemcl", "systemctl"),
    ("systmctl", "systemctl"),
    ("journalclt", "journalctl"), ("journctl", "journalctl"), ("jounalctl", "journalctl"),
    // Common terms
    ("kernal", "kernel"), ("kerne", "kernel"), ("kernle", "kernel"),
    ("wif", "wifi"), ("wfii", "wifi"), ("wiif", "wifi"),
    ("bluetoth", "bluetooth"), ("bluethooth", "bluetooth"), ("blutooth", "bluetooth"),
    ("bluetooh", "bluetooth"), ("bluettoth", "bluetooth"),
    ("netwrok", "network"), ("newtork", "network"), ("netowrk", "network"),
    ("memroy", "memory"), ("memeory", "memory"), ("memor", "memory"),
    ("stoarge", "storage"), ("stroage", "storage"), ("sotrage", "storage"),
    ("direcotry", "directory"), ("dirctory", "directory"), ("directroy", "directory"),
    ("permisions", "permissions"), ("permsisions", "permissions"), ("permssions", "permissions"),
    ("temperture", "temperature"), ("temprature", "temperature"), ("tempurature", "temperature"),
    // Commands
    ("grb", "grub"), ("grbu", "grub"),
    ("dokcer", "docker"), ("docekr", "docker"), ("dcoker", "docker"),
    ("firwall", "firewall"), ("firewll", "firewall"), ("firewal", "firewall"),
    ("crontba", "crontab"), ("corntab", "crontab"), ("crontb", "crontab"),
    // Hardware
    ("grahpics", "graphics"), ("grpahics", "graphics"), ("graphcis", "graphics"),
    ("processer", "processor"), ("procesor", "processor"), ("proccessor", "processor"),
    ("baterry", "battery"), ("battrey", "battery"), ("batery", "battery"),
    // Services
    ("servcie", "service"), ("serivce", "service"), ("sevice", "service"),
    ("deamon", "daemon"), ("dameon", "daemon"),
    // Actions
    ("instal", "install"), ("intall", "install"), ("isntall", "install"),
    ("uninstal", "uninstall"), ("unintall", "uninstall"),
    ("updte", "update"), ("udpate", "update"), ("upate", "update"),
    ("upgarde", "upgrade"), ("upgrad", "upgrade"), ("upgade", "upgrade"),
    ("rebbot", "reboot"), ("reobot", "reboot"), ("reeboot", "reboot"),
    ("shutdwon", "shutdown"), ("shudown", "shutdown"), ("shutodwn", "shutdown"),
    // File system
    ("partiton", "partition"), ("parttion", "partition"), ("parition", "partition"),
    ("formating", "formatting"), ("fomratting", "formatting"),
    ("mountig", "mounting"), ("moutning", "mounting"),
    // Audio
    ("pipewrie", "pipewire"), ("pipewie", "pipewire"), ("pipwire", "pipewire"),
    ("pulsaudio", "pulseaudio"), ("pusleaudio", "pulseaudio"), ("pulseadio", "pulseaudio"),
    ("headpohnes", "headphones"), ("headhpones", "headphones"), ("headphons", "headphones"),
    ("spekaers", "speakers"), ("spaekers", "speakers"), ("spekers", "speakers"),
    // Printing
    ("pritner", "printer"), ("printr", "printer"), ("prniter", "printer"),
    // SSH
    ("shh", "ssh"), ("shs", "ssh"),
    // Time
    ("timezoen", "timezone"), ("timezon", "timezone"), ("tiemzone", "timezone"),
    // Users
    ("passwrod", "password"), ("pasword", "password"), ("passowrd", "password"),
    ("usernmae", "username"), ("usernam", "username"), ("usrname", "username"),
    // Backup
    ("rsynce", "rsync"), ("rsynv", "rsync"),
    ("bakup", "backup"), ("bakcup", "backup"), ("backpu", "backup"),
    // Locale
    ("keyboad", "keyboard"), ("keybord", "keyboard"), ("keybaord", "keyboard"),
    ("langauge", "language"), ("languge", "language"), ("langage", "language"),
    // Swap
    ("swpa", "swap"), ("sawp", "swap"),
    ("swappines", "swappiness"), ("swapiness", "swappiness"),
    // Process
    ("proceses", "processes"), ("porcess", "process"), ("proccess", "process"),
    ("zombi", "zombie"), ("zombies", "zombie"), ("zombei", "zombie"),
];

/// Apply typo corrections to query.
pub fn fix_typos(q: &str) -> String {
    let mut result = q.to_string();
    for (typo, correction) in TYPO_CORRECTIONS {
        if result.contains(typo) {
            result = result.replace(typo, correction);
        }
    }
    result
}

/// Calculate simple edit distance (Levenshtein) for short strings.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if len_a == 0 { return len_b; }
    if len_b == 0 { return len_a; }
    if len_a > 15 || len_b > 15 { return usize::MAX; }

    let mut matrix = vec![vec![0usize; len_b + 1]; len_a + 1];
    for i in 0..=len_a { matrix[i][0] = i; }
    for j in 0..=len_b { matrix[0][j] = j; }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i-1][j] + 1)
                .min(matrix[i][j-1] + 1)
                .min(matrix[i-1][j-1] + cost);
        }
    }

    matrix[len_a][len_b]
}

/// Key terms that patterns commonly check for fuzzy matching.
pub const FUZZY_TARGETS: &[&str] = &[
    "disk", "memory", "cpu", "gpu", "ram", "storage", "network", "wifi",
    "bluetooth", "battery", "temperature", "kernel", "services", "processes",
    "packages", "installed", "running", "failed", "errors", "logs", "boot",
    "grub", "partition", "mount", "firewall", "ports", "ssh", "docker",
    "steam", "wine", "proton", "vulkan", "opengl", "audio", "sound",
    "display", "monitor", "resolution", "wayland", "gnome", "kde", "plasma",
];

/// Try to fuzzy-match query words to known terms.
pub fn fuzzy_correct_query(q: &str) -> Option<String> {
    let words: Vec<&str> = q.split_whitespace().collect();
    let mut corrected = false;
    let mut result_words = Vec::new();

    for word in words {
        let mut best_match = word.to_string();
        let mut best_distance = 3;

        if word.len() >= 4 {
            for target in FUZZY_TARGETS {
                let dist = edit_distance(word, target);
                if dist > 0 && dist <= 2 && dist < best_distance {
                    best_match = target.to_string();
                    best_distance = dist;
                    corrected = true;
                }
            }
        }
        result_words.push(best_match);
    }

    if corrected { Some(result_words.join(" ")) } else { None }
}
