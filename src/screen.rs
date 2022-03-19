use scraper::{Html, Selector};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Write},
};

use ncurses::*;

pub struct Screen {
    raw_doc: Vec<String>,
    doc: Vec<String>,
    maxx: i32,
    maxy: i32,
    curr_bot: i32,
    curr_top: i32,
    pub url: String,
    path: String,
}

impl Screen {
    pub async fn new(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut s = Self {
            raw_doc: vec![],
            doc: vec![],
            maxx: -1,
            maxy: -1,
            curr_bot: -1,
            curr_top: 0,
            url: String::new(),
            path,
        };

        setlocale(LcCategory::all, "");
        initscr();
        keypad(stdscr(), true);
        start_color();
        init_pair(1, COLOR_GREEN, COLOR_BLACK);
        noecho();
        raw();
        clear();
        getmaxyx(stdscr(), &mut s.maxy, &mut s.maxx);
        s.maxy -= 2;
        s.curr_bot = s.maxy;
        s.draw_welcome_screen().await?;
        endwin();
        Ok(s)
    }

    pub async fn draw_chapter_screen(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        clear();
        keypad(stdscr(), true);

        self.draw();

        let mut ch = getch();

        loop {
            if is_term_resized(self.maxy, self.maxx) {
                getmaxyx(stdscr(), &mut self.maxy, &mut self.maxx);
                self.parse_doc();
                // wrefresh(stdscr());
                clear();
                self.draw();
            }

            match ch as u32 {
                // q
                113 => {
                    clear();
                    self.curr_top = 0;
                    self.curr_bot = self.maxy;
                    self.update_novels();
                    break;
                }
                // j or down_arrow
                106 | 258 => {
                    clear();
                    self.scroll(1);
                    self.draw();
                    ch = getch();
                }
                // d
                100 => {
                    clear();
                    self.scroll(self.maxy / 2);
                    self.draw();
                    ch = getch();
                }
                // k or up_arrow
                107 | 259 => {
                    clear();
                    self.scroll(-1);
                    self.draw();
                    ch = getch();
                }
                // u
                117 => {
                    clear();
                    self.scroll(-(self.maxy / 2));
                    self.draw();
                    ch = getch();
                }
                // h or left_arrow
                104 | 260 => {
                    // move 1 chapter back
                    clear();
                    // reset screen to top of page
                    self.curr_top = 0;
                    self.curr_bot = self.maxy;

                    // self.url = self.prev_url.clone();
                    self.change_chapter(-1);
                    self.get_doc().await?;
                    self.draw();
                    ch = getch();
                }
                // l or right_arrow
                108 | 261 => {
                    // move one chapter front
                    clear();
                    // reset screen to top of page
                    self.curr_top = 0;
                    self.curr_bot = self.maxy;

                    // self.url = self.next_url.clone();
                    self.change_chapter(1);
                    self.get_doc().await?;
                    self.draw();
                    ch = getch();
                }
                _ => {
                    ch = getch();
                }
            }
        }
        Ok(())
    }

    pub fn scroll(&mut self, scroll_by: i32) {
        if scroll_by > 0 {
            self.curr_bot += scroll_by;
            self.curr_top += scroll_by;
            if self.curr_bot >= self.doc.len() as i32 {
                self.curr_bot = self.doc.len() as i32;
                self.curr_top = self.doc.len() as i32 - self.maxy;
            }
        } else if scroll_by < 0 {
            if self.curr_top + scroll_by <= 0 {
                self.curr_top = 0;
                self.curr_bot = self.maxy;
            } else {
                self.curr_bot += scroll_by;
                self.curr_top += scroll_by;
            }
        }
    }

    pub fn draw(&mut self) {
        for (index, line) in (&self.doc).iter().enumerate() {
            if index < self.curr_top as usize {
                continue;
            }
            if index as i32 >= self.curr_bot {
                break;
            }

            if index == 1 {
                attron(A_BOLD);
                attron(COLOR_PAIR(1));
                addstr(line.as_str());
                attroff(COLOR_PAIR(1));
                attroff(A_BOLD);
            } else {
                addstr(line.as_str());
            }
        }
    }

    pub fn add_padding(&self, x: String) -> Vec<String> {
        let mut vec: Vec<String> = Vec::new();
        let mut result = String::new();
        // calculate padding
        let padding = self.maxx / 8;
        let line_len = self.maxx - padding - padding;
        result.push_str(" ".repeat(padding as usize).as_str());

        let mut length = 0;
        for word in x.split(' ') {
            length += word.len() + 1;
            if length >= line_len as usize {
                result.push_str(" ".repeat(padding as usize).as_str());
                result.push_str("\n");
                vec.push(result);
                result = "".to_string();
                length = word.len() + 1;
                result.push_str(" ".repeat(padding as usize).as_str());
                result.push_str(word);
                result.push_str(" ");
            } else {
                result.push_str(word);
                result.push_str(" ");
            }
        }
        result.push_str(" ".repeat(padding as usize).as_str());
        result.push_str("\n");
        vec.push(result);
        vec.push("\n".to_string());
        vec
    }

