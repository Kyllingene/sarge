use std::num::NonZeroUsize;

use crate::tag::{Cli, Full};

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct DocParams {
    pub(crate) max_doc_width: usize,
    pub(crate) has_short: bool,
    pub(crate) long_width: Option<NonZeroUsize>,
    pub(crate) env_width: Option<NonZeroUsize>,
}

/// If `width.is_none()`, returns a single space. Else, returns width + 2 spaces.
fn empty(width: Option<NonZeroUsize>) -> String {
    " ".repeat(width.map_or(0, |x| usize::from(x) + 1) + 1)
}

/// Returns the padding necessary for continuing doc lines.
fn doc_newline(params: DocParams) -> String {
    let mut width = if params.has_short { 6 } else { 0 };

    if let Some(long_width) = params.long_width {
        width += usize::from(long_width) + 1;
    }

    if let Some(env_width) = params.env_width {
        width += usize::from(env_width) + 1;
    }

    " ".repeat(width)
}

fn wrap_doc(doc: &str, params: DocParams) -> String {
    assert!(
        params.max_doc_width > 5,
        "{} is not wide enough for docs",
        params.max_doc_width
    );

    if doc.len() < params.max_doc_width - 1 {
        format!(" : {doc}")
    } else {
        let mut s = String::from(" : ");
        let padding = doc_newline(params);

        // TODO: add soft wrapping
        let mut width = 2;
        for ch in doc.chars() {
            if width >= params.max_doc_width {
                s.push_str("\n ");
                s.push_str(&padding);
                s.push(ch);
                width = 1;
            } else if ch == '\n' {
                s.push_str("\n ");
                s.push_str(&padding);
                width = 0;
            } else if ch != '\r' {
                s.push(ch);
                width += 1;
            }
        }

        s
    }
}

pub(crate) fn update_params(params: &mut DocParams, arg: &Full) {
    if let Some(cli) = &arg.cli {
        match cli {
            Cli::Short(_) => params.has_short = true,
            Cli::Long(long) => {
                params.long_width = Some(
                    params
                        .long_width
                        .map_or(0, usize::from)
                        .max(long.len())
                        .try_into()
                        .unwrap(),
                );
            }
            Cli::Both(_, long) => {
                params.has_short = true;
                params.long_width = Some(
                    params
                        .long_width
                        .map_or(0, usize::from)
                        .max(long.len())
                        .try_into()
                        .unwrap(),
                );
            }
        }
    }

    if let Some(env) = &arg.env {
        params.env_width = Some(
            params
                .env_width
                .map_or(0, usize::from)
                .max(env.len())
                .try_into()
                .unwrap(),
        );
    }

    params.max_doc_width = (80
        - if params.has_short { 3 } else { 0 }
        - params.long_width.map_or(0, usize::from)
        - params.env_width.map_or(0, usize::from))
    .max(12);
}

pub(crate) fn render_argument(arg: &Full, params: DocParams) -> String {
    let mut s = String::from(" ");

    if let Some(cli) = &arg.cli {
        match cli {
            Cli::Short(short) => {
                s.push('-');
                s.push(*short);
                s.push(' ');

                // s.push_str(" /");
                s.push_str(&empty(params.long_width));
            }
            Cli::Long(long) => {
                if params.has_short {
                    // s.push_str("   / ");
                    s.push_str("   ");
                }

                s.push_str("--");
                s.push_str(long);
                s.push_str(&" ".repeat(usize::from(params.long_width.unwrap()) - long.len()));
            }
            Cli::Both(short, long) => {
                s.push('-');
                s.push(*short);

                // s.push_str(" / ");
                s.push(' ');

                s.push_str("--");
                s.push_str(long);
                s.push_str(&" ".repeat(usize::from(params.long_width.unwrap()) - long.len()));
            }
        }
    } else {
        if params.has_short {
            s.push_str("    ");
        }

        if let Some(width) = params.long_width {
            s.push_str(&" ".repeat(usize::from(width) + 1));
        }
    }

    if let Some(env) = &arg.env {
        s.push_str(" $");
        s.push_str(env);
        s.push_str(&" ".repeat(usize::from(params.env_width.unwrap()) - env.len()));
    } else {
        s.push_str(&empty(params.env_width));
    }

    if let Some(doc) = &arg.doc {
        s.push_str(&wrap_doc(doc, params));
    }

    s
}
