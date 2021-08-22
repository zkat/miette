use atty::Stream;
use owo_colors::Style;

/**
Theme used by [crate::GraphicalReportPrinter] to render fancy [crate::Diagnostic] reports.

A theme consists of two things: the set of characters to be used for drawing,
and the [owo_colors::Style]s to be used to paint various items.

You can create your own custom graphical theme using this type, or you can use
one of the predefined ones using the methods below.
*/
#[derive(Debug, Clone)]
pub struct GraphicalTheme {
    /// Characters to be used for drawing.
    pub characters: ThemeCharacters,
    /// Styles to be used for painting.
    pub styles: ThemeStyles,
}

impl GraphicalTheme {
    /// ASCII-art-based graphical drawing, with ANSI styling.
    pub fn ascii() -> Self {
        Self {
            characters: ThemeCharacters::ascii(),
            styles: ThemeStyles::ansi(),
        }
    }

    /// Graphical theme that draws using both ansi colors and unicode characters.
    pub fn unicode() -> Self {
        Self {
            characters: ThemeCharacters::unicode(),
            styles: ThemeStyles::ansi(),
        }
    }

    /// Graphical theme that draws in monochrome, while still using unicode
    /// characters.
    pub fn unicode_nocolor() -> Self {
        Self {
            characters: ThemeCharacters::unicode(),
            styles: ThemeStyles::none(),
        }
    }

    /// A "basic" graphical theme that skips colors and unicode characters and
    /// just does monochrome ascii art. If you want a completely non-graphical
    /// rendering of your `Diagnostic`s, check out
    /// [crate::NarratableReportPrinter], or write your own
    /// [crate::DiagnosticReportPrinter]!
    pub fn none() -> Self {
        Self {
            characters: ThemeCharacters::ascii(),
            styles: ThemeStyles::none(),
        }
    }
}

impl Default for GraphicalTheme {
    fn default() -> Self {
        match std::env::var("NO_COLOR") {
            _ if !atty::is(Stream::Stdout) || !atty::is(Stream::Stderr) => Self::ascii(),
            Ok(string) if string != "0" => Self::unicode_nocolor(),
            _ => Self::unicode(),
        }
    }
}

/**
Styles for various parts of graphical rendering for the [crate::GraphicalReportPrinter].
*/
#[derive(Debug, Clone)]
pub struct ThemeStyles {
    /// Style to apply to things highlighted as "error".
    pub error: Style,
    /// Style to apply to things highlighted as "warning".
    pub warning: Style,
    /// Style to apply to things highlighted as "advice".
    pub advice: Style,
    /// Style to apply to the diagnostic code.
    pub code: Style,
    /// Style to apply to the help text.
    pub help: Style,
    /// Style to apply to the filename/source name.
    pub filename: Style,
    /// Styles to cycle through (using `.iter().cycle()`), to render the lines
    /// and text for diagnostic highlights.
    pub highlights: Vec<Style>,
}

fn style() -> Style {
    Style::new()
}

impl ThemeStyles {
    /// Nice RGB colors.
    /// Credit: http://terminal.sexy/#FRUV0NDQFRUVrEFCkKlZ9L91ap-1qnWfdbWq0NDQUFBQrEFCkKlZ9L91ap-1qnWfdbWq9fX1
    pub fn rgb() -> Self {
        Self {
            error: style().fg_rgb::<172, 65, 66>(),
            warning: style().fg_rgb::<244, 191, 117>(),
            advice: style().fg_rgb::<106, 159, 181>(),
            code: style().fg_rgb::<170, 117, 159>(),
            help: style().fg_rgb::<106, 159, 181>(),
            filename: style().fg_rgb::<117, 181, 170>().underline().bold(),
            highlights: vec![
                style().fg_rgb::<255, 135, 162>(),
                style().fg_rgb::<150, 232, 133>(),
                style().fg_rgb::<62, 238, 210>(),
                style().fg_rgb::<234, 207, 182>(),
                style().fg_rgb::<130, 221, 255>(),
                style().fg_rgb::<255, 188, 242>(),
            ],
        }
    }

    /// ANSI color-based styles.
    pub fn ansi() -> Self {
        Self {
            error: style().red(),
            warning: style().yellow(),
            advice: style().cyan(),
            code: style().yellow(),
            help: style().cyan(),
            filename: style().cyan().underline().bold(),
            highlights: vec![
                style().red().bold(),
                style().yellow().bold(),
                style().cyan().bold(),
            ],
        }
    }

    /// No styling. Just regular ol' monochrome.
    pub fn none() -> Self {
        Self {
            error: style(),
            warning: style(),
            advice: style(),
            code: style(),
            help: style(),
            filename: style(),
            highlights: vec![style()],
        }
    }
}

// ---------------------------------------
// Most of these characters were taken from
// https://github.com/zesterer/ariadne/blob/e3cb394cb56ecda116a0a1caecd385a49e7f6662/src/draw.rs

/// Characters to be used when drawing when using [crate::GraphicalReportPrinter].
#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ThemeCharacters {
    pub hbar: char,
    pub vbar: char,
    pub xbar: char,
    pub vbar_break: char,

    pub uarrow: char,
    pub rarrow: char,

    pub ltop: char,
    pub mtop: char,
    pub rtop: char,
    pub lbot: char,
    pub rbot: char,
    pub mbot: char,

    pub lbox: char,
    pub rbox: char,

    pub lcross: char,
    pub rcross: char,

    pub underbar: char,
    pub underline: char,

    pub fyi: char,
    pub x: char,
    pub warning: char,
    pub point_right: char,
}

impl ThemeCharacters {
    /// Fancy unicode-based graphical elements.
    pub fn unicode() -> Self {
        Self {
            hbar: '─',
            vbar: '│',
            xbar: '┼',
            vbar_break: '·',
            uarrow: '▲',
            rarrow: '▶',
            ltop: '╭',
            mtop: '┬',
            rtop: '╮',
            lbot: '╰',
            mbot: '┴',
            rbot: '╯',
            lbox: '[',
            rbox: ']',
            lcross: '├',
            rcross: '┤',
            underbar: '┬',
            underline: '─',
            fyi: '‽',
            x: '×',
            warning: '⚠',
            point_right: '☞',
        }
    }

    /// ASCII-art-based graphical elements. Works well on older terminals.
    pub fn ascii() -> Self {
        Self {
            hbar: '-',
            vbar: '|',
            xbar: '+',
            vbar_break: ':',
            uarrow: '^',
            rarrow: '>',
            ltop: ',',
            mtop: 'v',
            rtop: '.',
            lbot: '`',
            mbot: '^',
            rbot: '\'',
            lbox: '[',
            rbox: ']',
            lcross: '|',
            rcross: '|',
            underbar: '|',
            underline: '^',
            fyi: 'i',
            x: 'x',
            warning: '!',
            point_right: '>',
        }
    }
}
