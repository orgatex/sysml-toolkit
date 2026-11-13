//! Case overview: the classic use case diagram, one boundary per
//! subject type, each actor type once.

use sysmlv2_model::json::ResolvedModel;
use sysmlv2_model::model::Model;
use sysmlv2_viz::{Direction, View, VizOptions, plantuml, view_directed};

const SRC: &str = "package P {
    part def Sol; part def Sys; part def User; part def Dongle;
    use case def A { subject s : Sol; actor u : User; }
    use case def B { subject s : Sol; actor u : User; actor u2 : User; }
    use case def C { subject s : Sys; actor d : Dongle; actor u : User; }
    use case def D { actor n; }
    use case def E { subject s : Sys; objective { doc /* O */ } }
    view def Overview {
        rendering asClassicUseCaseDiagram;
        render asClassicUseCaseDiagram;
    }
    view overview : Overview { expose P::*; }
    package Empty { part def X; }
}
";

fn resolved() -> ResolvedModel {
    let mut model = Model::new();
    let unit = model.add_source("case_overview.sysml".to_string(), SRC);
    assert!(
        unit.diagnostics.is_empty(),
        "fixture must parse: {:?}",
        unit.diagnostics
    );
    ResolvedModel::build(&model)
}

fn render_with(root: &str, opts: VizOptions) -> String {
    let mut r = resolved();
    let root = r
        .resolve_qualified(root)
        .unwrap_or_else(|| panic!("{root} must resolve"));
    plantuml(&mut r, Some(root), &opts.with_view(View::CaseOverview))
}

fn puml() -> String {
    render_with("P", VizOptions::default())
}

/// The alias of the node line that starts with `prefix`.
fn alias(out: &str, prefix: &str) -> String {
    let line = out
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with(prefix))
        .unwrap_or_else(|| panic!("no {prefix} in\n{out}"));
    line.split(" as ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap()
        .to_string()
}

/// The lines between `rectangle "<name>" ... {` and its `}`.
fn inside<'a>(out: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("rectangle \"{name}\"");
    out.lines()
        .skip_while(|l| !l.starts_with(&open))
        .skip(1)
        .take_while(|l| l.trim() != "}")
        .collect()
}

fn count(out: &str, prefix: &str) -> usize {
    out.lines().filter(|l| l.trim().starts_with(prefix)).count()
}

#[test]
fn groups_use_cases_per_subject_type() {
    let out = puml();
    assert_eq!(count(&out, "rectangle \"Sol\""), 1, "{out}");
    assert_eq!(count(&out, "rectangle \"Sys\""), 1, "{out}");
    let sol = inside(&out, "Sol");
    for case in ["A", "B"] {
        let line = format!("usecase \"{case}\"");
        assert!(sol.iter().any(|l| l.trim().starts_with(&line)), "{out}");
    }
    let sys = inside(&out, "Sys");
    for case in ["C", "E"] {
        let line = format!("usecase \"{case}\"");
        assert!(sys.iter().any(|l| l.trim().starts_with(&line)), "{out}");
    }
}

#[test]
fn draws_each_actor_type_once() {
    let out = puml();
    assert_eq!(count(&out, "actor \"User\""), 1, "{out}");
    assert_eq!(count(&out, "actor \"Dongle\""), 1, "{out}");
    let user = alias(&out, "actor \"User\"");
    for case in ["A", "B", "C"] {
        let n = alias(&out, &format!("usecase \"{case}\""));
        let edge = format!("{user} -- {n}");
        assert_eq!(count(&out, &edge), 1, "{edge} in\n{out}");
    }
}

#[test]
fn untyped_actor_uses_its_name() {
    let out = puml();
    assert_eq!(count(&out, "actor \"n\""), 1, "{out}");
    let n = alias(&out, "actor \"n\"");
    let d = alias(&out, "usecase \"D\"");
    assert_eq!(count(&out, &format!("{n} -- {d}")), 1, "{out}");
}

#[test]
fn use_case_without_subject_is_outside() {
    let out = puml();
    let d = out
        .lines()
        .find(|l| l.trim().starts_with("usecase \"D\""))
        .unwrap_or_else(|| panic!("no D in\n{out}"));
    assert!(d.starts_with("usecase"), "D is indented: {d:?}");
    for name in ["Sol", "Sys"] {
        assert!(
            !inside(&out, name).iter().any(|l| l.contains("\"D\"")),
            "{out}"
        );
    }
}

#[test]
fn no_notes() {
    let out = puml();
    assert!(!out.contains("note "), "{out}");
    assert!(!out.contains("«objective»"), "{out}");
}

#[test]
fn left_to_right() {
    for dir in [Direction::TopToBottom, Direction::LeftToRight] {
        let out = render_with("P", VizOptions::default().with_direction(dir));
        assert_eq!(count(&out, "left to right direction"), 1, "{out}");
    }
}

#[test]
fn empty_scope() {
    let out = render_with("P::Empty", VizOptions::default());
    assert!(out.starts_with("@startuml\n"), "{out}");
    assert!(out.ends_with("@enduml\n"), "{out}");
    assert_eq!(count(&out, "usecase"), 0, "{out}");
}

#[test]
fn rendering_selects_case_overview() {
    let mut r = resolved();
    let view = r.resolve_qualified("P::overview").expect("view resolves");
    let (style, exposed) = view_directed(&mut r, view).expect("a view usage");
    assert_eq!(style, Some(View::CaseOverview));
    for case in ["A", "B", "C", "D", "E"] {
        let def = r
            .resolve_qualified(&format!("P::{case}"))
            .expect("case resolves");
        assert!(exposed.contains(&def), "{case} not exposed");
    }
}
