use std::fmt;

use crate::{protocol::Diagnostic, ReportHandler, Severity};

/**
[ReportHandler] that renders json output.
It's a machine-readable output.
*/
#[derive(Debug, Clone)]
pub struct JSONReportHandler;

impl JSONReportHandler {
    /// Create a new [JSONReportHandler]. There are no customization
    /// options.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JSONReportHandler {
    fn default() -> Self {
        Self::new()
    }
}

fn escape(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '"' => "\\\\\"".to_string(),
            '\'' => "\\\\'".to_string(),
            '\r' => "\\\\r".to_string(),
            '\n' => "\\\\n".to_string(),
            '\t' => "\\\\t".to_string(),
            '\u{08}' => "\\\\b".to_string(),
            '\u{0c}' => "\\\\f".to_string(),
            c => format!("{}", c),
        })
        .collect()
}

impl JSONReportHandler {
    /// Render a [Diagnostic]. This function is mostly internal and meant to
    /// be called by the toplevel [ReportHandler] handler, but is
    /// made public to make it easier (possible) to test in isolation from
    /// global state.
    pub fn render_report(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        write!(f, r#"{{"message": "{}","#, escape(&diagnostic.to_string()))?;
        if let Some(code) = diagnostic.code() {
            write!(f, r#""code": "{}","#, escape(&code.to_string()))?;
        }
        let severity = match diagnostic.severity() {
            Some(Severity::Error) | None => "error",
            Some(Severity::Warning) => "warning",
            Some(Severity::Advice) => "advice",
        };
        write!(f, r#""severity": "{:}","#, severity)?;
        if let Some(url) = diagnostic.url() {
            write!(f, r#""url": "{}","#, &url.to_string())?;
        }
        if let Some(help) = diagnostic.help() {
            write!(f, r#""help": "{}","#, escape(&help.to_string()))?;
        }
        if diagnostic.source_code().is_some() {
            self.render_snippets(f, diagnostic)?;
        }
        if let Some(labels) = diagnostic.labels() {
            write!(f, r#""labels": ["#)?;
            let mut add_comma = false;
            for label in labels {
                if add_comma {
                    write!(f, ",")?;
                } else {
                    add_comma = true;
                }
                write!(f, "{{")?;
                if let Some(label_name) = label.label() {
                    write!(f, r#""label": "{}","#, escape(label_name))?;
                }
                write!(f, r#""span": {{"#)?;
                write!(f, r#""offset": {},"#, label.offset())?;
                write!(f, r#""length": {}"#, label.len())?;

                write!(f, "}}}}")?;
            }
            write!(f, "],")?;
        } else {
            write!(f, r#""labels": [],"#)?;
        }
        if let Some(relateds) = diagnostic.related() {
            write!(f, r#""related": ["#)?;
            for related in relateds {
                self.render_report(f, related)?;
            }
            write!(f, "]")?;
        } else {
            write!(f, r#""related": []"#)?;
        }
        write!(f, "}}")
    }

    fn render_snippets(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        if let Some(source) = diagnostic.source_code() {
            if let Some(mut labels) = diagnostic.labels() {
                if let Some(label) = labels.next() {
                    if let Ok(span_content) = source.read_span(label.inner(), 0, 0) {
                        let filename = span_content.name().unwrap_or_default();
                        return write!(f, r#""filename": "{}","#, escape(filename));
                    }
                }
            }
        }
        write!(f, r#""filename": "","#)
    }
}

impl ReportHandler for JSONReportHandler {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render_report(f, diagnostic)
    }
}
