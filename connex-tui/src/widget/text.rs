use std::str::FromStr;

use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
};

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{alpha1, char, one_of},
    combinator::{map, map_res, opt, verify},
    multi::many1,
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Debug)]
enum Command {
    Fg(Color),
    Bg(Color),
    Modifier(Modifier),
}

impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "green" => Self::Fg(Color::Green),
            "bg:green" => Self::Bg(Color::Green),
            "b" => Self::Modifier(Modifier::BOLD),
            "i" => Self::Modifier(Modifier::ITALIC),
            _ => return Err(()),
        })
    }
}

impl From<Command> for Style {
    fn from(c: Command) -> Self {
        match c {
            Command::Fg(color) => Style::default().fg(color),
            Command::Bg(color) => Style::default().bg(color),
            Command::Modifier(modifier) => Style::default().add_modifier(modifier),
        }
    }
}

#[derive(Debug)]
enum Part<'a> {
    PlainText(&'a str),
    Command(Command, Vec<Part<'a>>),
}

impl<'a> Part<'a> {
    pub fn into_spans(self, style: Option<Style>) -> Spans<'a> {
        match self {
            Part::PlainText(t) => if let Some(style) = style {
                Span::styled(t, style)
            } else {
                Span::raw(t)
            }
            .into(),
            Part::Command(cmd, children) => {
                let style = style.unwrap_or_default().patch(cmd.into());
                children
                    .into_iter()
                    .flat_map(|part| part.into_spans(Some(style)).0)
                    .collect::<Vec<_>>()
                    .into()
            }
        }
    }
}

fn command_start(s: &str) -> IResult<&str, Command> {
    let (s, cmd) = preceded(char('<'), map_res(alpha1, |s: &str| s.parse()))(s)?;
    let (s, _) = opt(char(' '))(s)?;
    Ok((s, cmd))
}

fn command_end(s: &str) -> IResult<&str, &str> {
    tag(">")(s)
}

fn command(s: &str) -> IResult<&str, Part> {
    let (s, (name, parts, _)) = tuple((command_start, parts, command_end))(s)?;
    Ok((s, Part::Command(name, parts)))
}

fn plain_text(s: &str) -> IResult<&str, &str> {
    escaped(is_not("<>\\"), '\\', one_of("<>\\q"))(s)
}

fn part(s: &str) -> IResult<&str, Part> {
    alt((map(plain_text, Part::PlainText), command))(s)
}

fn parts(s: &str) -> IResult<&str, Vec<Part>> {
    many1(verify(part, |p| !matches!(p, Part::PlainText(""))))(s)
}

fn tui_spans(s: &str) -> Result<Spans<'_>, &str> {
    let (remain, parts) = parts(s).unwrap();
    if !remain.is_empty() {
        return Err(remain);
    }

    let x = parts
        .into_iter()
        .flat_map(|part| part.into_spans(None).0)
        .collect::<Vec<_>>();

    Ok(x.into())
}

pub fn tui_text(s: &str) -> Text<'_> {
    s.lines().map(tui_spans).collect::<Result<Vec<_>, _>>().unwrap().into()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_part() {
        let s = "<green w>/<green s>";
        println!("{:?}", super::tui_text(s));
    }
}
