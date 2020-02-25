use crate::rudoku::{ChildObserved, Observer, Value, ValueExt};
use std::rc::Rc;

pub struct PositionedText {}
impl PositionedText {
    pub fn new() -> PositionedText {
        PositionedText {}
    }

    fn position(row: i32, column: i32) -> String {
        format!("\x1b[{};{}H", row, column)
    }
}
impl ChildObserved<String, (i32, i32, String)> for PositionedText {
    fn value(
        &mut self,
        _: &Rc<dyn Observer>,
        input: Box<dyn Value<(i32, i32, String)>>,
    ) -> Box<dyn Value<String>> {
        input.map(|x| format!("{}{}", PositionedText::position(x.0, x.1), x.2))
    }
}

pub struct HorizontalLine {
    positioned: PositionedText,
}
impl HorizontalLine {
    pub fn new() -> HorizontalLine {
        HorizontalLine {
            positioned: PositionedText::new(),
        }
    }

    fn line(length: i32) -> String {
        std::iter::repeat("=")
            .take(length as usize)
            .collect::<String>()
    }
}
impl ChildObserved<String, (i32, i32, i32)> for HorizontalLine {
    fn value(
        &mut self,
        observer: &Rc<dyn Observer>,
        input: Box<dyn Value<(i32, i32, i32)>>,
    ) -> Box<dyn Value<String>> {
        self.positioned.value(
            observer,
            input.map(|x| (x.0, x.1, HorizontalLine::line(x.2))),
        )
    }
}

pub struct VerticalLine {}
impl VerticalLine {
    pub fn new() -> VerticalLine {
        VerticalLine {}
    }
}
impl ChildObserved<String, (i32, i32, i32)> for VerticalLine {
    fn value(
        &mut self,
        _: &Rc<dyn Observer>,
        input: Box<dyn Value<(i32, i32, i32)>>,
    ) -> Box<dyn Value<String>> {
        input.map(|(row, col, len)| {
            let mut r = String::new();
            for i in 0..len {
                r.push_str(&format!("{}|", PositionedText::position(row + i, col)));
            }
            r
        })
    }
}

pub struct Clear {}
impl Clear {
    pub fn new() -> Clear {
        Clear {}
    }

    fn clear(length: i32) -> String {
        std::iter::repeat(" ")
            .take(length as usize)
            .collect::<String>()
    }
}
impl ChildObserved<String, (i32, i32, i32, i32)> for Clear {
    fn value(
        &mut self,
        _: &Rc<dyn Observer>,
        input: Box<dyn Value<(i32, i32, i32, i32)>>,
    ) -> Box<dyn Value<String>> {
        input.map(|(row, col, width, height)| {
            let mut r = String::new();
            for i in 0..height {
                r.push_str(&format!(
                    "{}{}",
                    PositionedText::position(row + i, col),
                    Clear::clear(width)
                ))
            }
            r
        })
    }
}

pub struct Border {
    top: HorizontalLine,
    bottom: HorizontalLine,
    left: VerticalLine,
    right: VerticalLine,
}
impl Border {
    pub fn new() -> Border {
        Border {
            top: HorizontalLine::new(),
            bottom: HorizontalLine::new(),
            left: VerticalLine::new(),
            right: VerticalLine::new(),
        }
    }
}
impl ChildObserved<String, (i32, i32, i32, i32)> for Border {
    fn value(
        &mut self,
        observer: &Rc<dyn Observer>,
        input_unique: Box<dyn Value<(i32, i32, i32, i32)>>,
    ) -> Box<dyn Value<String>> {
        let input = input_unique.split();

        self.top
            .value(
                observer,
                input
                    .take()
                    .map(|(row, col, width, _height)| (row, col, width)),
            )
            .join(
                self.bottom.value(
                    observer,
                    input
                        .take()
                        .map(|(row, col, width, height)| (row + height - 1, col, width)),
                ),
            )
            .map(|x| format!("{}{}", x.0, x.1))
            .join(
                self.left.value(
                    observer,
                    input
                        .take()
                        .map(|(row, col, _width, height)| (row, col, height)),
                ),
            )
            .map(|x| format!("{}{}", x.0, x.1))
            .join(
                self.right.value(
                    observer,
                    input
                        .take()
                        .map(|(row, col, width, height)| (row, col + width - 1, height)),
                ),
            )
            .map(|x| format!("{}{}", x.0, x.1))
    }
}

pub struct Decorated {
    clear: Clear,
    border: Border,
}
impl Decorated {
    pub fn new() -> Decorated {
        Decorated {
            clear: Clear::new(),
            border: Border::new(),
        }
    }
}
impl ChildObserved<String, (i32, i32, Vec<String>)> for Decorated {
    fn value(
        &mut self,
        observer: &Rc<dyn Observer>,
        input_unique: Box<dyn Value<(i32, i32, Vec<String>)>>,
    ) -> Box<dyn Value<String>> {
        let input = input_unique
            .map(|(row, col, text)| {
                (
                    row,
                    col,
                    text.iter().map(|x| x.len()).max().unwrap() as i32 + 4,
                    text.len() as i32 + 4,
                    text,
                )
            })
            .split();

        self.clear
            .value(
                observer,
                input
                    .take()
                    .map(|(row, col, width, height, _text)| (row, col, width, height)),
            )
            .join(
                self.border.value(
                    observer,
                    input
                        .take()
                        .map(|(row, col, width, height, _text)| (row, col, width, height)),
                ),
            )
            .map(|x| format!("{}{}", x.0, x.1))
            .join(input.take().map(|(row, col, _width, _height, text)| {
                let mut r = String::new();
                for i in 0..(text.len() as i32) {
                    r.push_str(&format!(
                        "{}{}",
                        PositionedText::position(row + 2 + i, col + 2),
                        text[i as usize]
                    ));
                }
                r
            }))
            .map(|x| format!("{}{}", x.0, x.1))
    }
}
