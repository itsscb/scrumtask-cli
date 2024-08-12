use ellipse::Ellipse;

pub fn get_column_string(text: &str, width: usize) -> String {
    match width {
        0 => return String::new(),
        1 => return ".".to_owned(),
        2 => return "..".to_owned(),
        3 => return "...".to_owned(),
        _ => {}
    }

    let length = text.len();

    if width == length {
        return text.to_owned();
    }

    if width > length {
        let diff = " ".repeat(width - length);
        let mut out = text.to_owned();
        out.push_str(&diff);

        return out;
    }

    text.truncate_ellipse(width - 3).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_column_string() {
        let text1 = "";
        let text2 = "test";
        let text3 = "testme";
        let text4 = "testmetest";

        let width = 0;

        assert_eq!(get_column_string(text4, width), "".to_owned());

        let width = 1;

        assert_eq!(get_column_string(text4, width), ".".to_owned());

        let width = 2;

        assert_eq!(get_column_string(text4, width), "..".to_owned());

        let width = 3;

        assert_eq!(get_column_string(text4, width), "...".to_owned());

        let width = 4;

        assert_eq!(get_column_string(text4, width), "t...".to_owned());

        let width = 6;

        assert_eq!(get_column_string(text1, width), "      ".to_owned());
        assert_eq!(get_column_string(text2, width), "test  ".to_owned());
        assert_eq!(get_column_string(text3, width), "testme".to_owned());
        assert_eq!(get_column_string(text4, width), "tes...".to_owned());
    }
}
