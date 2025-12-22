use anyhow::{bail, Result};
use rand::{distributions::Uniform, prelude::Distribution, rngs::OsRng};

/// Options for generating a password
#[derive(Debug, Clone)]
pub struct Options {
    pub length: usize,
    pub lowercase: bool,
    pub uppercase: bool,
    pub numbers: bool,
    pub special: bool,
    pub no_ambiguous: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            length: 12,
            lowercase: true,
            uppercase: true,
            numbers: true,
            special: false,
            no_ambiguous: false,
        }
    }
}

const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
const SPECIAL: &str = "!@#%&*-_?"; // removed . , () [] {} per user request
const AMBIGUOUS: &str = "O0Il1"; // O, 0, I, l, 1

/// Generate a password given `Options`.
pub fn generate(opts: &Options) -> Result<String> {
    if opts.length < 6 || opts.length > 25 {
        bail!("length must be between 6 and 25")
    }

    let mut pool = String::new();

    if opts.lowercase {
        pool.push_str(LOWERCASE);
    }
    if opts.uppercase {
        pool.push_str(UPPERCASE);
    }
    if opts.numbers {
        pool.push_str(NUMBERS);
    }
    if opts.special {
        pool.push_str(SPECIAL);
    }

    if pool.is_empty() {
        bail!("no character classes selected")
    }

    if opts.no_ambiguous {
        pool = pool.chars().filter(|c| !AMBIGUOUS.contains(*c)).collect();
    }

    if pool.is_empty() {
        bail!("character pool is empty after applying options")
    }

    let mut rng = OsRng;
    let chars: Vec<char> = pool.chars().collect();

    // Build a non-special set so we can avoid starting the password with a special
    // character when at least one non-special character is available
    let non_special: Vec<char> = chars.iter().cloned().filter(|c| !SPECIAL.contains(*c)).collect();

    let mut password = String::with_capacity(opts.length);

    // First character: prefer a non-special character if available
    if opts.length > 0 {
        if !non_special.is_empty() {
            let between_first = Uniform::from(0..non_special.len());
            let idx = between_first.sample(&mut rng);
            password.push(non_special[idx]);
        } else {
            let between_first = Uniform::from(0..chars.len());
            let idx = between_first.sample(&mut rng);
            password.push(chars[idx]);
        }
    }

    // Remaining characters (if any) can be any from the full pool
    if opts.length > 1 {
        let between = Uniform::from(0..chars.len());
        for _ in 1..opts.length {
            let idx = between.sample(&mut rng);
            password.push(chars[idx]);
        }
    }

    Ok(password)
}

/// Compute the size of the character pool for given options
pub fn pool_size(opts: &Options) -> usize {
    let mut size = 0usize;
    if opts.lowercase {
        let mut c = LOWERCASE.len();
        if opts.no_ambiguous && LOWERCASE.contains('l') {
            c -= 1;
        }
        size += c;
    }
    if opts.uppercase {
        let mut c = UPPERCASE.len();
        if opts.no_ambiguous {
            if UPPERCASE.contains('O') {
                c -= 1;
            }
            if UPPERCASE.contains('I') {
                c -= 1;
            }
        }
        size += c;
    }
    if opts.numbers {
        let mut c = NUMBERS.len();
        if opts.no_ambiguous {
            if NUMBERS.contains('0') {
                c -= 1;
            }
            if NUMBERS.contains('1') {
                c -= 1;
            }
        }
        size += c;
    }
    if opts.special {
        size += SPECIAL.len();
    }
    size
}

/// Estimate entropy in bits for the given options (length * log2(pool_size)).
pub fn estimate_entropy(opts: &Options) -> f32 {
    let pool = pool_size(opts);
    if pool == 0 || opts.length == 0 {
        return 0.0;
    }
    (pool as f32).log2() * opts.length as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_generates_length() {
        let opts = Options::default();
        let p = generate(&opts).unwrap();
        assert_eq!(p.len(), opts.length);
    }

    #[test]
    fn no_ambiguous_removes_chars() {
        let mut opts = Options::default();
        opts.length = 15;
        opts.no_ambiguous = true;
        // create password and ensure none of the ambiguous chars appear
        let p = generate(&opts).unwrap();
        for ch in AMBIGUOUS.chars() {
            assert!(!p.contains(ch));
        }
    }

    #[test]
    fn errors_on_empty_set() {
        let opts = Options {
            length: 10,
            lowercase: false,
            uppercase: false,
            numbers: false,
            special: false,
            no_ambiguous: false,
        };
        assert!(generate(&opts).is_err());
    }

    #[test]
    fn respects_length_bounds() {
        let mut opts = Options::default();
        opts.length = 5; // too short
        assert!(generate(&opts).is_err());
        opts.length = 26; // too long
        assert!(generate(&opts).is_err());
    }

    #[test]
    fn banned_punctuation_not_used_in_special() {
        let mut opts = Options::default();
        opts.length = 20;
        opts.lowercase = false;
        opts.uppercase = false;
        opts.numbers = false;
        opts.special = true;

        let p = generate(&opts).unwrap();
        let banned = ".,()[]{}";
        for ch in banned.chars() {
            assert!(!p.contains(ch), "Password must not include banned char {}", ch);
        }
    }

    #[test]
    fn does_not_start_with_special_when_possible() {
        let mut opts = Options::default();
        opts.length = 12;
        opts.lowercase = true;
        opts.uppercase = false;
        opts.numbers = false;
        opts.special = true;

        let p = generate(&opts).unwrap();
        let first = p.chars().next().unwrap();
        assert!(!SPECIAL.contains(first), "Password should not start with special when non-special chars are available");
    }
}
