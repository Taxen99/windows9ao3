use std::collections::HashMap;

pub struct Page {
    pub title: String,
    pub html: String,
    pub css: String,
}

impl Page {
    fn new() -> Page {
        Self {
            title: String::new(),
            html: String::new(),
            css: String::new(),
        }
    }
}

pub struct PageParser {
    page: Page,
    input: Vec<char>,
    idx: usize,
}

impl PageParser {
    pub fn new(text: &str) -> Self {
        Self {
            page: Page::new(),
            input: text.chars().collect(),
            idx: 0,
        }
    }
    pub fn parse(mut self) -> Page {
        let mut sections: HashMap<String, String> = HashMap::new();
        loop {
            match self.consume() {
                Some(c) => match c {
                    '@' => {
                        if self.consume_if(|c| c == '!').is_some() {
                            self.consume_whitespace();
                            let section_name = self
                                .consume_while(|c| !c.is_whitespace())
                                .expect("section must have name");
                            if !section_name.chars().all(|x| x.is_alphabetic()) {
                                panic!("invalid section name '{}'", section_name);
                            }
                            let section_content = self.parse_section_content();
                            if let Some(_) = sections.insert(section_name.clone(), section_content)
                            {
                                panic!("duplicacte section '{}'", section_name);
                            }
                        }
                    }
                    _ => {
                        panic!("expected section");
                    }
                },
                None => {
                    break;
                }
            }
        }
        Page {
            title: sections.remove("title").unwrap(),
            html: sections.remove("content").unwrap(),
            css: sections.remove("style").unwrap(),
        }
    }
    fn parse_section_content(&mut self) -> String {
        let mut content = String::new();
        loop {
            match self.consume() {
                Some(c) => match c {
                    '@' => {
                        if self.consume_if(|c| c == '!').is_some() {
                            self.idx -= 2;
                            break;
                        }
                        content.push(c);
                    }
                    _ => {
                        content.push(c);
                    }
                },
                None => {
                    break;
                }
            }
        }
        content.trim().to_owned()
    }
    fn try_peek(&self) -> Option<char> {
        self.input.get(self.idx).cloned()
    }
    fn consume(&mut self) -> Option<char> {
        let c = self.try_peek()?;
        self.idx += 1;
        Some(c)
    }
    fn consume_if(&mut self, filter: impl Fn(char) -> bool) -> Option<char> {
        if let Some(c) = self.try_peek()
            && filter(c)
        {
            self.idx += 1;
            return Some(c);
        }
        None
    }
    fn consume_while(&mut self, filter: impl Fn(char) -> bool) -> Option<String> {
        let mut res = String::new();
        while let Some(c) = self.try_peek()
            && filter(c)
        {
            self.idx += 1;
            res.push(c);
        }
        if res.is_empty() {
            return None;
        }
        Some(res)
    }
    fn consume_whitespace(&mut self) -> () {
        self.consume_while(|x| x.is_whitespace());
    }
    fn consume_char(&mut self, filter: char) -> bool {
        if let Some(c) = self.try_peek()
            && filter == c
        {
            self.idx += 1;
            return true;
        }
        false
    }
}

// fn parse_page(text: &str) -> Page {
//     let chars = text.chars().collect::<Vec<_>>();
//     let mut page = Page {
//         title: String::new(),
//         html: String::new(),
//         css: String::new(),
//     };
//     let mut i = 0;
//     loop {}
//     page
// }
