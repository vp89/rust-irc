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
fn foo() {
    assert_eq!(false, match_mask("nick!username@host", "nick"));
}

#[test]
fn foo2() {
    assert_eq!(false, match_mask("nick!username@host", "?"));
}

#[test]
fn foo3() {
    assert_eq!(true, match_mask("nick!username@host", "*"));
}

#[test]
fn baz() {
    assert_eq!(true, match_mask("nick!username@host", "nick*"));
}
