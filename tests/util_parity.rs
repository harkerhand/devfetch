use envfetch::util;

#[test]
fn find_version_matches_real_world_cases() {
    let cases: &[(&str, &str)] = &[
        ("apt 1.4.9 (amd64)", "1.4.9"),
        ("cargo 1.31.0 (339d9f9c8 2018-11-16)", "1.31.0"),
        ("go version go1.9.3 darwin/amd64", "1.9.3"),
        ("javac 1.8.0_192-b12", "1.8.0_192-b12"),
        ("postgres (PostgreSQL) 10.3", "10.3"),
        ("rustc 1.31.1 (b6c32da9b 2018-12-18)", "1.31.1"),
        ("3.19.4 2017-08-18 19:28:12", "3.19.4"),
        ("Docker version 18.03.0-ce, build 0520e24", "18.03.0-ce"),
    ];

    for (raw, expected) in cases {
        let got = util::find_version(raw).unwrap_or_default();
        assert_eq!(got, *expected, "failed for case: {raw}");
    }
}

#[test]
fn to_readable_bytes_matches_expected() {
    assert_eq!(util::to_readable_bytes(1337), "1.31 KB");
    assert_eq!(util::to_readable_bytes(0), "0 Bytes");
}

#[test]
fn glob_match_behaves_like_expected_subset() {
    assert!(util::simple_glob_match("*webpack*", "webpack-cli"));
    assert!(util::simple_glob_match("@apollo/*", "@apollo/client"));
    assert!(util::simple_glob_match("react?", "react1"));
    assert!(!util::simple_glob_match("react?", "react"));
}
