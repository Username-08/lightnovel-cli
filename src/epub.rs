use crate::ueberzug;
use crate::Screen;
use epub::doc::EpubDoc;
use html2text::{
    render::text_renderer::{RichAnnotation, RichDecorator, TaggedLine},
    RenderedText,
};
use ncurses::*;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use tempfile;

pub struct Image {
    image_path: String,
    rows: u16,
    line_no: usize,
    col_no: usize,
    ue_conf: ueberzug::UeConf,
}

pub struct Epub {
    maxx: i32,
    maxy: i32,
    path: String,
    curr_top: i32,
    curr_bot: i32,
    epub_doc: EpubDoc<File>,
    drawer: ueberzug::Ueberzug,
    chapter: Vec<TaggedLine<Vec<RichAnnotation>>>,
    temp_dir: tempfile::TempDir,
    image_list: Vec<Image>,
}

impl Epub {
    pub fn new(maxx: i32, maxy: i32, path: String) -> Self {
        let mut epub_doc = EpubDoc::new("/home/yash/Downloads/ReZero Starting Life in Another World - LN 01.epub").unwrap();
        for _ in 0..26 {
            epub_doc.go_next().unwrap();
        }
        let chapter_string =
            String::from_utf8(epub_doc.get_current().unwrap()).unwrap();
        let doc = html2text::parse(chapter_string.as_bytes());
        let padding = maxx / 8;
        let chapter = doc
            .render_rich((maxx - padding - padding) as usize)
            .into_lines();

        let temp_dir = tempfile::tempdir().unwrap();

        let drawer = ueberzug::Ueberzug::new();

        let mut s = Self {
            maxx,
            maxy,
            path: String::from("/home/yash/Downloads/ReZero Starting Life in Another World - LN 01.epub"),
            curr_top: 0,
            curr_bot: maxy,
            epub_doc,
            drawer,
            chapter,
            temp_dir,
            image_list: vec![]
        };

        s.parse_epub_from_path();
        s
    }

    pub fn parse_epub_from_path(&mut self) {
        // for _ in 0..26 {
        //     self.epub_doc.go_next().unwrap();
        // }
        let x =
            String::from_utf8(self.epub_doc.get_current().unwrap()).unwrap();

        let mut meow = ueberzug::UeConf {
            identifier: "0".to_string(),
            path: "/home/yash/Downloads/wallpaper(1).png".to_string(),
            y: -10,
            // draw: Some(true),
            ..Default::default()
        };
        // let cover_data = doc
        //     .get_resource_by_path("OEBPS/images/Art_P122.jpg")
        //     .unwrap();
        //
        let doc = html2text::parse(x.as_bytes());
        let padding = self.maxx / 8;
        let lines = doc
            .render_rich((self.maxx - padding - padding) as usize)
            .into_lines();

        self.display_epub_chapter_screen();
    }

