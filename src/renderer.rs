use libc;

pub fn addstr(str: String) -> Result<(), ()> {
    Ok(())
}

pub fn clear() -> Result<(), ()> {
    Ok(())
}

pub fn get_screen_size() -> (u16, u16) {
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size as *mut _);
        (size.ws_xpixel, size.ws_ypixel)
    }
}

pub fn get_term_dim() -> (u16, u16) {
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size as *mut _);
        (size.ws_row, size.ws_col)
    }
}

pub fn move_cursor(str: String) -> Result<(), ()> {
    Ok(())
}

pub fn get_cursor_position(str: String) -> Result<(), ()> {
    Ok(())
}

pub fn toggle_cursor() -> Result<(), ()> {
    Ok(())
}

pub enum Attributes {
    Strike,
    Bold,
    Italic,
    Underline,
}
