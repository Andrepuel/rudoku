use crate::rudoku::{Observed, ObservedExt};

fn position(row: i32, column: i32) -> String {
    format!("\x1b[{};{}H", row, column)
}

pub fn positioned_text(
    pos: Box<dyn Observed<(i32, i32)>>,
    text: Box<dyn Observed<String>>,
) -> Box<dyn Observed<String>> {
    pos.join(text)
        .map(|((row, col), text)| format!("{}{}", position(row, col), text))
}

fn line(length: i32) -> String {
    std::iter::repeat("=")
        .take(length as usize)
        .collect::<String>()
}

pub fn horizontal_line(
    pos: Box<dyn Observed<(i32, i32)>>,
    width: Box<dyn Observed<i32>>,
) -> Box<dyn Observed<String>> {
    positioned_text(pos, width.map(|x| line(x)))
}

pub fn vertical_line(
    pos: Box<dyn Observed<(i32, i32)>>,
    height: Box<dyn Observed<i32>>,
) -> Box<dyn Observed<String>> {
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
    pos: Box<dyn Observed<(i32, i32)>>,
    dim: Box<dyn Observed<(i32, i32)>>,
) -> Box<dyn Observed<String>> {
    pos.join(dim).map(|((row, col), (width, height))| {
        let mut r = String::new();
        for i in 0..height {
            r.push_str(&format!("{}{}", position(row + i, col), blank(width)))
        }
        r
    })
}

pub fn border(
    pos_arg: Box<dyn Observed<(i32, i32)>>,
    dim_arg: Box<dyn Observed<(i32, i32)>>,
) -> Box<dyn Observed<String>> {
    let pos = pos_arg.split();
    let dim = dim_arg.split();

    horizontal_line(pos.take(), dim.take().map(|(width, _)| width))
        .join(horizontal_line(
            pos.take()
                .join(dim.take())
                .map(|((row, col), (_, height))| (row + height - 1, col)),
            dim.take().map(|(width, _)| width),
        ))
        .map(|(a, b)| format!("{}{}", a, b))
        .join(vertical_line(
            pos.take(),
            dim.take().map(|(_, height)| height),
        ))
        .map(|(a, b)| format!("{}{}", a, b))
        .join(vertical_line(
            pos.take()
                .join(dim.take())
                .map(|((row, col), (width, _))| (row, col + width - 1)),
            dim.take().map(|(_, height)| height),
        ))
        .map(|(a, b)| format!("{}{}", a, b))
}

pub fn decorated(
    pos_arg: Box<dyn Observed<(i32, i32)>>,
    text_arg: Box<dyn Observed<Vec<String>>>,
) -> Box<dyn Observed<String>> {
    let pos = pos_arg.split();
    let text = text_arg.split();
    let dim = text
        .take()
        .map(|text| {
            (
                text.iter().map(|x| x.len()).max().unwrap() as i32 + 4,
                text.len() as i32 + 4,
            )
        })
        .split();

    clear(pos.take(), dim.take())
        .join(border(pos.take(), dim.take()))
        .map(|x| format!("{}{}", x.0, x.1))
        .join(pos.take().join(text.take()).map(|((row, col), text)| {
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