    pub fn epub_draw(&mut self, scroll_by: i32) {
        self.scroll_image(scroll_by);
        clear();
        let mut lines_taken = 0;
        for image in &self.image_list {
            if image.ue_conf.y < 0 && image.ue_conf.y + image.rows as i16 >= 0 {
                let lines_taken_by_image = {
                    if image.ue_conf.y + image.rows as i16 >= self.maxy as i16 {
                        self.maxy as u16 + 1
                    } else {
                        (image.ue_conf.y + image.rows as i16) as u16 + 1
                    }
                };
                lines_taken += lines_taken_by_image;
                self.drawer.draw(&image.ue_conf);
                for _ in 0..lines_taken_by_image {
                    addstr("\n");
                }
                if lines_taken >= self.maxy as u16 {
                    return ();
                }

                self.curr_top = image.line_no as i32;
            }
        }

        match self.image_list.last() {
            Some(image) => {
                if lines_taken == 0 {
                    if image.ue_conf.y < 0
                        && image.ue_conf.y
                            + image.rows as i16
                            + scroll_by as i16
                            >= 0
                    {
                        self.curr_top = image.line_no as i32;
                        // self.scroll_text(scroll_by);
                    } else {
                        self.scroll_text(scroll_by);
                    }
                }
            }
            None => {
                self.scroll_text(scroll_by);
            }
        }

        // addstr(continue_line.to_string().as_str());
        // addstr("\n");
        // addstr(continue_line.to_string().as_str());

        for (line_number, line) in self.chapter.iter().enumerate() {
            if line_number < self.curr_top as usize {
                continue;
            }
            if lines_taken >= self.maxy as u16 {
                return ();
            }

            addstr(" ".repeat((self.maxx / 8) as usize).as_str());
            let mut is_header = false;
            'outer: for (index, ts) in line.tagged_strings().enumerate() {
                // check if header
                if index == 0 && ts.s.eq("# ") {
                    is_header = true;
                    continue;
                }
                for ann in &ts.tag {
                    match ann {
                        RichAnnotation::Default => {
                            addstr(ts.s.as_str());
                        }

                        RichAnnotation::Image => {
                            if line_number == self.curr_top as usize {
                                continue 'outer;
                            }
                            let image_path = self
                                .temp_dir
                                .path()
                                .join(get_image_name_from_path(&ts.s))
                                .to_str()
                                .unwrap()
                                .to_string();

                            if std::path::Path::new(&image_path).exists() {
                                for image in &self.image_list {
                                    if image.col_no == index
                                        && image.line_no == line_number
                                    {
                                        if image.ue_conf.path == image_path {
                                            let lines_taken_by_image = {
                                                if image.rows
                                                    + lines_taken as u16
                                                    > self.maxy as u16
                                                {
                                                    self.maxy as u16
                                                        - lines_taken as u16
                                                } else {
                                                    image.rows
                                                }
                                            };
                                            lines_taken += lines_taken_by_image;
                                            self.drawer.draw(&image.ue_conf);
                                            for _ in 0..lines_taken_by_image {
                                                addstr("\n");
                                            }
                                            if lines_taken >= self.maxy as u16 {
                                                return ();
                                            }
                                        }
                                    } else {
                                        self.image_list.push(
                                            self.create_image_from_path(
                                                image_path,
                                                lines_taken as usize,
                                                index,
                                                line_number,
                                            ),
                                        );

                                        let image =
                                            &self.image_list.last().unwrap();
                                        let lines_taken_by_image = {
                                            if image.rows + lines_taken as u16
                                                > self.maxy as u16
                                            {
                                                self.maxy as u16
                                                    - lines_taken as u16
                                            } else {
                                                image.rows
                                            }
                                        };

                                        self.drawer.draw(&image.ue_conf);
                                        for _ in 0..lines_taken_by_image {
                                            addstr("\n");
                                        }
                                        lines_taken += image.rows as u16;
                                        if lines_taken >= self.maxy as u16 {
                                            return ();
                                        }
                                    }

                                    addstr("\n");
                                    addstr(
                                        " ".repeat((self.maxx / 8) as usize)
                                            .as_str(),
                                    );
                                    lines_taken += 1;
                                    continue 'outer;
                                }
                                continue 'outer;
                            } else {
                                let mut temp = String::from("OEBPS/");
                                temp.push_str(&ts.s);
                                let contents = self
                                    .epub_doc
                                    .get_resource_by_path(temp)
                                    .unwrap();
                                let mut image_file =
                                    std::fs::File::create(&image_path).unwrap();
                                image_file.write(&contents.as_slice()).unwrap();
                                self.image_list.push(
                                    self.create_image_from_path(
                                        image_path,
                                        lines_taken as usize,
                                        index,
                                        line_number,
                                    ),
                                );

                                let image = &self.image_list.last().unwrap();
                                let lines_taken_by_image = {
                                    if image.rows + lines_taken as u16
                                        > self.maxy as u16
                                    {
                                        self.maxy as u16 - lines_taken as u16
                                    } else {
                                        image.rows
                                    }
                                };

                                self.drawer.draw(&image.ue_conf);
                                for _ in 0..lines_taken_by_image {
                                    addstr("\n");
                                }
                                lines_taken += image.rows as u16;
                                if lines_taken >= self.maxy as u16 {
                                    return ();
                                }
                            }

                            addstr("\n");
                            addstr(
                                " ".repeat((self.maxx / 8) as usize).as_str(),
                            );
                            lines_taken += 1;
                            continue 'outer;
                        }

                        RichAnnotation::Link(_) => {}

                        // italics
                        RichAnnotation::Emphasis => {
                            if is_header && line_number >= 1 {
                                attron(A_ITALIC());
                                attron(A_BOLD());
                                attron(COLOR_PAIR(1));
                                addstr(ts.s.as_str());
                                attroff(A_ITALIC());
                                attroff(A_BOLD());
                                attroff(COLOR_PAIR(1));
                                continue 'outer;
                            }
                            attron(A_ITALIC());
                            addstr(ts.s.as_str());
                            attroff(A_ITALIC());
                            continue 'outer;
                        }

                        // bold
                        RichAnnotation::Strong => {
                            attron(A_BOLD());
                            let string: String =
                                ts.s.chars()
                                    .skip(1)
                                    .take(ts.s.len() - 2)
                                    .collect();
                            addstr(string.as_str());
                            attroff(A_BOLD());
                            continue 'outer;
                        }

                        // strikeout
                        RichAnnotation::Strikeout => {
                            addstr(ts.s.as_str());
                        }
                        RichAnnotation::Code => {
                            addstr(ts.s.as_str());
                        }
                        RichAnnotation::Preformat(_) => {
                            addstr(ts.s.as_str());
                        }
                    }
                }

                if line_number >= 1 && is_header {
                    attron(A_BOLD());
                    attron(COLOR_PAIR(1));
                    addstr(ts.s.as_str());
                    attroff(A_BOLD());
                    attroff(COLOR_PAIR(1));
                    continue;
                }
                addstr(ts.s.as_str());
            }
            addstr("\n");
            lines_taken += 1;
        }
    }

    pub fn display_epub_chapter_screen(&mut self) {
        clear();
        keypad(stdscr(), true);

        self.epub_draw(0);

        let mut ch = getch();

        loop {
            if is_term_resized(self.maxy, self.maxx) {
                let tempy = self.maxy;
                getmaxyx(stdscr(), &mut self.maxy, &mut self.maxx);
                self.maxy -= 1;
                self.curr_bot += self.maxy - tempy;
                let chapter_string =
                    String::from_utf8(self.epub_doc.get_current().unwrap())
                        .unwrap();
                let doc = html2text::parse(chapter_string.as_bytes());
                let padding = self.maxx / 8;
                let chapter = doc
                    .render_rich((self.maxx - padding - padding) as usize)
                    .into_lines();
                self.chapter = chapter;
                clear();
                self.epub_draw(0);
            }
            match ch as u32 {
                113 => {
                    clear();
                    break;
                }
                106 | 258 => {
                    // self.scroll(1);

                    self.epub_draw(1);
                    ch = getch();
                }
                107 | 259 => {
                    // self.scroll(-1);

                    self.epub_draw(-1);
                    ch = getch();
                }
                _ => {
                    ch = getch();
                }
            }
        }
    }

    pub fn scroll_text(&mut self, scroll_by: i32) {
        if scroll_by > 0 {
            self.curr_bot += scroll_by;
            self.curr_top += scroll_by;
            if self.curr_bot >= self.chapter.len() as i32 {
                self.curr_bot = self.chapter.len() as i32;
                self.curr_top = self.chapter.len() as i32 - self.maxy;
            }
        } else if scroll_by < 0 {
            if self.curr_top + scroll_by < 0 {
                self.curr_top = 0;
                self.curr_bot = self.maxy;
            } else {
                self.curr_bot += scroll_by;
                self.curr_top += scroll_by;
            }
        }
    }

    pub fn scroll_image(&mut self, scroll_by: i32) {
        if scroll_by > 0 {
            if self.curr_bot >= self.chapter.len() as i32 {
                self.curr_bot = self.chapter.len() as i32;
                self.curr_top = self.chapter.len() as i32 - self.maxy;
            } else {
                for image in &mut self.image_list {
                    image.ue_conf.y -= scroll_by as i16;
                    if image.ue_conf.y + image.rows as i16 + scroll_by as i16
                        >= 0
                        && image.ue_conf.y + image.rows as i16 <= 0
                    {
                        self.drawer.draw(&image.ue_conf);
                    }
                }
            }
        } else if scroll_by < 0 {
            if self.curr_top + scroll_by < 0 {
                self.curr_top = 0;
                self.curr_bot = self.maxy;
            } else {
                for image in &mut self.image_list {
                    image.ue_conf.y -= scroll_by as i16;
                    if image.ue_conf.y < 0
                        && image.ue_conf.y + scroll_by as i16 >= 0
                    {
                        self.drawer.draw(&image.ue_conf);
                    }
                }
            }
        }
    }

    pub fn create_image_from_path(
        &self,
        image_path: String,
        row_no: usize,
        col_no: usize,
        line_no: usize,
    ) -> Image {
        let (_, y) = image::image_dimensions(&image_path).unwrap();
        let (_, term_y) = get_term_size();
        let rows = ((self.maxy as f32 / term_y as f32) * y as f32) as u16 + 1;
        let image_name = get_image_name_from_path(&image_path);

        let ue_conf = ueberzug::UeConf {
            identifier: self.image_list.len().to_string(),
            path: image_path.clone(),
            // y: (term_y as i16 / self.maxy as i16) * line_no as i16,
            y: row_no as i16,
            ..Default::default()
        };

        Image {
            image_path,
            rows,
            line_no,
            col_no,
            ue_conf,
        }
    }
}

pub fn get_term_size() -> (u16, u16) {
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size as *mut _);
        (size.ws_xpixel, size.ws_ypixel)
    }
}

pub fn get_image_name_from_path(path: &String) -> String {
    path.split("/").last().unwrap().to_string()
}
