use regex::Regex;

pub fn match_mask(input: &str, mask: &str) -> bool {
    let mut regex = mask.replace("*", ".*").replace("?", ".");
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
fn baz() {
    assert_eq!(true, match_mask("nick!username@host", "nick*"));
}
