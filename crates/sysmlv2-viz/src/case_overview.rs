//! Case overview view: the classic use case diagram. The use case
//! definitions sit as `usecase` nodes inside one boundary rectangle per
//! subject type; each actor type is one `actor` node outside the
//! boundaries, with one line to each use case that names it. No
//! objective or doc notes.
//!
//! Scope: `UseCaseDefinition`s with their own `subject` and `actor`
//! members, found through transparent containers. A subject or actor
//! without a type is keyed and labelled by its own name.

use std::collections::HashSet;
use std::fmt::Write as _;

use sysmlv2_model::json::{ElementRef, ResolvedModel};

use crate::{
    Direction, VizOptions, escape, frame, label_types, link_suffix, stereo_text, stereotype,
    style_header,
};

#[derive(PartialEq, Eq)]
enum Key {
    Type(ElementRef),
    Name(String),
}

/// A subject or actor, merged by its type.
struct Party {
    key: Key,
    label: String,
    link: String,
}

struct Case {
    e: ElementRef,
    subject: Option<usize>,
    actors: Vec<usize>,
}

pub(crate) fn emit(r: &mut ResolvedModel, tops: &[ElementRef], opts: &VizOptions) -> String {
    let mut found = Vec::new();
    for &e in tops {
        collect(r, e, &mut found);
    }

    let mut subjects: Vec<Party> = Vec::new();
    let mut actors: Vec<Party> = Vec::new();
    let mut cases = Vec::new();
    for e in found {
        let mut case = Case {
            e,
            subject: None,
            actors: Vec::new(),
        };
        for m in r.owned_members(e) {
            match r.owning_membership_type(m) {
                Some("SubjectMembership") if case.subject.is_none() => {
                    case.subject = Some(party(r, opts, m, &mut subjects));
                }
                Some("ActorMembership") => {
                    let k = party(r, opts, m, &mut actors);
                    if !case.actors.contains(&k) {
                        case.actors.push(k);
                    }
                }
                _ => {}
            }
        }
        cases.push(case);
    }

    let mut body = String::from("left to right direction\n");
    for (k, s) in subjects.iter().enumerate() {
        let _ = writeln!(
            body,
            "rectangle \"{}\" as s{}{} {{",
            escape(&s.label),
            k + 1,
            s.link
        );
        for (n, c) in cases.iter().enumerate() {
            if c.subject == Some(k) {
                body.push_str("  ");
                case_line(r, opts, c.e, n + 1, &mut body);
            }
        }
        body.push_str("}\n");
    }
    for (n, c) in cases.iter().enumerate() {
        if c.subject.is_none() {
            case_line(r, opts, c.e, n + 1, &mut body);
        }
    }
    for (k, a) in actors.iter().enumerate() {
        let _ = writeln!(
            body,
            "actor \"{}\" as a{}{}",
            escape(&a.label),
            k + 1,
            a.link
        );
    }

    let mut edges = String::new();
    let mut drawn = HashSet::new();
    for (n, c) in cases.iter().enumerate() {
        for &k in &c.actors {
            if drawn.insert((k, n)) {
                let _ = writeln!(edges, "a{} -- n{}", k + 1, n + 1);
            }
        }
    }

    let header = style_header(opts, &["usecase", "rectangle"]);
    let opts = VizOptions {
        direction: Direction::TopToBottom,
        ..opts.clone()
    };
    frame(&opts, &header, &body, &edges)
}

/// Top-down search through transparent containers.
fn collect(r: &mut ResolvedModel, e: ElementRef, found: &mut Vec<ElementRef>) {
    if r.element_type(e) == "UseCaseDefinition" {
        if !found.contains(&e) {
            found.push(e);
        }
        return;
    }
    for m in r.owned_members(e) {
        collect(r, m, found);
    }
}

/// The index of the party that member `m` stands for, added on first
/// sight.
fn party(r: &mut ResolvedModel, opts: &VizOptions, m: ElementRef, all: &mut Vec<Party>) -> usize {
    let (types, _) = label_types(r, m);
    let (key, label, link) = match types.first() {
        Some(&t) => (
            Key::Type(t),
            r.element_name(t).unwrap_or("").to_string(),
            link_suffix(r, opts, t),
        ),
        None => {
            let name = r.element_name(m).unwrap_or("").to_string();
            (Key::Name(name.clone()), name, link_suffix(r, opts, m))
        }
    };
    if let Some(k) = all.iter().position(|p| p.key == key) {
        return k;
    }
    all.push(Party { key, label, link });
    all.len() - 1
}

fn case_line(r: &mut ResolvedModel, opts: &VizOptions, e: ElementRef, n: usize, out: &mut String) {
    let label = r.element_name(e).unwrap_or("").to_string();
    let stereos = stereo_text(r, opts, e, &stereotype(r.element_type(e)));
    let link = link_suffix(r, opts, e);
    let _ = writeln!(
        out,
        "usecase \"{}\" as n{n} {stereos}{link}",
        escape(&label)
    );
}
