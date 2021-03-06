use super::*;

#[test]
fn candidate_from_line_ok_short() {
    fn helper(line: &str) {
        assert_eq!(
            Candidate::from_line(line).unwrap(),
            Candidate {
                inner: CandidateInner::from_line(line).unwrap()
            }
        );
    }

    // Testing spacing.
    helper("[name](mod1)");
    helper("  [name](mod1)");
    helper("\t[name](mod1)");
    helper("/// [name](mod1)");
    helper("///  [name](mod1)");
    helper("///\t[name](mod1)");
    helper(" /// [name](mod1)");
    helper("  /// [name](mod1)");
    helper("\t/// [name](mod1)");

    // Testing bangs.
    helper("/// [name](mod1)");
    helper("//! [name](mod1)");

    // Testing code in links.
    helper("[`name`](mod1)");

    // Testing non-HTTP links
    helper("[name](mod1/)");
    helper("[name](mod1/mod2)");
    helper("[name](mod1/mod2/struct.Type.html)");
    helper("[name](mod1/mod2/struct.Type.html#const.NAME)");
    helper("[name](mod1/mod2/#section)");
    helper("[name](mod1/mod2#section)");
    helper("[name](mod1/mod2/index.html#section)");
    helper("[name](mod1/mod2/index.html/#section)");
    helper("[name](../mod1/mod2)");
    helper("[name](./../mod1)");

    // Testing HTTP links
    helper("[name](https://docs.rs/regex/)");
    helper("[name](https://docs.rs/regex/1.0.33/regex/mod1/mod2)");
    helper("[name](https://github.com/poliorcetics/cargo-intraconv/issues/21)");

    // Testing all sorts of characters in the link name.
    helper("[azertyuiop](mod1)");
    helper("[AZERTYUIOP](mod1)");
    helper("[@&é\"'(§è!çà)-](mod1)");
    helper("[#1234567890°_](mod1)");
    helper("[•ë“‘{¶«¡Çø}—](mod1)");
    helper("[Ÿ´„”’[å»ÛÁØ]–](mod1)");
    helper("[æÂê®†Úºîœπ](mod1)");
    helper("[ÆÅÊ‚™ŸªïŒ∏](mod1)");
    helper("[‡Ò∂ƒﬁÌÏÈ¬µ](mod1)");
    helper("[Ω∑∆·ﬂÎÍË|Ó](mod1)");
    helper("[‹≈©◊ß~](mod1)");
    helper("[›⁄¢√∫ı](mod1)");
    helper("[^$ù`,;=<](mod1)");
    helper("[¨*%£?./+>](mod1)");
    helper("[ô€Ù@∞…÷≠≤](mod1)");
    helper("[Ô¥‰#¿•\\±≥](mod1)");
}

#[test]
fn candidate_from_line_ok_long() {
    fn helper(line: &str) {
        assert_eq!(
            Candidate::from_line(line).unwrap(),
            Candidate {
                inner: CandidateInner::from_line(line).unwrap()
            }
        );
    }

    // Testing spacing.
    helper("[name]:mod1");
    helper("[name]: mod1");
    helper("[name]:  mod1");
    helper("[name]:\tmod1");
    helper("  [name]: mod1");
    helper("\t[name]: mod1");
    helper("/// [name]: mod1");
    helper("///  [name]: mod1");
    helper("///\t[name]: mod1");
    helper(" /// [name]: mod1");
    helper("  /// [name]: mod1");
    helper("\t/// [name]: mod1");

    // Testing bangs.
    helper("/// [name]: mod1");
    helper("//! [name]: mod1");

    // Testing code in links.
    helper("[`name`]: mod1");

    // Testing non-HTTP links
    helper("[name]: mod1/");
    helper("[name]: mod1/mod2");
    helper("[name]: mod1/mod2/struct.Type.html");
    helper("[name]: mod1/mod2/struct.Type.html#const.NAME");
    helper("[name]: mod1/mod2/#section");
    helper("[name]: mod1/mod2#section");
    helper("[name]: mod1/mod2/index.html#section");
    helper("[name]: mod1/mod2/index.html/#section");
    helper("[name]: ../mod1/mod2");
    helper("[name]: ./../mod1");

    // Testing HTTP links
    helper("[name]: https://docs.rs/regex/");
    helper("[name]: https://docs.rs/regex/1.0.33/regex/mod1/mod2");
    helper("[name]: https://github.com/poliorcetics/cargo-intraconv/issues/21");

    // Testing all sorts of characters in the link name.
    helper("[azertyuiop]: mod1");
    helper("[AZERTYUIOP]: mod1");
    helper("[@&é\"'(§è!çà)-]: mod1");
    helper("[#1234567890°_]: mod1");
    helper("[•ë“‘{¶«¡Çø}—]: mod1");
    helper("[Ÿ´„”’[å»ÛÁØ]–]: mod1");
    helper("[æÂê®†Úºîœπ]: mod1");
    helper("[ÆÅÊ‚™ŸªïŒ∏]: mod1");
    helper("[‡Ò∂ƒﬁÌÏÈ¬µ]: mod1");
    helper("[Ω∑∆·ﬂÎÍË|Ó]: mod1");
    helper("[‹≈©◊ß~]: mod1");
    helper("[›⁄¢√∫ı]: mod1");
    helper("[^$ù`,;:=<]: mod1");
    helper("[¨*%£?./+>]: mod1");
    helper("[ô€Ù@∞…÷≠≤]: mod1");
    helper("[Ô¥‰#¿•\\±≥]: mod1");
}

