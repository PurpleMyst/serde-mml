use std::str::Chars;

pub struct Reader<'a> {
    chars: Chars<'a>,
    indents: Vec<usize>,
    state: State,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item<'a> {
    Link { text: &'a str, uri: &'a str },
    PushOrderedList,
    PushUnorderedList,
    PopList,
}

#[derive(Debug)]
enum State {
    BeforeItem,
    InItem(usize),
    EOF,
}

impl<'a> Reader<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars(),
            indents: vec![],
            state: State::BeforeItem,
        }
    }

    /// Return the portion of the input string until the given char
    fn take_chars_until(&mut self, needle: char) -> Option<&'a str> {
        let start = self.chars.as_str();
        self.chars.by_ref().find(|&ch| ch == needle)?;
        let end = self.chars.as_str();
        Some(&start[..start.len() - end.len() - needle.len_utf8()])
    }

    /// Calculate the indent of the current item and remove it from the input
    fn next_depth(&mut self) -> usize {
        // We use some Chars::as_str trickery to avoid consuming the first char after the indent
        let result = self
            .chars
            .as_str()
            .chars()
            .take_while(|&c| c == ' ')
            .count();
        self.chars.by_ref().take(result).for_each(|_| ());
        result
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::BeforeItem => {
                    self.state = State::InItem(self.next_depth());
                }

                State::InItem(new_depth) => {
                    // If we've dedented, pop an indent and return
                    if self
                        .indents
                        .last()
                        .map_or(false, |&depth| new_depth < depth)
                    {
                        self.indents.pop();
                        break Some(Item::PopList);
                    }

                    // Get the next character or move to the EOF state
                    let ch = if let Some(ch) = self.chars.next() {
                        ch
                    } else {
                        self.state = State::EOF;
                        continue;
                    };

                    match ch {
                        // If the first character represents a bullet, we've found a new list item
                        '0'..='9' | '*' => {
                            // If we found a number, we must parse more digits and the dot
                            if let '0'..='9' = ch {
                                assert_eq!(
                                    self.chars
                                        .by_ref()
                                        .skip_while(|c| c.is_ascii_digit())
                                        .next(),
                                    Some('.')
                                );
                            }

                            // The Writer always puts a space after the bullet
                            assert_eq!(self.chars.next(), Some(' '));

                            // If we've indented, push on a new indent and reutrn a Push*List
                            if self.indents.last().map_or(true, |&depth| new_depth > depth) {
                                self.indents.push(new_depth);
                                return Some(if ch == '*' {
                                    Item::PushUnorderedList
                                } else {
                                    Item::PushOrderedList
                                });
                            }

                            // Stay in the same state to parse the item
                        }

                        // This is an empty item, most likely just contains a sublist
                        '\n' => {
                            self.state = State::BeforeItem;
                        }

                        // This item a link, parse it
                        '[' => {
                            let text = self.take_chars_until(']')?;
                            if self.chars.next() != Some('(') {
                                break None;
                            }
                            let uri = self.take_chars_until(')')?;
                            self.take_chars_until('\n')?;
                            self.state = State::BeforeItem;
                            break Some(Item::Link { text, uri });
                        }

                        // The Writer never outputs anything else
                        _ => unreachable!("{:?}", ch),
                    }
                }

                // If we've ran out of characters, just pop out of all the lists and return
                State::EOF => {
                    break if let Some(..) = self.indents.pop() {
                        Some(Item::PopList)
                    } else {
                        None
                    }
                }
            }
        }
    }
}
