use owo_colors::Style;

pub struct MietteTheme {
    pub characters: MietteCharacters,
    pub styles: MietteStyles,
}

impl MietteTheme {
    pub fn basic() -> Self {
        Self {
            characters: MietteCharacters::ascii(),
            styles: MietteStyles::ansi(),
        }
    }
    pub fn unicode() -> Self {
        Self {
            characters: MietteCharacters::unicode(),
            styles: MietteStyles::ansi(),
        }
    }
    pub fn unicode_nocolor() -> Self {
        Self {
            characters: MietteCharacters::unicode(),
            styles: MietteStyles::none(),
        }
    }
    pub fn none() -> Self {
        Self {
            characters: MietteCharacters::ascii(),
            styles: MietteStyles::none(),
        }
    }
}

impl Default for MietteTheme {
    fn default() -> Self {
        match std::env::var("NO_COLOR") {
            Ok(string) if string != "0" => Self::unicode_nocolor(),
            _ => Self::unicode(),
        }
    }
}

pub struct MietteStyles {
    pub error: Style,
    pub warning: Style,
    pub advice: Style,
    pub code: Style,
    pub help: Style,
    pub filename: Style,
    pub highlights: Vec<Style>,
}

fn style() -> Style {
    Style::new()
}

impl MietteStyles {
    pub fn ansi() -> Self {
        Self {
            error: style().red(),
            warning: style().yellow(),
            advice: style().cyan(),
            code: style().yellow(),
            help: style().cyan(),
            filename: style().green(),
            highlights: vec![style().red(), style().magenta(), style().cyan()],
        }
    }

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
// All code below here was taken from ariadne here:
// https://github.com/zesterer/ariadne/blob/e3cb394cb56ecda116a0a1caecd385a49e7f6662/src/draw.rs
pub struct MietteCharacters {
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

    pub eq: char,
}

impl MietteCharacters {
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
            eq: '﹦',
        }
    }

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
            eq: '=',
        }
    }
}
