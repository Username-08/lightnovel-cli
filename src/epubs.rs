use crate::ueberzug;
use crate::Screen;
use epub::doc::EpubDoc;
use html2text::render::text_renderer::TaggedString;
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

        s.display_chapter();
        s
    }

    pub fn display_chapter(&mut self) {
        clear();
        endwin();
        for line in &self.chapter {
            let is_head;
            let mut tagged_strings = line.tagged_strings();
            // check if head tag
            if line.tagged_strings().count() >= 2 {
                if line.tagged_strings().next().unwrap().s == "# " {
                    is_head = true;
                    tagged_strings.next();
                } else {
                    is_head = false;
                }
            } else {
                is_head = false;
            }
            for ann in tagged_strings {
                // println!("{:?}", ann);
                self.handle_annotation(ann, is_head);
            }
            addstr("\n");
        }
    }

    fn handle_annotation(
        &self,
        ann: &TaggedString<Vec<RichAnnotation>>,
        is_head: bool,
    ) {
        let mut attributes = vec![];
        let mut bold: bool = false;
        // TODO: loop through tags and apply each one
        for tag in &ann.tag {
            match tag {
                RichAnnotation::Strong => {
                    attributes.push(A_BOLD());
                    bold = true;
                }
                RichAnnotation::Emphasis => attributes.push(A_ITALIC()),
                RichAnnotation::Strikeout => attributes.push(A_HORIZONTAL()),
                _ => {
                    if is_head {
                        attributes.push(A_BOLD());
                    }
                }
            }
        }

        for attr in attributes.iter() {
            attron(attr.clone());
        }

        // remove asterisk
        if bold {
            addstr(ann.s.trim_matches('*'));
        } else {
            addstr(ann.s.as_str());
        }

        for attr in attributes {
            attroff(attr);
        }
    }

    pub fn scroll(&mut self, scroll_by: i32) {
        todo!()
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