#[test]
fn candidate_from_line_err() {
    fn helper(line: &str) {
        assert!(Candidate::from_line(line).is_none());
    }

    helper("not a link");
    helper("[name]: /absolute/path");
    helper("[name]: intra::doc::link");
}

#[test]
fn candidate_transform() {
    use crate::ConversionContext;

    fn check_transform(value: &str, target: &str, ctx: &ConversionContext) {
        let candidate = Candidate::from_line(value).unwrap();
        let candidate_clone = Candidate::from_line(value).unwrap();
        let transform = candidate.transform(&ctx).unwrap();
        assert_eq!(
            target, transform,
            "\n--> Value: {:#?}, candidate: {:#?}",
            value, candidate_clone
        );
    }

    // Both contexts can transform favored links, for a context that cannot
    // see `test_link_parts`.
    let mut ctx_dis = ConversionContext::with_options(crate::OPTS_KRATE_DIS_AND_FAV.clone());
    let mut ctx_no_dis = ConversionContext::with_options(crate::OPTS_KRATE_NO_DIS_BUT_FAV.clone());

    // Ensure sections and associated items are not transformed when the
    // current type block is empty.
    check_transform("[`Link`]: #section", "[`Link`]: #section", &ctx_dis);
    check_transform("[`Link`]: #section", "[`Link`]: #section", &ctx_no_dis);

    check_transform(
        "[`Link`]: #method.drain",
        "[`Link`]: Self::drain()",
        &ctx_dis,
    );
    check_transform(
        "[`Link`]: #method.drain",
        "[`Link`]: Self::drain()",
        &ctx_no_dis,
    );

    check_transform("[`Link`](#section)", "[`Link`](#section)", &ctx_dis);
    check_transform("[`Link`](#section)", "[`Link`](#section)", &ctx_no_dis);

    check_transform(
        "[`Link`](#method.drain)",
        "[`Link`](Self::drain())",
        &ctx_dis,
    );
    check_transform(
        "[`Link`](#method.drain)",
        "[`Link`](Self::drain())",
        &ctx_no_dis,
    );

    check_transform(
        "[`Link`](struct.Link.html)",
        "[`Link`](type@Link)",
        &ctx_dis,
    );
    check_transform("[`Link`](struct.Link.html)", "[`Link`]", &ctx_no_dis);

    check_transform(
        "See [a()](struct.A.html#method.a) and [b](struct.B.html#struct.b) for more informations",
        "See [a()](A::a()) and [b](type@B::b) for more informations",
        &ctx_dis,
    );
    check_transform(
        "See [a()](struct.A.html#method.a) and [b](struct.B.html#struct.b) for more informations",
        "See [a()](A::a()) and [b](B::b) for more informations",
        &ctx_no_dis,
    );

    ctx_dis.set_current_type_block("Block".into());
    ctx_no_dis.set_current_type_block("Block".into());

    for &(value, with_dis, without_dis) in VALID_LONG_LINKS {
        check_transform(value, with_dis, &ctx_dis);
        check_transform(value, without_dis, &ctx_no_dis);
    }

    for &(value, with_dis, without_dis) in VALID_SHORT_LINKS {
        check_transform(value, with_dis, &ctx_dis);
        check_transform(value, without_dis, &ctx_no_dis);
    }
}

