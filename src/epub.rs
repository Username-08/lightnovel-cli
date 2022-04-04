use crate::ueberzug;
use crate::Screen;
use epub::doc::EpubDoc;
use ncurses::*;
use scraper::{Html, Selector};

impl Screen {
    pub fn parse_epub_from_path(&mut self) {
        let mut doc =
        EpubDoc::new("/home/yash/Downloads/ReZero Starting Life in Another World - LN 01.epub")
            .unwrap();
        for _ in 0..26 {
            doc.go_next().unwrap();
        }
        let x = String::from_utf8(doc.get_current().unwrap()).unwrap();

        let document = Html::parse_document(&x);
        let selector = Selector::parse(r#"body"#).unwrap();
        let meow: String =
            document.select(&selector).next().unwrap().text().collect();
        let meow = meow.split("\n").map(|x| x.to_string()).collect();
        self.raw_doc = meow;
        self.parse_doc();
        let mut meow = ueberzug::UeConf {
            identifier: "0",
            path: "/home/yash/Downloads/wallpaper(1).png",
            y: -10,
            draw: Some(true),
            ..Default::default()
        };
        // let cover_data = doc
        //     .get_resource_by_path("OEBPS/images/Art_P122.jpg")
        //     .unwrap();

        self.display_epub_chapter_screen(&mut meow);
    }

    pub fn epub_draw(&self) {
        for (index, line) in (&self.doc).iter().enumerate() {
            if index < self.curr_top as usize {
                continue;
            }
            if index as i32 >= self.curr_bot {
                break;
            }

            let mut temp = line.clone();
            // add completition percentage
            if index as i32 == self.curr_top {
                temp.pop();
                let length = temp.chars().collect::<Vec<_>>().len();
                temp.push_str(
                    " ".repeat((self.maxx - length as i32 - 5) as usize)
                        .as_str(),
                );
                let mut percentage = String::new();
                let mut percentage_val = ((self.curr_bot as f32
                    / self.doc.len() as f32)
                    * 100.00) as i32;
                if percentage_val > 100 {
                    percentage_val = 100;
                }

                percentage.push_str(
                    format!("{:0.3}", percentage_val.to_string()).as_str(),
                );
                temp.push_str(percentage.as_str());
                temp.push_str("%\n");
            }
            // color title
            if index == 1 {
                attron(A_BOLD);
                attron(COLOR_PAIR(1));
                addstr(line.as_str());
                attroff(COLOR_PAIR(1));
                attroff(A_BOLD);
            } else {
                addstr(temp.as_str());
            }
        }
    }

    pub fn display_epub_chapter_screen(&mut self, img: &mut ueberzug::UeConf) {
        clear();
        keypad(stdscr(), true);
        let drawer = ueberzug::Ueberzug::new();
        drawer.draw(&img);

        // self.draw(true);

        let mut ch = getch();

        loop {
            if is_term_resized(self.maxy, self.maxx) {
                let tempy = self.maxy;
                getmaxyx(stdscr(), &mut self.maxy, &mut self.maxx);
                self.maxy -= 1;
                self.curr_bot += self.maxy - tempy;
                self.parse_doc();
                // wrefresh(stdscr());
                clear();
                self.draw(true);
            }
            match ch as u32 {
                113 => {
                    clear();
                    break;
                }
                106 | 258 => {
                    // self.epub_draw();
                    img.y += 1;
                    drawer.draw(&img);
                    ch = getch();
                }
                107 | 259 => {
                    // self.epub_draw();
                    img.y -= 1;
                    drawer.draw(&img);
                    ch = getch();
                }
                _ => {
                    ch = getch();
                }
            }
        }
    }
}
