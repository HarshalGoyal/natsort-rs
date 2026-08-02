//! OS path sorting: mimics file explorer / Finder sort order.
//!
//! On Unix, this uses `ns.LOCALE | ns.PATH | ns.IGNORECASE` as a fallback
/// when ICU is not available.  On Windows, it would use `StrCmpLogicalW`.

use crate::keygen::NatsortKey;
use crate::ns::NsFlags;
use crate::segment::NatsortKeyPart;

/// Generate a sort key for an OS path.
///
/// Splits the path by directory separators (`/` or `\`), applies natural sort
/// to each component, and also splits off file extensions so that
/// `file(1).txt` sorts before `file(10).txt`.
///
/// Returns a vector of component keys, where each component key is itself a
/// vector of [`NatsortKeyPart`] elements. This preserves the tuple structure
/// that Python's nested-tuple comparison relies on.
pub fn os_sort_key(path: &str) -> Vec<Vec<NatsortKeyPart>> {
    let components = split_path_components(path);
    let mut component_keys = Vec::new();

    for component in components {
        let kg = NatsortKey::new(NsFlags::LOCALE | NsFlags::PATH | NsFlags::IGNORECASE);
        let parts = kg.key(component);
        component_keys.push(parts);
    }

    component_keys
}

/// Compare two OS sort keys component-by-component.
fn os_key_cmp(a: &[Vec<NatsortKeyPart>], b: &[Vec<NatsortKeyPart>]) -> core::cmp::Ordering {
    for (ca, cb) in a.iter().zip(b.iter()) {
        match ca.cmp(cb) {
            core::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    // All compared components equal: fewer components sorts first.
    a.len().cmp(&b.len())
}

/// Split a path into its directory components and extension.
///
/// For `"dir/file(10).txt"` returns `["dir", "file(10)", ".txt"]`.
/// Mirrors Python's `path_splitter` with `treat_base = true`.
fn split_path_components(path: &str) -> Vec<&str> {
    let mut components = Vec::new();

    // Split by path separators.
    for component in path.split(['/', '\\']) {
        if component.is_empty() {
            continue;
        }

        // Try to extract file extension(s).
        // Python logic: split off extensions until we hit a decimal-number suffix,
        // more than two suffixes, or a suffix longer than 5 chars.
        let (base, exts) = split_extension(component);
        components.push(base);
        for ext in exts {
            components.push(ext);
        }
    }

    components
}

/// Split a filename into base and extension parts.
///
/// Extracts up to two short suffixes (≤ 5 chars including leading dot) that
/// don't start with a digit after the dot.
fn split_extension(filename: &str) -> (&str, Vec<&str>) {
    let mut exts = Vec::new();
    let mut remaining = filename;

    // Find all suffixes.
    while let Some(dot_pos) = remaining.rfind('.') {
        if dot_pos == 0 {
            break; // Don't treat leading dot as extension.
        }

        let suffix = &remaining[dot_pos..];
        if suffix.len() > 5 {
            break; // Too long.
        }

        // Check if it looks like a decimal number (e.g., ".5" or ".10").
        let after_dot = &suffix[1..];
        if after_dot.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            break; // Decimal number suffix → stop.
        }

        exts.push(suffix);
        remaining = &remaining[..dot_pos];

        if exts.len() >= 2 {
            break; // Max two extensions.
        }
    }

    // Reverse to get left-to-right order.
    exts.reverse();
    (remaining, exts)
}

// ---- Public API ------------------------------------------------

/// Sort paths in the same order as a file explorer.
///
/// Uses locale-aware, case-insensitive sorting with path component awareness.
/// File extensions are separated from basenames so that `file(1).txt` sorts
/// before `file(10).txt`.
///
/// # Examples
///
/// ```
/// use natsort::os_sorted;
///
/// let paths = vec![
///     "/dir/file10.txt",
///     "/dir/file2.txt",
///     "/dir/file1.txt",
/// ];
/// let sorted = os_sorted(&paths);
/// assert_eq!(sorted, vec![
///     "/dir/file1.txt",
///     "/dir/file2.txt",
///     "/dir/file10.txt",
/// ]);
/// ```
pub fn os_sorted(items: &[&str]) -> Vec<String> {
    let mut indexed: Vec<_> = items.iter().enumerate().collect();
    indexed.sort_by(|&(_, &a), &(_, &b)| {
        os_key_cmp(&os_sort_key(a), &os_sort_key(b))
    });
    indexed.into_iter().map(|(_, item)| item.to_string()).collect()
}

/// Generate a reusable OS sort key function.
///
/// Returns a closure that can be passed to `sort_by` or similar.
/// Equivalent to Python's `os_sort_keygen`.
pub fn os_sort_keygen() -> impl Fn(&str) -> Vec<Vec<NatsortKeyPart>> {
    |path: &str| os_sort_key(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_sort_basic() {
        let data = vec!["/dir/file10.txt", "/dir/file2.txt", "/dir/file1.txt"];
        let result = os_sorted(&data);
        assert_eq!(result, vec![
            "/dir/file1.txt",
            "/dir/file2.txt",
            "/dir/file10.txt",
        ]);
    }

    #[test]
    fn os_sort_extension_handling() {
        let data = vec!["file(10).txt", "file(2).txt", "file(1).txt"];
        let result = os_sorted(&data);
        assert_eq!(result, vec![
            "file(1).txt",
            "file(2).txt",
            "file(10).txt",
        ]);
    }

    #[test]
    fn os_sort_case_insensitive() {
        let data = vec!["/Dir/B.txt", "/dir/a.txt"];
        let result = os_sorted(&data);
        // Both have same dir component, case-insensitive.
        assert_eq!(result, vec![
            "/dir/a.txt",
            "/Dir/B.txt",
        ]);
    }

    #[test]
    fn os_sort_multiple_extensions() {
        let data = vec!["file.tar.gz", "file2.tar.gz"];
        let result = os_sorted(&data);
        assert_eq!(result, vec![
            "file.tar.gz",
            "file2.tar.gz",
        ]);
    }

    #[test]
    fn split_extension_single() {
        let (base, exts) = split_extension("file.txt");
        assert_eq!(base, "file");
        assert_eq!(exts, vec![".txt"]);
    }

    #[test]
    fn split_extension_double() {
        let (base, exts) = split_extension("file.tar.gz");
        assert_eq!(base, "file");
        assert_eq!(exts, vec![".tar", ".gz"]);
    }

    #[test]
    fn split_extension_decimal_stop() {
        let (base, exts) = split_extension("file.5txt");
        // ".5txt" starts with digit after dot → stop.
        assert_eq!(base, "file.5txt");
        assert!(exts.is_empty());
    }

    #[test]
    fn split_extension_too_long() {
        let (base, exts) = split_extension("file.abcdefgh");
        // ".abcdefgh" is 9 chars (> 5) → stop.
        assert_eq!(base, "file.abcdefgh");
        assert!(exts.is_empty());
    }

    #[test]
    fn test_split_path_components() {
        let comps = split_path_components("/dir/file(10).txt");
        assert_eq!(comps, vec!["dir", "file(10)", ".txt"]);
    }

    #[test]
    fn os_sort_keygen_returns_fn() {
        let keygen = os_sort_keygen();
        let key = keygen("/dir/file2.txt");
        assert!(!key.is_empty());
    }
}