#[test]
fn do_not_transform_primitive() {
    use crate::ConversionContext;

    let long_primitive_links = [
        "[`Link`]: primitive.Type.html",
        "[`Link`]: primitive.Type.html#method.call",
        "[`Link`]: primitive.Type.html#section-name",
        "[`Link`]: ./primitive.Type.html",
        "[`Link`]: ../primitive.Type.html",
        "[`Link`]: ../mod1/mod2/primitive.Type.html",
    ];

    let short_primitive_links = [
        "[`Link`](primitive.Type.html)",
        "[`Link`](primitive.Type.html#method.call)",
        "[`Link`](primitive.Type.html#section-name)",
        "[`Link`](./primitive.Type.html)",
        "[`Link`](../primitive.Type.html)",
        "[`Link`](../mod1/mod2/primitive.Type.html)",
    ];

    // Both contexts can transform favored links, for a context that cannot
    // see `test_link_parts`.
    let ctx_dis = ConversionContext::with_options(crate::OPTS_KRATE_DIS_AND_FAV.clone());
    let ctx_no_dis = ConversionContext::with_options(crate::OPTS_KRATE_NO_DIS_BUT_FAV.clone());

    for value in long_primitive_links.iter() {
        let candidate = Candidate::from_line(value).unwrap();
        assert!(candidate.transform(&ctx_dis).is_none());
        let candidate = Candidate::from_line(value).unwrap();
        assert!(candidate.transform(&ctx_no_dis).is_none());
    }

    for &value in short_primitive_links.iter() {
        let candidate = Candidate::from_line(value).unwrap();
        assert_eq!(value, candidate.transform(&ctx_dis).unwrap());
        let candidate = Candidate::from_line(value).unwrap();
        assert_eq!(value, candidate.transform(&ctx_no_dis).unwrap());
    }
}

