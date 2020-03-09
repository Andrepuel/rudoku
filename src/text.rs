use crate::rudoku::{Observed, ObservedExt};

fn position(row: i32, column: i32) -> String {
    format!("\x1b[{};{}H", row, column)
}

pub fn positioned_text(
    pos: impl Observed<T=(i32, i32)>,
    text: impl Observed<T=String>,
) -> impl Observed<T=String> {
    pos.join(text)
        .map(|((row, col), text)| format!("{}{}", position(row, col), text))
}

fn line(length: i32) -> String {
    std::iter::repeat("=")
        .take(length as usize)
        .collect::<String>()
}

pub fn horizontal_line(
    pos: impl Observed<T=(i32, i32)>,
    width: impl Observed<T=i32>,
) -> impl Observed<T=String> {
    positioned_text(pos, width.map(|x| line(x)))
}

pub fn vertical_line(
    pos: impl Observed<T=(i32, i32)>,
    height: impl Observed<T=i32>,
) -> impl Observed<T=String> {
    pos.join(height).map(|((row, col), height)| {
        // FIXME Use observed combination for that
        let mut r = String::new();
        for i in 0..height {
            r.push_str(&format!("{}|", position(row + i, col)));
        }
        r
    })
}

fn blank(length: i32) -> String {
    std::iter::repeat(" ")
        .take(length as usize)
        .collect::<String>()
}

pub fn clear(
    pos: impl Observed<T=(i32, i32)>,
    dim: impl Observed<T=(i32, i32)>,
) -> impl Observed<T=String> {
    pos.join(dim).map(|((row, col), (width, height))| {
        let mut r = String::new();
        for i in 0..height {
            r.push_str(&format!("{}{}", position(row + i, col), blank(width)))
        }
        r
    })
}

pub fn border(
    pos: impl Observed<T=(i32, i32)> + Clone,
    dim: impl Observed<T=(i32, i32)> + Clone,
) -> impl Observed<T=String> {
    horizontal_line(pos.clone(), dim.clone().map(|(width, _)| width))
        .join(horizontal_line(
            pos.clone()
                .join(dim.clone())
                .map(|((row, col), (_, height))| (row + height - 1, col)),
            dim.clone().map(|(width, _)| width),
        ))
        .map(|(a, b)| format!("{}{}", a, b))
        .join(vertical_line(
            pos.clone(),
            dim.clone().map(|(_, height)| height),
        ))
        .map(|(a, b)| format!("{}{}", a, b))
        .join(vertical_line(
            pos.clone()
                .join(dim.clone())
                .map(|((row, col), (width, _))| (row, col + width - 1)),
            dim.clone().map(|(_, height)| height),
        ))
        .map(|(a, b)| format!("{}{}", a, b))
}

pub fn decorated(
    pos: impl Observed<T=(i32, i32)> + Clone,
    text: impl Observed<T=Vec<String>> + Clone,
) -> impl Observed<T=String> {
    let dim = text.clone()
        .map(|text| {
            (
                text.iter().map(|x| x.len()).max().unwrap() as i32 + 4,
                text.len() as i32 + 4,
            )
        });

    clear(pos.clone(), dim.clone())
        .join(border(pos.clone(), dim.clone()))
        .map(|x| format!("{}{}", x.0, x.1))
        .join(pos.clone().join(text).map(|((row, col), text)| {
            let mut r = String::new();
            for i in 0..(text.len() as i32) {
                r.push_str(&format!(
                    "{}{}",
                    position(row + 2 + i, col + 2),
                    text[i as usize]
                ));
            }
            r
        }))
        .map(|x| format!("{}{}", x.0, x.1))
}
