use fspec_core::{MatchSettings, Report, Severity};

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
