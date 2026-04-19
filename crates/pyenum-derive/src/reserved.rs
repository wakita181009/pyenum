//! The forbidden Rust-variant-identifier set for `#[derive(PyEnum)]`.
//!
//! Variants whose identifier matches a Python keyword, an `enum`-module
//! reserved name, or an `enum.EnumType` special method are rejected at
//! compile time. The list is sorted lexicographically so lookups can use
//! `binary_search` — O(log n) and trivial even at 1,024 variants.

/// Category a reserved identifier falls into — surfaced in diagnostics so
/// the error message tells the user *why* the name is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReservedKind {
    /// Python keyword (`class`, `def`, `None`, …).
    PythonKeyword,
    /// Reserved `enum`-module member / class attribute name
    /// (`_name_`, `_value_`, `_missing_`, …, plus `name`, `value`).
    EnumReservedMember,
    /// Special method the `enum.EnumType` metaclass interprets
    /// (`__init__`, `__new__`, `__class__`, …).
    EnumSpecialMethod,
}

/// Python 3.13 keywords. `match` / `case` are soft keywords but surfaced here
/// because we want them rejected (they're commonly confused at read time).
const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "case", "class",
    "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if",
    "import", "in", "is", "lambda", "match", "nonlocal", "not", "or", "pass", "raise", "return",
    "try", "while", "with", "yield",
];

/// Reserved enum-module member names and class attributes.
const ENUM_RESERVED_MEMBERS: &[&str] = &[
    "_generate_next_value_",
    "_ignore_",
    "_missing_",
    "_name_",
    "_order_",
    "_value_",
    "name",
    "value",
];

/// Dunders the `enum.EnumType` metaclass interprets specially. Declaring a
/// variant with one of these identifiers would either collide with the
/// metaclass machinery or shadow a user-visible operator on every member.
const ENUM_SPECIAL_METHODS: &[&str] = &[
    "__bool__",
    "__class__",
    "__class_getitem__",
    "__dir__",
    "__eq__",
    "__format__",
    "__hash__",
    "__init__",
    "__init_subclass__",
    "__members__",
    "__new__",
    "__reduce_ex__",
    "__repr__",
    "__set_name__",
    "__str__",
];

/// Returns the reserved category for `ident`, or `None` if the name is safe.
pub(crate) fn is_reserved(ident: &str) -> Option<ReservedKind> {
    if PYTHON_KEYWORDS.binary_search(&ident).is_ok() {
        return Some(ReservedKind::PythonKeyword);
    }
    if ENUM_RESERVED_MEMBERS.binary_search(&ident).is_ok() {
        return Some(ReservedKind::EnumReservedMember);
    }
    if ENUM_SPECIAL_METHODS.binary_search(&ident).is_ok() {
        return Some(ReservedKind::EnumSpecialMethod);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_keywords_are_sorted() {
        let mut sorted = PYTHON_KEYWORDS.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, PYTHON_KEYWORDS);
    }

    #[test]
    fn enum_reserved_members_are_sorted() {
        let mut sorted = ENUM_RESERVED_MEMBERS.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, ENUM_RESERVED_MEMBERS);
    }

    #[test]
    fn enum_special_methods_are_sorted() {
        let mut sorted = ENUM_SPECIAL_METHODS.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, ENUM_SPECIAL_METHODS);
    }

    #[test]
    fn detects_keywords() {
        assert_eq!(is_reserved("class"), Some(ReservedKind::PythonKeyword));
        assert_eq!(is_reserved("None"), Some(ReservedKind::PythonKeyword));
        assert_eq!(is_reserved("match"), Some(ReservedKind::PythonKeyword));
    }

    #[test]
    fn detects_enum_members() {
        assert_eq!(
            is_reserved("_value_"),
            Some(ReservedKind::EnumReservedMember)
        );
        assert_eq!(is_reserved("name"), Some(ReservedKind::EnumReservedMember));
    }

    #[test]
    fn detects_special_methods() {
        assert_eq!(
            is_reserved("__init__"),
            Some(ReservedKind::EnumSpecialMethod)
        );
        assert_eq!(
            is_reserved("__class__"),
            Some(ReservedKind::EnumSpecialMethod)
        );
    }

    #[test]
    fn allows_normal_identifiers() {
        assert_eq!(is_reserved("Red"), None);
        assert_eq!(is_reserved("HttpOk"), None);
        assert_eq!(is_reserved("MY_CONST"), None);
    }
}
