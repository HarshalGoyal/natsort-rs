//! Faithful port of Python's `natsort.utils.path_splitter` (POSIX flavour).

/// Split a path string exactly the way Python's `natsort.utils.path_splitter`
/// does (with the default `treat_base = True`).
///
/// See the CPython `PurePosixPath` semantics this mirrors:
/// - `parts` are split on `/`; a `/` or exactly `//` root is kept as its own
///   component, `.` and empty segments are dropped, `..` is kept;
/// - the last component ("base") has its extensions split off using
///   `PurePath.suffixes`, then `base = base.replace("".join(suffixes), "")`;
/// - an empty or `.` path normalizes to a single `"."` component.
pub fn path_splitter(path: &str) -> Vec<String> {
    let parts = posix_parts(path);

    // *path_parts, base = parts  (PurePath('')/' .  have no parts -> base = ".")
    let (path_parts, mut base) = match parts.split_last() {
        Some((last, rest)) => (rest.to_vec(), last.clone()),
        None => (vec![], ".".to_string()),
    };

    let suffixes = split_suffixes(&base);
    base = base.replace(&suffixes.concat(), "");

    let mut out = path_parts;
    if !base.is_empty() {
        out.push(base);
    }
    out.extend(suffixes);
    out.retain(|c| !c.is_empty());
    out
}

/// The normalized `PurePosixPath.parts` of a path string.
fn posix_parts(path: &str) -> Vec<String> {
    // PurePosixPath('') or PurePosixPath('.') have no parts.
    if path.is_empty() || path == "." {
        return vec![];
    }
    let (root, tail) = splitroot(path);
    let mut parts = Vec::new();
    if !root.is_empty() {
        parts.push(root.to_string());
    }
    for seg in tail.split('/') {
        // PurePath drops empty and "." segments.
        if seg.is_empty() || seg == "." {
            continue;
        }
        parts.push(seg.to_string());
    }
    parts
}

/// Port of the extension-splitting loop in `natsort.path_splitter`.
///
/// Returns the list of suffixes to strip. Uses CPython's `PurePath.suffixes`:
/// `name = s; if name.endswith('.'): [] else ['.'+p for p in name.lstrip('.')
/// .split('.')[1:]]`, then walks the reversed list stopping on the first
/// suffix that matches `\.\d` (decimal), exceeds two suffixes, or exceeds
/// five characters.
fn split_suffixes(base: &str) -> Vec<String> {
    if base.ends_with('.') {
        return Vec::new();
    }
    let name = base.trim_start_matches('.');
    let mut candidates: Vec<String> = name.split('.').skip(1).map(|s| format!(".{s}")).collect();
    candidates.reverse();

    let mut accepted = Vec::new();
    let threshold = 5;
    for (i, suffix) in candidates.iter().enumerate() {
        let is_decimal = suffix.len() > 1 && suffix.as_bytes()[1].is_ascii_digit();
        if is_decimal || i > 1 || suffix.chars().count() > threshold {
            break;
        }
        accepted.push(suffix.clone());
    }
    accepted.reverse();
    accepted
}

/// Port of `PurePosixPath._splitroot`: `(root, tail)`.
///
/// - relative path: `("", path)`
/// - one slash `/x`: `("/", "x")`
/// - exactly two slashes `//x`: `("//", "x")`
/// - three+ slashes `///x`: `("/", "//x")`
fn splitroot(p: &str) -> (&str, &str) {
    if !p.starts_with('/') {
        ("", p)
    } else if p[1..].starts_with('/') && !p[2..].starts_with('/') {
        // exactly two leading slashes
        ("//", &p[2..])
    } else {
        ("/", &p[1..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_dot() {
        assert_eq!(path_splitter(""), vec!["."]);
        assert_eq!(path_splitter("."), vec!["."]);
    }

    #[test]
    fn root_only() {
        assert_eq!(path_splitter("/"), vec!["/"]);
        assert_eq!(path_splitter("//"), vec!["//"]);
    }

    #[test]
    fn root_kept() {
        assert_eq!(path_splitter("/a"), vec!["/", "a"]);
        assert_eq!(path_splitter("/a/b"), vec!["/", "a", "b"]);
        assert_eq!(path_splitter("//server"), vec!["//", "server"]);
    }

    #[test]
    fn collapses_slashes() {
        assert_eq!(path_splitter("a//b"), vec!["a", "b"]);
        assert_eq!(path_splitter("a///b"), vec!["a", "b"]);
    }

    #[test]
    fn dot_components_dropped() {
        assert_eq!(path_splitter("a/./b"), vec!["a", "b"]);
        assert_eq!(path_splitter("./a"), vec!["a"]);
        assert_eq!(path_splitter("a/."), vec!["a"]);
    }

    #[test]
    fn dotdot_kept() {
        assert_eq!(path_splitter("a/../b"), vec!["a", "..", "b"]);
        assert_eq!(path_splitter("a/.."), vec!["a", ".."]);
        assert_eq!(path_splitter(".."), vec![".."]);
    }

    #[test]
    fn trailing_slash_dropped() {
        assert_eq!(path_splitter("a/"), vec!["a"]);
    }

    #[test]
    fn simple() {
        assert_eq!(path_splitter("a"), vec!["a"]);
        assert_eq!(path_splitter("a/b/c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn extensions_split() {
        assert_eq!(path_splitter("b.tar.gz"), vec!["b", ".tar", ".gz"]);
        assert_eq!(path_splitter("file.txt"), vec!["file", ".txt"]);
    }

    #[test]
    fn max_two_suffixes() {
        assert_eq!(path_splitter("a.b.c.d"), vec!["a.b", ".c", ".d"]);
    }

    #[test]
    fn oversize_suffix_not_split() {
        assert_eq!(path_splitter("file.abcdef"), vec!["file.abcdef"]);
    }

    #[test]
    fn decimal_named_no_split() {
        assert_eq!(path_splitter("v1.0.0"), vec!["v1.0.0"]);
    }

    #[test]
    fn dot_ending_no_ext() {
        assert_eq!(path_splitter("file."), vec!["file."]);
        assert_eq!(path_splitter(".gitconfig"), vec![".gitconfig"]);
    }

    #[test]
    fn double_dot_emits_dot_suffix() {
        // CPython suffixes() yields ".", ".gz" for "a..gz".
        assert_eq!(path_splitter("a..gz"), vec!["a", ".", ".gz"]);
        assert_eq!(path_splitter("foo..bar"), vec!["foo", ".", ".bar"]);
    }

    #[test]
    fn repeated_suffix_replaced_everywhere() {
        assert_eq!(path_splitter("x.gz.tar.gz"), vec!["x.gz", ".tar", ".gz"]);
    }
}
