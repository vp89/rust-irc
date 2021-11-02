use regex::Regex;

// TODO work in progress, this will be used by the WHO command
// to find users
pub fn match_mask(input: &str, mask: &str) -> bool {
    let re = match Regex::new(mask) {
        Ok(re) => re,
        // TODO
        Err(e) => return false
    };

    re.is_match(input)    
}

#[test]
fn foo() {
    // <nick>!<username>@<host>
    assert_eq!(true, match_mask("nick!username@host", "nick*"));
}
