//! The `url` feature: `Url` is a leaf.

use typesift::TypeSift;
use url::Url;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Page {
    canonical: Url,
    links: Vec<Url>,
    referrer: Option<Url>,
    title: String,
}

fn page() -> Page {
    Page {
        canonical: Url::parse("https://example.test/a").expect("a valid url"),
        links: vec![
            Url::parse("https://example.test/b").expect("a valid url"),
            Url::parse("https://example.test/c").expect("a valid url"),
        ],
        referrer: None,
        title: "A page".to_string(),
    }
}

#[test]
fn urls_are_found_wherever_they_are_nested() {
    let page = page();
    assert_same_refs(
        &page.sift::<Url>(),
        &[&page.canonical, &page.links[0], &page.links[1]],
    );
}

#[test]
fn url_is_a_leaf() {
    let page = page();
    // A `Url` holds the string it was parsed from; only the title is found.
    assert_eq!(page.sift::<String>(), ["A page"]);
    assert!(page.sift::<u32>().is_empty());
}

#[test]
fn entry_points_agree_for_url() {
    check_consistency::<_, Url>(&page());
    check_consistency::<_, Option<Url>>(&page());
    check_consistency::<_, Page>(&page());
}