#[cfg(test)]
const VALID_LONG_LINKS: &[(&str, &str, &str)] = &[
    (
        "[`Link`]: https://docs.rs/krate-name/1.2.3/krate/struct.Type.html",
        "[`Link`]: type@crate::Type",
        "[`Link`]: crate::Type",
    ),
    (
        "[`Link`]: https://docs.rs/regex/",
        "[`Link`]: mod@regex",
        "[`Link`]: regex",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2",
        "[`Link`]: mod@regex",
        "[`Link`]: regex",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex",
        "[`Link`]: mod@regex",
        "[`Link`]: regex",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/struct.Regex.html",
        "[`Link`]: type@regex::Regex",
        "[`Link`]: regex::Regex",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples",
        "[`Link`]: type@regex::Regex#examples",
        "[`Link`]: regex::Regex#examples",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/struct.Regex.html#method.is_match",
        "[`Link`]: regex::Regex::is_match()",
        "[`Link`]: regex::Regex::is_match()",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/bytes/index.html",
        "[`Link`]: mod@regex::bytes",
        "[`Link`]: regex::bytes",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/bytes/index.html#syntax",
        "[`Link`]: regex::bytes#syntax",
        "[`Link`]: regex::bytes#syntax",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples",
        "[`Link`]: type@regex::bytes::Regex#examples",
        "[`Link`]: regex::bytes::Regex#examples",
    ),
    (
        "[`Link`]: https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#method.is_match",
        "[`Link`]: regex::bytes::Regex::is_match()",
        "[`Link`]: regex::bytes::Regex::is_match()",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/std",
        "[`Link`]: mod@std",
        "[`Link`]: std",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/alloc",
        "[`Link`]: mod@alloc",
        "[`Link`]: alloc",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/core",
        "[`Link`]: mod@core",
        "[`Link`]: core",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/test",
        "[`Link`]: mod@test",
        "[`Link`]: test",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/proc_macro",
        "[`Link`]: mod@proc_macro",
        "[`Link`]: proc_macro",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/std/string/index.html",
        "[`Link`]: mod@std::string",
        "[`Link`]: std::string",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/std/string/struct.String.html",
        "[`Link`]: type@std::string::String",
        "[`Link`]: std::string::String",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/std/string/struct.String.html#examples",
        "[`Link`]: type@std::string::String#examples",
        "[`Link`]: std::string::String#examples",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/std/string/struct.String.html#method.drain",
        "[`Link`]: std::string::String::drain()",
        "[`Link`]: std::string::String::drain()",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/std",
        "[`Link`]: mod@std",
        "[`Link`]: std",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/alloc",
        "[`Link`]: mod@alloc",
        "[`Link`]: alloc",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/core",
        "[`Link`]: mod@core",
        "[`Link`]: core",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/test",
        "[`Link`]: mod@test",
        "[`Link`]: test",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/proc_macro",
        "[`Link`]: mod@proc_macro",
        "[`Link`]: proc_macro",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/std/string/index.html",
        "[`Link`]: mod@std::string",
        "[`Link`]: std::string",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/std/string/struct.String.html",
        "[`Link`]: type@std::string::String",
        "[`Link`]: std::string::String",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples",
        "[`Link`]: type@std::string::String#examples",
        "[`Link`]: std::string::String#examples",
    ),
    (
        "[`Link`]: https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain",
        "[`Link`]: std::string::String::drain()",
        "[`Link`]: std::string::String::drain()",
    ),
    (
        "[`Link`]: #struct.Item",
        "[`Link`]: type@Block::Item",
        "[`Link`]: Block::Item",
    ),
    (
        "[`Link`]: ./#struct.Item",
        "[`Link`]: type@Block::Item",
        "[`Link`]: Block::Item",
    ),
    (
        "[`Link`]: ././#struct.Item",
        "[`Link`]: type@Block::Item",
        "[`Link`]: Block::Item",
    ),
    (
        "[`Link`]: #section-a",
        "[`Link`]: #section-a",
        "[`Link`]: #section-a",
    ),
    (
        "[`Link`]: #section-1",
        "[`Link`]: #section-1",
        "[`Link`]: #section-1",
    ),
    (
        "[`Link`]: #section-A",
        "[`Link`]: #section-A",
        "[`Link`]: #section-A",
    ),
    (
        "[`Link`]: #section_a",
        "[`Link`]: #section_a",
        "[`Link`]: #section_a",
    ),
    (
        "[`Link`]: #section.a",
        "[`Link`]: #section.a",
        "[`Link`]: #section.a",
    ),
    (
        "[`Link`]: #Section.a",
        "[`Link`]: #Section.a",
        "[`Link`]: #Section.a",
    ),
    (
        "[`Link`]: #rection.a",
        "[`Link`]: #rection.a",
        "[`Link`]: #rection.a",
    ),
    (
        "[`Link`]: #0ection.a",
        "[`Link`]: #0ection.a",
        "[`Link`]: #0ection.a",
    ),
    (
        "[`Link`]: #_ection.a",
        "[`Link`]: #_ection.a",
        "[`Link`]: #_ection.a",
    ),
    (
        "[`Link`]: krate/#section",
        "[`Link`]: crate#section",
        "[`Link`]: crate#section",
    ),
    (
        "[`Link`]: ../krate/#section",
        "[`Link`]: crate#section",
        "[`Link`]: crate#section",
    ),
    (
        "[`Link`]: mod1/#section",
        "[`Link`]: mod1#section",
        "[`Link`]: mod1#section",
    ),
    (
        "[`Link`]: mod1/mod2/#section",
        "[`Link`]: mod1::mod2#section",
        "[`Link`]: mod1::mod2#section",
    ),
    (
        "[`Link`]: ../../mod1/mod2/#section",
        "[`Link`]: super::super::mod1::mod2#section",
        "[`Link`]: super::super::mod1::mod2#section",
    ),
    (
        "[`Link`]: associatedconstant.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: associatedconstant.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: associatedconstant.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./associatedconstant.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../associatedconstant.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/associatedconstant.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: associatedtype.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: associatedtype.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: associatedtype.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./associatedtype.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../associatedtype.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/associatedtype.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: attr.Type.html",
        "[`Link`]: macro@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: attr.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: attr.Type.html#section-name",
        "[`Link`]: macro@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./attr.Type.html",
        "[`Link`]: macro@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../attr.Type.html",
        "[`Link`]: macro@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/attr.Type.html",
        "[`Link`]: macro@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: constant.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: constant.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: constant.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./constant.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../constant.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/constant.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: derive.Type.html",
        "[`Link`]: macro@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: derive.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: derive.Type.html#section-name",
        "[`Link`]: macro@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./derive.Type.html",
        "[`Link`]: macro@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../derive.Type.html",
        "[`Link`]: macro@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/derive.Type.html",
        "[`Link`]: macro@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: enum.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: enum.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: enum.Type.html#section-name",
        "[`Link`]: type@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./enum.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../enum.Type.html",
        "[`Link`]: type@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/enum.Type.html",
        "[`Link`]: type@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: externcrate.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: externcrate.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: externcrate.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./externcrate.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../externcrate.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/externcrate.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: fn.Type.html",
        "[`Link`]: Type()",
        "[`Link`]: Type()",
    ),
    (
        "[`Link`]: fn.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: fn.Type.html#section-name",
        "[`Link`]: Type()#section-name",
        "[`Link`]: Type()#section-name",
    ),
    (
        "[`Link`]: ./fn.Type.html",
        "[`Link`]: Type()",
        "[`Link`]: Type()",
    ),
    (
        "[`Link`]: ../fn.Type.html",
        "[`Link`]: super::Type()",
        "[`Link`]: super::Type()",
    ),
    (
        "[`Link`]: ../mod1/mod2/fn.Type.html",
        "[`Link`]: super::mod1::mod2::Type()",
        "[`Link`]: super::mod1::mod2::Type()",
    ),
    (
        "[`Link`]: foreigntype.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: foreigntype.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: foreigntype.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./foreigntype.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../foreigntype.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/foreigntype.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: impl.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: impl.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: impl.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./impl.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../impl.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/impl.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: import.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: import.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: import.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./import.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../import.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/import.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: keyword.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: keyword.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: keyword.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./keyword.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../keyword.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/keyword.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: macro.Type.html",
        "[`Link`]: Type!",
        "[`Link`]: Type!",
    ),
    (
        "[`Link`]: macro.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: macro.Type.html#section-name",
        "[`Link`]: Type!#section-name",
        "[`Link`]: Type!#section-name",
    ),
    (
        "[`Link`]: ./macro.Type.html",
        "[`Link`]: Type!",
        "[`Link`]: Type!",
    ),
    (
        "[`Link`]: ../macro.Type.html",
        "[`Link`]: super::Type!",
        "[`Link`]: super::Type!",
    ),
    (
        "[`Link`]: ../mod1/mod2/macro.Type.html",
        "[`Link`]: super::mod1::mod2::Type!",
        "[`Link`]: super::mod1::mod2::Type!",
    ),
    (
        "[`Link`]: method.Type.html",
        "[`Link`]: Type()",
        "[`Link`]: Type()",
    ),
    (
        "[`Link`]: method.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: method.Type.html#section-name",
        "[`Link`]: Type()#section-name",
        "[`Link`]: Type()#section-name",
    ),
    (
        "[`Link`]: ./method.Type.html",
        "[`Link`]: Type()",
        "[`Link`]: Type()",
    ),
    (
        "[`Link`]: ../method.Type.html",
        "[`Link`]: super::Type()",
        "[`Link`]: super::Type()",
    ),
    (
        "[`Link`]: ../mod1/mod2/method.Type.html",
        "[`Link`]: super::mod1::mod2::Type()",
        "[`Link`]: super::mod1::mod2::Type()",
    ),
    (
        "[`Link`]: mod.Type.html",
        "[`Link`]: mod@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: mod.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: mod.Type.html#section-name",
        "[`Link`]: mod@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./mod.Type.html",
        "[`Link`]: mod@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../mod.Type.html",
        "[`Link`]: mod@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/mod.Type.html",
        "[`Link`]: mod@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: opaque.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: opaque.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: opaque.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./opaque.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../opaque.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/opaque.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: static.Type.html",
        "[`Link`]: value@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: static.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: static.Type.html#section-name",
        "[`Link`]: value@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./static.Type.html",
        "[`Link`]: value@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../static.Type.html",
        "[`Link`]: value@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/static.Type.html",
        "[`Link`]: value@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: struct.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: struct.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: struct.Type.html#section-name",
        "[`Link`]: type@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./struct.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../struct.Type.html",
        "[`Link`]: type@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/struct.Type.html",
        "[`Link`]: type@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: structfield.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: structfield.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: structfield.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./structfield.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../structfield.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/structfield.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: trait.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: trait.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: trait.Type.html#section-name",
        "[`Link`]: type@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./trait.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../trait.Type.html",
        "[`Link`]: type@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/trait.Type.html",
        "[`Link`]: type@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: traitalias.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: traitalias.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: traitalias.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./traitalias.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../traitalias.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/traitalias.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: tymethod.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: tymethod.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: tymethod.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./tymethod.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../tymethod.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/tymethod.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: type.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: type.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: type.Type.html#section-name",
        "[`Link`]: type@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./type.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../type.Type.html",
        "[`Link`]: type@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/type.Type.html",
        "[`Link`]: type@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: union.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: union.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: union.Type.html#section-name",
        "[`Link`]: type@Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./union.Type.html",
        "[`Link`]: type@Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../union.Type.html",
        "[`Link`]: type@super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/union.Type.html",
        "[`Link`]: type@super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    (
        "[`Link`]: variant.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: variant.Type.html#method.call",
        "[`Link`]: Type::call()",
        "[`Link`]: Type::call()",
    ),
    (
        "[`Link`]: variant.Type.html#section-name",
        "[`Link`]: Type#section-name",
        "[`Link`]: Type#section-name",
    ),
    (
        "[`Link`]: ./variant.Type.html",
        "[`Link`]: Type",
        "[`Link`]: Type",
    ),
    (
        "[`Link`]: ../variant.Type.html",
        "[`Link`]: super::Type",
        "[`Link`]: super::Type",
    ),
    (
        "[`Link`]: ../mod1/mod2/variant.Type.html",
        "[`Link`]: super::mod1::mod2::Type",
        "[`Link`]: super::mod1::mod2::Type",
    ),
    ("[`Link`]: regex", "[`Link`]: mod@regex", "[`Link`]: regex"),
    (
        "[`Link`]: ../../regex",
        "[`Link`]: mod@super::super::regex",
        "[`Link`]: super::super::regex",
    ),
    (
        "[`Link`]: ../../mod1/mod2/regex",
        "[`Link`]: mod@super::super::mod1::mod2::regex",
        "[`Link`]: super::super::mod1::mod2::regex",
    ),
    (
        "[`Link`]: mod1/mod2",
        "[`Link`]: mod@mod1::mod2",
        "[`Link`]: mod1::mod2",
    ),
    (
        "[`Link`]: ../../krate/mod1/mod2/regex",
        "[`Link`]: mod@crate::mod1::mod2::regex",
        "[`Link`]: crate::mod1::mod2::regex",
    ),
    (
        "[`Link`]: mod1/mod2",
        "[`Link`]: mod@mod1::mod2",
        "[`Link`]: mod1::mod2",
    ),
    (
        "[`Link`]: regex/bytes/index.html",
        "[`Link`]: mod@regex::bytes",
        "[`Link`]: regex::bytes",
    ),
    (
        "[`Link`]: regex/bytes/index.html#syntax",
        "[`Link`]: regex::bytes#syntax",
        "[`Link`]: regex::bytes#syntax",
    ),
];

