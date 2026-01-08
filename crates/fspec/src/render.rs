use crate::args::OutputFormat;
use fspec_core::{MatchSettings, Report, Severity};
use serde::Serialize;

// Until the report schema stabilizes, we probably don't want to directly deserialize the
// struct via serde. so we use a helper.
// This
#[derive(Serialize)]
struct JsonOut<'a> {
    ok: bool,
    unaccounted: Vec<&'a str>,
    diagnostics: Vec<JsonDiag<'a>>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonDiag<'a> {
    code: &'a str,
    severity: String,
    path: &'a str,
    message: &'a str,
    rule_lines: &'a [usize],
}

#[derive(Serialize)]
struct JsonSummary {
    unaccounted_count: usize,
    // you can add more later without breaking humans
}

fn severity_to_string(sev: Severity) -> String {
    match sev {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
    .to_string()
}

pub fn render_json(report: &Report, settings: &MatchSettings) -> String {
    let un = report.unaccounted_paths();
    let diags = report.diagnostics();

    let out = JsonOut {
        ok: un.is_empty(),
        unaccounted: un.clone(),
        diagnostics: diags
            .iter()
            .map(|d| JsonDiag {
                code: d.code,
                severity: severity_to_string(d.severity),
                path: d.path.as_str(),
                message: d.message.as_str(),
                rule_lines: &d.rule_lines,
            })
            .collect(),
        summary: JsonSummary {
            unaccounted_count: un.len(),
        },
    };

    // if this fails, it’s a programmer error; still return something sane
    serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{\"ok\":false}".to_string())
}

pub fn render_human(
    report: &Report,
    settings: &MatchSettings,
    verbosity: u8,
    quiet: bool,
) -> String {
    let mut out = String::new();

    if !quiet && verbosity > 0 {
        out.push_str(&format!(
            "fspec: leaf={:?}, default_severity={:?}\n",
            settings.allow_file_or_dir_leaf, settings.default_severity
        ));
    }

    let un = report.unaccounted_paths();

    if quiet {
        for p in un {
            out.push_str(p);
            out.push('\n');
        }
        return out;
    }

    if un.is_empty() {
        out.push_str("OK: no unaccounted paths\n");
        return out;
    }

    // For now, all “findings” are printed using settings.default_severity,
    // because Report doesn’t yet attach severities per-path.
    let sev_label = match settings.default_severity {
        Severity::Info => "INFO",
        Severity::Warning => "WARNING",
        Severity::Error => "ERROR",
    };

    for p in un {
        out.push_str(&format!("{sev_label} unaccounted: {p}\n"));
    }

    if verbosity > 0 {
        out.push_str(&format!(
            "summary: unaccounted={}\n",
            report.unaccounted_paths().len()
        ));
    }

    out
}

pub fn render(
    report: &Report,
    settings: &MatchSettings,
    format: OutputFormat,
    verbosity: u8,
    quiet: bool,
) -> String {
    match format {
        OutputFormat::Human => render_human(report, settings, verbosity, quiet),
        OutputFormat::Json => render_json(report, settings),
    }
}
