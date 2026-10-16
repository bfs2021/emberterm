use super::*;

fn fam(name: &str, mono: bool) -> FontFamily {
    FontFamily {
        name: name.into(),
        monospaced: mono,
    }
}

#[test]
fn one_entry_per_family_sorted_case_insensitively() {
    let out = group_families([
        ("consolas".to_string(), true),
        ("Arial".to_string(), false),
        ("Cascadia Code".to_string(), true),
    ]);
    assert_eq!(
        out,
        vec![
            fam("Arial", false),
            fam("Cascadia Code", true),
            fam("consolas", true)
        ]
    );
}

#[test]
fn faces_of_one_family_merge_and_monospaced_wins() {
    let out = group_families([
        ("JetBrains Mono".to_string(), true),
        ("JetBrains Mono".to_string(), true),
        ("Mixed".to_string(), false),
        ("Mixed".to_string(), true),
    ]);
    assert_eq!(out, vec![fam("JetBrains Mono", true), fam("Mixed", true)]);
}

#[test]
fn hidden_and_vertical_variants_are_skipped() {
    let out = group_families([
        ("@MS Gothic".to_string(), true),
        (".SF Hidden".to_string(), false),
        ("   ".to_string(), false),
        ("Fira Code".to_string(), true),
    ]);
    assert_eq!(out, vec![fam("Fira Code", true)]);
}

#[test]
fn no_fonts_is_an_empty_list() {
    assert!(group_families(Vec::<(String, bool)>::new()).is_empty());
}