#[cfg(test)]
const VALID_SHORT_LINKS: &[(&str, &str, &str)] = &[
    (
        "[`Link`](https://docs.rs/krate-name/1.2.3/krate/struct.Type.html)",
        "[`Link`](type@crate::Type)",
        "[`Link`](crate::Type)",
    ),
    (
        "[`Link`](https://docs.rs/regex/)",
        "[`Link`](mod@regex)",
        "[`Link`](regex)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2)",
        "[`Link`](mod@regex)",
        "[`Link`](regex)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex)",
        "[`Link`](mod@regex)",
        "[`Link`](regex)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/struct.Regex.html)",
        "[`Link`](type@regex::Regex)",
        "[`Link`](regex::Regex)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples)",
        "[`Link`](type@regex::Regex#examples)",
        "[`Link`](regex::Regex#examples)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/struct.Regex.html#method.is_match)",
        "[`Link`](regex::Regex::is_match())",
        "[`Link`](regex::Regex::is_match())",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/bytes/index.html)",
        "[`Link`](mod@regex::bytes)",
        "[`Link`](regex::bytes)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/bytes/index.html#syntax)",
        "[`Link`](regex::bytes#syntax)",
        "[`Link`](regex::bytes#syntax)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples)",
        "[`Link`](type@regex::bytes::Regex#examples)",
        "[`Link`](regex::bytes::Regex#examples)",
    ),
    (
        "[`Link`](https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#method.is_match)",
        "[`Link`](regex::bytes::Regex::is_match())",
        "[`Link`](regex::bytes::Regex::is_match())",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/std)",
        "[`Link`](mod@std)",
        "[`Link`](std)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/alloc)",
        "[`Link`](mod@alloc)",
        "[`Link`](alloc)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/core)",
        "[`Link`](mod@core)",
        "[`Link`](core)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/test)",
        "[`Link`](mod@test)",
        "[`Link`](test)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/proc_macro)",
        "[`Link`](mod@proc_macro)",
        "[`Link`](proc_macro)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/std/string/index.html)",
        "[`Link`](mod@std::string)",
        "[`Link`](std::string)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/std/string/struct.String.html)",
        "[`Link`](type@std::string::String)",
        "[`Link`](std::string::String)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/std/string/struct.String.html#examples)",
        "[`Link`](type@std::string::String#examples)",
        "[`Link`](std::string::String#examples)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/std/string/struct.String.html#method.drain)",
        "[`Link`](std::string::String::drain())",
        "[`Link`](std::string::String::drain())",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/std)",
        "[`Link`](mod@std)",
        "[`Link`](std)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/alloc)",
        "[`Link`](mod@alloc)",
        "[`Link`](alloc)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/core)",
        "[`Link`](mod@core)",
        "[`Link`](core)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/test)",
        "[`Link`](mod@test)",
        "[`Link`](test)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/proc_macro)",
        "[`Link`](mod@proc_macro)",
        "[`Link`](proc_macro)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/std/string/index.html)",
        "[`Link`](mod@std::string)",
        "[`Link`](std::string)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/std/string/struct.String.html)",
        "[`Link`](type@std::string::String)",
        "[`Link`](std::string::String)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples)",
        "[`Link`](type@std::string::String#examples)",
        "[`Link`](std::string::String#examples)",
    ),
    (
        "[`Link`](https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain)",
        "[`Link`](std::string::String::drain())",
        "[`Link`](std::string::String::drain())",
    ),
    (
        "[`Link`](#struct.Item)",
        "[`Link`](type@Block::Item)",
        "[`Link`](Block::Item)",
    ),
    (
        "[`Link`](./#struct.Item)",
        "[`Link`](type@Block::Item)",
        "[`Link`](Block::Item)",
    ),
    (
        "[`Link`](././#struct.Item)",
        "[`Link`](type@Block::Item)",
        "[`Link`](Block::Item)",
    ),
    (
        "[`Link`](#section-a)",
        "[`Link`](#section-a)",
        "[`Link`](#section-a)",
    ),
    (
        "[`Link`](#section-1)",
        "[`Link`](#section-1)",
        "[`Link`](#section-1)",
    ),
    (
        "[`Link`](#section-A)",
        "[`Link`](#section-A)",
        "[`Link`](#section-A)",
    ),
    (
        "[`Link`](#section_a)",
        "[`Link`](#section_a)",
        "[`Link`](#section_a)",
    ),
    (
        "[`Link`](#section.a)",
        "[`Link`](#section.a)",
        "[`Link`](#section.a)",
    ),
    (
        "[`Link`](#Section.a)",
        "[`Link`](#Section.a)",
        "[`Link`](#Section.a)",
    ),
    (
        "[`Link`](#rection.a)",
        "[`Link`](#rection.a)",
        "[`Link`](#rection.a)",
    ),
    (
        "[`Link`](#0ection.a)",
        "[`Link`](#0ection.a)",
        "[`Link`](#0ection.a)",
    ),
    (
        "[`Link`](#_ection.a)",
        "[`Link`](#_ection.a)",
        "[`Link`](#_ection.a)",
    ),
    (
        "[`Link`](krate/#section)",
        "[`Link`](crate#section)",
        "[`Link`](crate#section)",
    ),
    (
        "[`Link`](../krate/#section)",
        "[`Link`](crate#section)",
        "[`Link`](crate#section)",
    ),
    (
        "[`Link`](mod1/#section)",
        "[`Link`](mod1#section)",
        "[`Link`](mod1#section)",
    ),
    (
        "[`Link`](mod1/mod2/#section)",
        "[`Link`](mod1::mod2#section)",
        "[`Link`](mod1::mod2#section)",
    ),
    (
        "[`Link`](../../mod1/mod2/#section)",
        "[`Link`](super::super::mod1::mod2#section)",
        "[`Link`](super::super::mod1::mod2#section)",
    ),
    (
        "[`Link`](associatedconstant.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](associatedconstant.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](associatedconstant.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./associatedconstant.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../associatedconstant.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/associatedconstant.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](associatedtype.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](associatedtype.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](associatedtype.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./associatedtype.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../associatedtype.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/associatedtype.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](attr.Type.html)",
        "[`Link`](macro@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](attr.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](attr.Type.html#section-name)",
        "[`Link`](macro@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./attr.Type.html)",
        "[`Link`](macro@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../attr.Type.html)",
        "[`Link`](macro@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/attr.Type.html)",
        "[`Link`](macro@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](constant.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](constant.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](constant.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./constant.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../constant.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/constant.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](derive.Type.html)",
        "[`Link`](macro@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](derive.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](derive.Type.html#section-name)",
        "[`Link`](macro@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./derive.Type.html)",
        "[`Link`](macro@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../derive.Type.html)",
        "[`Link`](macro@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/derive.Type.html)",
        "[`Link`](macro@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](enum.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](enum.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](enum.Type.html#section-name)",
        "[`Link`](type@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./enum.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../enum.Type.html)",
        "[`Link`](type@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/enum.Type.html)",
        "[`Link`](type@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](externcrate.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](externcrate.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](externcrate.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./externcrate.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../externcrate.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/externcrate.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](fn.Type.html)",
        "[`Link`](Type())",
        "[`Link`](Type())",
    ),
    (
        "[`Link`](fn.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](fn.Type.html#section-name)",
        "[`Link`](Type()#section-name)",
        "[`Link`](Type()#section-name)",
    ),
    (
        "[`Link`](./fn.Type.html)",
        "[`Link`](Type())",
        "[`Link`](Type())",
    ),
    (
        "[`Link`](../fn.Type.html)",
        "[`Link`](super::Type())",
        "[`Link`](super::Type())",
    ),
    (
        "[`Link`](../mod1/mod2/fn.Type.html)",
        "[`Link`](super::mod1::mod2::Type())",
        "[`Link`](super::mod1::mod2::Type())",
    ),
    (
        "[`Link`](foreigntype.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](foreigntype.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](foreigntype.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./foreigntype.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../foreigntype.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/foreigntype.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](impl.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](impl.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](impl.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./impl.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../impl.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/impl.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](import.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](import.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](import.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./import.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../import.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/import.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](keyword.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](keyword.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](keyword.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./keyword.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../keyword.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/keyword.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](macro.Type.html)",
        "[`Link`](Type!)",
        "[`Link`](Type!)",
    ),
    (
        "[`Link`](macro.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](macro.Type.html#section-name)",
        "[`Link`](Type!#section-name)",
        "[`Link`](Type!#section-name)",
    ),
    (
        "[`Link`](./macro.Type.html)",
        "[`Link`](Type!)",
        "[`Link`](Type!)",
    ),
    (
        "[`Link`](../macro.Type.html)",
        "[`Link`](super::Type!)",
        "[`Link`](super::Type!)",
    ),
    (
        "[`Link`](../mod1/mod2/macro.Type.html)",
        "[`Link`](super::mod1::mod2::Type!)",
        "[`Link`](super::mod1::mod2::Type!)",
    ),
    (
        "[`Link`](method.Type.html)",
        "[`Link`](Type())",
        "[`Link`](Type())",
    ),
    (
        "[`Link`](method.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](method.Type.html#section-name)",
        "[`Link`](Type()#section-name)",
        "[`Link`](Type()#section-name)",
    ),
    (
        "[`Link`](./method.Type.html)",
        "[`Link`](Type())",
        "[`Link`](Type())",
    ),
    (
        "[`Link`](../method.Type.html)",
        "[`Link`](super::Type())",
        "[`Link`](super::Type())",
    ),
    (
        "[`Link`](../mod1/mod2/method.Type.html)",
        "[`Link`](super::mod1::mod2::Type())",
        "[`Link`](super::mod1::mod2::Type())",
    ),
    (
        "[`Link`](mod.Type.html)",
        "[`Link`](mod@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](mod.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](mod.Type.html#section-name)",
        "[`Link`](mod@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./mod.Type.html)",
        "[`Link`](mod@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../mod.Type.html)",
        "[`Link`](mod@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/mod.Type.html)",
        "[`Link`](mod@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](opaque.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](opaque.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](opaque.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./opaque.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../opaque.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/opaque.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](static.Type.html)",
        "[`Link`](value@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](static.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](static.Type.html#section-name)",
        "[`Link`](value@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./static.Type.html)",
        "[`Link`](value@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../static.Type.html)",
        "[`Link`](value@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/static.Type.html)",
        "[`Link`](value@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](struct.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](struct.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](struct.Type.html#section-name)",
        "[`Link`](type@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./struct.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../struct.Type.html)",
        "[`Link`](type@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/struct.Type.html)",
        "[`Link`](type@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](structfield.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](structfield.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](structfield.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./structfield.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../structfield.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/structfield.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](trait.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](trait.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](trait.Type.html#section-name)",
        "[`Link`](type@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./trait.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../trait.Type.html)",
        "[`Link`](type@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/trait.Type.html)",
        "[`Link`](type@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](traitalias.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](traitalias.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](traitalias.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./traitalias.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../traitalias.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/traitalias.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](tymethod.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](tymethod.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](tymethod.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./tymethod.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../tymethod.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/tymethod.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](type.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](type.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](type.Type.html#section-name)",
        "[`Link`](type@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./type.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../type.Type.html)",
        "[`Link`](type@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/type.Type.html)",
        "[`Link`](type@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](union.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](union.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](union.Type.html#section-name)",
        "[`Link`](type@Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./union.Type.html)",
        "[`Link`](type@Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../union.Type.html)",
        "[`Link`](type@super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/union.Type.html)",
        "[`Link`](type@super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    (
        "[`Link`](variant.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](variant.Type.html#method.call)",
        "[`Link`](Type::call())",
        "[`Link`](Type::call())",
    ),
    (
        "[`Link`](variant.Type.html#section-name)",
        "[`Link`](Type#section-name)",
        "[`Link`](Type#section-name)",
    ),
    (
        "[`Link`](./variant.Type.html)",
        "[`Link`](Type)",
        "[`Link`](Type)",
    ),
    (
        "[`Link`](../variant.Type.html)",
        "[`Link`](super::Type)",
        "[`Link`](super::Type)",
    ),
    (
        "[`Link`](../mod1/mod2/variant.Type.html)",
        "[`Link`](super::mod1::mod2::Type)",
        "[`Link`](super::mod1::mod2::Type)",
    ),
    ("[`Link`](regex)", "[`Link`](mod@regex)", "[`Link`](regex)"),
    (
        "[`Link`](../../regex)",
        "[`Link`](mod@super::super::regex)",
        "[`Link`](super::super::regex)",
    ),
    (
        "[`Link`](../../mod1/mod2/regex)",
        "[`Link`](mod@super::super::mod1::mod2::regex)",
        "[`Link`](super::super::mod1::mod2::regex)",
    ),
    (
        "[`Link`](mod1/mod2)",
        "[`Link`](mod@mod1::mod2)",
        "[`Link`](mod1::mod2)",
    ),
    (
        "[`Link`](../../krate/mod1/mod2/regex)",
        "[`Link`](mod@crate::mod1::mod2::regex)",
        "[`Link`](crate::mod1::mod2::regex)",
    ),
    (
        "[`Link`](mod1/mod2)",
        "[`Link`](mod@mod1::mod2)",
        "[`Link`](mod1::mod2)",
    ),
    (
        "[`Link`](regex/bytes/index.html)",
        "[`Link`](mod@regex::bytes)",
        "[`Link`](regex::bytes)",
    ),
    (
        "[`Link`](regex/bytes/index.html#syntax)",
        "[`Link`](regex::bytes#syntax)",
        "[`Link`](regex::bytes#syntax)",
    ),
];
