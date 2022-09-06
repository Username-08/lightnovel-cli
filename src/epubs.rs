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
use std::path::Path;
use tempfile;

pub struct Image {
    image_path: String,
    rows: u16,
    line_no: usize,
    // col_no: usize,
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
        addstr(temp_dir.path().to_str().unwrap());
        getch();

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
        for (line_no, line) in self.chapter.iter().enumerate() {
            // dont render lines out of screen
            if (line_no as i32) < self.curr_top {
                continue;
            } else if (line_no as i32) > self.curr_bot {
                continue;
            }

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
                let mut attributes = vec![];
                let mut ann_str = ann.s.as_str();

                for tag in &ann.tag {
                    match tag {
                        RichAnnotation::Strong => {
                            attributes.push(A_BOLD());
                            ann_str = ann_str.trim_matches('*');
                        }
                        RichAnnotation::Emphasis => attributes.push(A_ITALIC()),
                        RichAnnotation::Strikeout => {
                            let ls = ann_str.len();
                            ann_str = &ann_str[1..ls - 2];
                        }
                        RichAnnotation::Image => {
                            let mut img_path = self.temp_dir.path().to_owned();
                            let img_name =
                                get_image_name_from_path(&ann_str.to_string());
                            img_path = img_path.join(img_name);
                            getch();

                            // image has been added to temp dir
                            if img_path.exists() {
                                // check if image has already been created
                                for image in self.image_list {
                                    if image.line_no == line_no {
                                        continue;
                                    }
                                }
                                let img = self.create_image_from_path(
                                    img_path.to_str().unwrap().to_string(),
                                    line_no,
                                    line_no,
                                );
                            } else {
                                // copy image from epub to temp file
                                let img_data =
                                    match self.epub_doc.get_resource_by_path(
                                        format!("OEBPS/{}", ann_str).as_str(),
                                    ) {
                                        Ok(data) => data,
                                        Err(_) => panic!("Error reading Epub"),
                                    };

                                let mut fd = File::options()
                                    .write(true)
                                    .create(true)
                                    .open(&img_path)
                                    .unwrap();
                                fd.write(&img_data[..]).unwrap();

                                let img = self.create_image_from_path(
                                    img_path.to_str().unwrap().to_string(),
                                    line_no,
                                    line_no,
                                );
                            }
                        }
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

                addstr(ann_str);

                for attr in attributes {
                    attroff(attr);
                }
            }
            addstr("\n");
        }
    }

    pub fn scroll(&mut self, scroll_by: i32) {
        todo!()
    }

    pub fn create_image_from_path(
        &self,
        image_path: String,
        row_no: usize,
        // col_no: usize,
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
            // col_no,
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
