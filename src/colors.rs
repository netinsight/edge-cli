#[macro_export]
macro_rules! red {
    ($text:expr) => {
        if atty::is(atty::Stream::Stdout) {
            format!("\x1b[31m{}\x1b[0m", $text)
        } else {
            format!("{}", $text)
        }
    };
}

#[macro_export]
macro_rules! green {
    ($text:expr) => {
        if atty::is(atty::Stream::Stdout) {
            format!("\x1b[32m{}\x1b[0m", $text)
        } else {
            format!("{}", $text)
        }
    };
}

#[macro_export]
macro_rules! yellow {
    ($text:expr) => {
        if atty::is(atty::Stream::Stdout) {
            format!("\x1b[33m{}\x1b[0m", $text)
        } else {
            format!("{}", $text)
        }
    };
}

#[macro_export]
macro_rules! grey {
    ($text:expr) => {
        if atty::is(atty::Stream::Stdout) {
            format!("\x1b[37m{}\x1b[0m", $text)
        } else {
            format!("{}", $text)
        }
    };
}