    pub fn parse_doc(&mut self) {
        let mut result: Vec<String> = Vec::new();
        result.push("\n".to_string());
        for line in &self.raw_doc {
            let parsed_line = self.add_padding(line.clone());
            for x in parsed_line {
                result.push(x);
            }
        }

        let mut bottom_line = "<-- previous chapter (h)".to_string();
        bottom_line.push_str(" ".repeat(self.maxx as usize - 44).as_str());
        bottom_line.push_str("next chapter (l) -->");
        result.push(bottom_line);
        self.doc = result;
    }

    pub async fn get_doc(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut result: Vec<String> = Vec::new();
        let resp = match reqwest::get(&self.url).await {
            Ok(x) => x.text().await?,
            Err(_) => panic!("Connection refused!")
        };

        let fragment = Html::parse_fragment(&resp);

        let selector_content = Selector::parse(r#"div[class="txt "]"#).unwrap();
        let article = fragment.select(&selector_content).next().unwrap();

        let selector_title = Selector::parse(r#"h4"#).unwrap();
        let title = match article.select(&selector_title).next() {
            Some(item) => item.text().collect::<Vec<_>>()[0]
                .to_string()
                .trim()
                .to_string(),
            None => "".to_string(),
        };
        if title != "".to_string() {
            result.push(title);

        }

        let empty_vec: Vec<&str> = vec![];
        let selector = Selector::parse(r#"p"#).unwrap();
        for element in article.select(&selector) {
            let content = element.text().collect::<Vec<_>>();
            if content == empty_vec {
                continue;
            }
            let line = content[0].to_string().trim().to_string();
            if line == "" {
                continue;
            }
            result.push(line);
        }
        self.raw_doc = result.clone();
        self.doc = result;
        self.parse_doc();

        Ok(())
    }

    pub fn update_novels(&self) {
        // get novel title
        let title =
            self.url.split("/").collect::<Vec<_>>()[3].replace("-", " ");

        // read file
        let mut line = String::new();
        let mut file_buffer = File::options()
            .read(true)
            .open(&self.path)
            .expect("error reading config file");

        file_buffer
            .read_to_string(&mut line)
            .expect("error reading config file");

        drop(file_buffer);

        // find list of all titles
        let mut list_of_titles = line
            .split("\n")
            .into_iter()
            .map(|x| x.split("#").collect::<Vec<_>>())
            .collect::<Vec<_>>();

        // if title is new add it to list else update the url of pre-existing novel
        let mut is_title_present = false;
        for i in 0..list_of_titles.len() {
            if list_of_titles[i][0] == title {
                list_of_titles[i][1] = self.url.trim();
                // push most recent novel to top of list
                let removed_title = list_of_titles.remove(i);
                list_of_titles.insert(0, removed_title);

                is_title_present = true;
                break;
            }
        }
        if !is_title_present {
            list_of_titles.insert(0, vec![&title, self.url.as_str()])
        }

        let mut file_buffer = File::options()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .expect("error reading config file");

        let mut file_content = list_of_titles
            .into_iter()
            .map(|x| x.join("#"))
            .collect::<Vec<_>>()
            .join("\n");
        file_content = file_content.trim().to_string();

        write!(&mut file_buffer, "{}", file_content.as_str())
            .expect("error writing to file");
    }

    pub async fn draw_welcome_screen(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut result, mut chapter_urls) = self.make_welcome_screen();
        let x = 2;
        let mut y = 3;

        self.draw();
        wmove(stdscr(), y, x);
        wrefresh(stdscr());

        let mut ch = getch();

        loop {
            match ch {
                113 => {
                    endwin();
                    break;
                }
                106 | 258 => {
                    // clear();
                    if y as i32 == self.maxy - 1 {
                        clear();
                        self.scroll(1);
                        self.draw();
                        wmove(stdscr(), y, x);
                        wrefresh(stdscr());
                    }
                    // if you have reached bottom of all options, stop scrolling
                    else if y == 2 + chapter_urls.len() as i32 {
                    } else {
                        y += 1;
                        wmove(stdscr(), y, x);
                        wrefresh(stdscr());
                    }
                    ch = getch();
                }
                107 | 259 => {
                    if y as i32 == 1 {
                        clear();
                        self.scroll(-1);
                        self.draw();
                        wmove(stdscr(), y, x);
                        wrefresh(stdscr());
                    } else {
                        y -= 1;
                        wmove(stdscr(), y, x);
                        wrefresh(stdscr());
                    }
                    ch = getch();
                }
                // s
                115 => {
                    self.display_search_screen().await?;
                    (result, chapter_urls) = self.make_welcome_screen();
                    clear();
                    self.draw();
                    wmove(stdscr(), y, x);
                    wrefresh(stdscr());
                    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
                    ch = getch();
                }
                10 => {
                    let mut line: Vec<chtype> = Vec::new();
                    if y != 1 {
                        mvinchnstr(y, 0, &mut line, self.maxx + 1);
                        wmove(stdscr(), y, x);
                        wrefresh(stdscr());
                        let mut chapter = line
                            .into_iter()
                            .map(|x| char::from_u32(x).unwrap())
                            .collect::<String>();
                        chapter.pop();
                        let chapter = chapter.trim().to_string();

                        let mut is_novel_found = false;
                        // get novel url
                        for id in 3..result.len() {
                            if result[id].trim() == chapter {
                                self.url = (&chapter_urls[id - 3]).clone();
                                self.get_doc().await?;

                                // draw ln screen
                                is_novel_found = true;
                                self.draw_chapter_screen().await?;
                                y = 3;
                                wmove(stdscr(), 3, 2);
                                wrefresh(stdscr());
                                break;
                            }
                        }
                        if is_novel_found {
                            (result, chapter_urls) = self.make_welcome_screen();
                            clear();
                            self.draw();
                            wmove(stdscr(), y, x);
                            wrefresh(stdscr());
                            curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
                        }
                    }
                    ch = getch();
                }
                _ => {
                    ch = getch();
                }
            }
        }
        Ok(())
    }

    pub async fn display_search_screen(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.make_search_screen(None).await?;
        let mut x = 3;
        let mut y = 3;

        clear();
        noecho();
        keypad(stdscr(), false);
        self.draw();
        wmove(stdscr(), y, x);
        wrefresh(stdscr());

        let search_result: Vec<[String; 3]>;

        let mut ch = getch();

        let mut novel: Vec<chtype> = Vec::new();
        loop {
            match ch {
                10 => {
                    mvinchnstr(y, 0, &mut novel, self.maxx + 1);
                    let mut chapter = novel
                        .into_iter()
                        .map(|x| char::from_u32(x).unwrap())
                        .collect::<Vec<_>>();
                    chapter.remove(0);
                    chapter.remove(0);
                    chapter.remove(0);
                    let chapter: String = chapter.into_iter().collect();
                    let temp = self.make_search_screen(Some(chapter)).await?;
                    match temp {
                        Some(result) => search_result = result,
                        None => panic!("error finding chapter"),
                    }
                    break;
                }
                _ => {
                    if ch == 127 {
                        if x < 4 {
                            x = 4;
                        }
                        mvdelch(y, x - 1);
                        x -= 1;
                    } else {
                        x += 1;
                        addch(ch as u32);
                    }
                    ch = getch();
                }
            }
        }
        // x = 3;
        y = 4;
        clear();
        noecho();
        keypad(stdscr(), true);
        self.draw();
        wmove(stdscr(), 4, 2);
        wrefresh(stdscr());
        ch = getch();

        let mut prompt_len;
        loop {
            match ch {
                113 => {
                    clear();
                    break;
                }
                106 | 258 => {
                    // hit bottom of page, start scrolling down
                    if y as i32 == self.maxy {
                        clear();
                        self.scroll(1);
                        self.draw();
                        wmove(stdscr(), y, 2);
                        wrefresh(stdscr());
                    } else if y == 3 + search_result.len() as i32 {
                    } else {
                        y += 1;
                        wmove(stdscr(), y, 2);
                        wrefresh(stdscr());
                    }
                    ch = getch();
                }
                107 | 259 => {
                    // hit top of page, start scrolling up
                    if y as i32 == 1 {
                        clear();
                        self.scroll(-1);
                        self.draw();
                        wmove(stdscr(), y, 2);
                        wrefresh(stdscr());
                    } else {
                        y -= 1;
                        wmove(stdscr(), y, 2);
                        wrefresh(stdscr());
                    }
                    ch = getch();
                }
                10 => {
                    let mut novel: Vec<chtype> = Vec::new();
                    mvinchnstr(y, 0, &mut novel, self.maxx + 1);
                    let mut chapter = novel
                        .into_iter()
                        .map(|x| char::from_u32(x).unwrap())
                        .collect::<Vec<_>>();
                    chapter.remove(0);
                    chapter.remove(0);
                    chapter.remove(0);
                    chapter.remove(0);
                    chapter.remove(0);
                    chapter.pop();
                    let chapter: String = chapter
                        .into_iter()
                        .collect::<String>()
                        .trim()
                        .to_string();

                    let mut url = "https://freewebnovel.com".to_string();
                    let mut max_chapter = -1;
                    let mut is_novel_found = false;
                    for element in &search_result {
                        if element[0] == chapter {
                            is_novel_found = true;
                            url.push_str(element[1].as_str());
                            max_chapter = element[2].parse::<i32>().unwrap();
                        }
                    }
                    if !is_novel_found {
                        ch = getch();
                        continue;
                    }
                    // url.push_str(&search_result[(y - 4) as usize][1].clone());
                    self.url = url;
                    // let max_chapter = search_result[(y - 4) as usize][2]
                    //     .parse::<i32>()
                    //     .unwrap();
                    clear();
                    noecho();
                    keypad(stdscr(), false);
                    addstr("\n");
                    let prompt =
                        format!(" enter chapter [1 - {}]: ", max_chapter);
                    x = prompt.len() as i32;
                    prompt_len = prompt.len() as i32 + 1;
                    addstr(prompt.as_str());
                    let mut novel: Vec<chtype> = Vec::new();
                    ch = getch();
                    loop {
                        match ch {
                            10 => {
                                mvinchnstr(1, 0, &mut novel, self.maxx + 1);
                                let chapter = novel
                                    .iter()
                                    .map(|x| char::from_u32(x.clone()).unwrap())
                                    .collect::<String>();
                                let mut chapter = chapter
                                    .split(":")
                                    .collect::<Vec<_>>()
                                    .pop()
                                    .unwrap()
                                    .to_string();
                                chapter.pop();
                                chapter = chapter.trim().to_string();

                                match chapter.parse::<i32>() {
                                    Ok(val) => {
                                        if val <= 0 {
                                            chapter = "1".to_string();
                                        } else if val >= max_chapter {
                                            chapter = max_chapter.to_string();
                                        }
                                        self.change_chapter(
                                            chapter.parse::<i32>().unwrap()
                                                - max_chapter,
                                        );
                                        break;
                                    }
                                    Err(_) => {
                                        clear();
                                        addstr("\n");
                                        let prompt = format!(
                                            " enter correct chapter [1 - {}]: ",
                                            max_chapter
                                        );
                                        x = prompt.len() as i32;
                                        prompt_len = prompt.len() as i32 + 1;
                                        addstr(prompt.as_str());
                                        ch = getch();
                                    }
                                }
                            }
                            _ => {
                                if ch == 127 {
                                    if x < prompt_len {
                                        x = prompt_len;
                                    }
                                    mvdelch(1, x - 1);
                                    x -= 1;
                                } else {
                                    x += 1;
                                    addch(ch as u32);
                                }
                                ch = getch();
                            }
                        }
                    }
                    noecho();
                    self.get_doc().await?;
                    self.draw_chapter_screen().await?;
                    break;
                }
                _ => {
                    ch = getch();
                }
            }
        }
        Ok(())
    }

    fn change_chapter(&mut self, offset: i32) {
        let result = self.url.clone();
        // split url into parts to get number and update it
        let mut result = result.split("/").collect::<Vec<_>>();
        let mut chapter_string = result[4].split("-").collect::<Vec<_>>();
        let mut chapter = chapter_string[1].split(".").collect::<Vec<_>>();
        let mut chapter_number = chapter[0].parse::<i32>().unwrap();
        chapter_number += offset;
        // combine the lists to 1 string and store it in self.url
        let chapter_number = chapter_number.to_string();
        chapter[0] = chapter_number.as_str();
        let chapter = chapter.join(".");
        chapter_string[1] = chapter.as_str();
        let chapter_string = chapter_string.join("-");
        result[4] = chapter_string.as_str();
        let result = result.join("/");
        self.url = result;
    }

    fn make_welcome_screen(&mut self) -> (Vec<String>, Vec<String>) {
        let mut result: Vec<String> = Vec::new();
        result.push("\n".to_string());
        result.push(" Recently Read Novels\n".to_string());
        result.push("\n".to_string());

        // read previously read light novels from text file
        let mut line: String = String::new();
        let mut chapter_urls = Vec::new();
        let file_buffer = File::options()
            .read(true)
            .open(&self.path)
            .expect("error reading config file");

        let mut reader = BufReader::new(file_buffer);
        loop {
            match reader.read_line(&mut line) {
                Ok(val) => {
                    if val != 0 {
                        chapter_urls.push(
                            line.split("#").collect::<Vec<&str>>()[1]
                                .to_string(),
                        );
                        let mut content =
                            line.split("#").collect::<Vec<&str>>()[0]
                                .to_string()
                                .trim()
                                .to_string();
                        if content.len() as i32 >= self.maxx {
                            let mut temp = content.clone();
                            temp = temp
                                .chars()
                                .take(self.maxx as usize - 9)
                                .collect();
                            temp.push_str("...");
                            content = temp;
                        }
                        line = format!("  *  {}", content);
                        line.push_str("\n");
                        result.push(line);
                        line = String::new();
                    } else {
                        break;
                    }
                }
                Err(err) => panic!("error read config file {}", err),
            }
        }
        self.doc = result.clone();
        (result, chapter_urls)
    }

    async fn make_search_screen(
        &mut self,
        keyword: Option<String>,
    ) -> Result<Option<Vec<[String; 3]>>, Box<dyn std::error::Error>> {
        let mut search_result = Vec::new();
        match keyword {
            Some(mut keyword) => {
                // trim keyword
                keyword.pop();
                keyword = keyword.trim().to_string();
                let url = "https://freewebnovel.com/search/";
                // pass a post request to get a response containing results
                let params = [("searchkey", keyword.as_str())];
                let client = reqwest::Client::new();
                let resp =
                    match client.post(url).form(&params).send().await {
                        Ok(x) => x.text().await?,
                        Err(_) => panic!("connection refused!")
                    };
                let fragment = Html::parse_fragment(&resp);

                // parse the document for data
                let title_div = Selector::parse(r#"div[class="txt"]"#).unwrap();
                let url_selector = Selector::parse(r#"h3"#).unwrap();
                let title_selector = Selector::parse(r#"a"#).unwrap();
                for element in fragment.select(&title_div) {
                    let title = element.select(&url_selector).next().unwrap();
                    let chapter_number =
                        Selector::parse(r#"a[class="chapter"]"#).unwrap();
                    let mut novel_data: [String; 3] =
                        ["".to_string(), "".to_string(), "".to_string()];
                    match title.select(&title_selector).next() {
                        Some(c) => {
                            let mut content =
                                c.value().attr("title").unwrap().to_string();
                            if content.len() as i32 >= self.maxx {
                                let mut temp = content.clone();
                                temp = temp
                                    .chars()
                                    .take(self.maxx as usize - 9)
                                    .collect();
                                temp.push_str("...");
                                content = temp;
                            }
                            novel_data[0] = content;
                        }
                        _ => {}
                    }
                    match element.select(&chapter_number).next() {
                        Some(c) => {
                            // add all data in a vector
                            let link = c.value().attr("href");
                            let chapter_number =
                                Selector::parse(r#"span[class="s1"]"#).unwrap();
                            let chapter = c
                                .select(&chapter_number)
                                .next()
                                .unwrap()
                                .text()
                                .collect::<Vec<_>>()[0]
                                .split(" ")
                                .collect::<Vec<_>>()[0];
                            novel_data[1] = link.unwrap().to_string();
                            novel_data[2] = chapter.to_string();
                        }
                        _ => {}
                    }
                    search_result.push(novel_data);
                }
                // result contains the data to be printed on the screen
                let mut result = Vec::new();
                result.push("\n".to_string());
                result.push(" Search for LightNovel\n".to_string());
                result.push("\n".to_string());
                result.push(" > \n".to_string());
                if search_result.len() >= 1 {
                    for value in search_result.iter() {
                        let mut line = format!("  *  {}", value[0]);
                        line.push_str("\n");
                        result.push(line);
                    }
                } else {
                    if keyword.len() < 3 {
                        result.push(
                            " Please enter more than 3 charachters!"
                                .to_string(),
                        );
                        result.push("\n".to_string());
                        result.push(" Press (q) to go back".to_string());
                    } else {
                        result.push(" No Light Novels Found!".to_string());
                        result.push("\n".to_string());
                        result.push(" Press (q) to go back".to_string());
                    }
                }
                self.doc = result;
                Ok(Some(search_result))
            }
            None => {
                let mut result = Vec::new();
                result.push("\n".to_string());
                result.push(" Search for LightNovels: \n".to_string());
                result.push("\n".to_string());
                result.push(" > ".to_string());
                result.push("\n".to_string());
                for (i, value) in search_result.into_iter().enumerate() {
                    let line = format!(" [{}] {}", i, value[0]);
                    result.push(line);
                }
                self.doc = result;
                Ok(None)
            }
        }
    }
}
