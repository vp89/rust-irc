use regex::Regex;

pub fn match_mask(input: &str, mask: &str) -> bool {
    let mut regex = String::from("^");
    regex.push_str(&mask.replace("*", ".*").replace("?", "."));
    regex.push('$');

    let re = match Regex::new(&regex) {
        Ok(re) => re,
        Err(e) => {
            println!("Error building regex {} {} {:?}", mask, regex, e);
            return false;
        }
    };

    re.is_match(input)
}

#[test]
fn match_mask_prefix_matches_no_wildcard_no_match() {
    assert_eq!(false, match_mask("nick!username@host", "nick"));
}

#[test]
fn match_mask_single_char_wildcard_multi_char_mask_no_match() {
    assert_eq!(false, match_mask("nick!username@host", "?"));
}

#[test]
fn match_mask_wildcard_matches() {
    assert_eq!(true, match_mask("nick!username@host", "*"));
}

#[test]
fn match_mask_prefix_with_wildcard_matches() {
    assert_eq!(true, match_mask("nick!username@host", "nick*"));
}
