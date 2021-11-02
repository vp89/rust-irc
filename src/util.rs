use regex::Regex;

// TODO work in progress, this will be used by the WHO command
// to find users
pub fn match_mask(input: &str, mask: &str) -> bool {
    let mut regex = mask.replace("*", ".*").replace("?", ".");
    regex.push('$');

    let re = match Regex::new(&regex) {
        Ok(re) => re,
        // TODO
        Err(e) => return false
    };

    re.is_match(input)
}

#[test]
fn foo() {
    assert_eq!(false, match_mask("nick!username@host", "nick"));
}

#[test]
fn baz() {
    assert_eq!(true, match_mask("nick!username@host", "nick*"));
}
