use std::{fmt, str::FromStr};

#[derive(Debug)]
pub struct IrcMessage {
    pub prefix: Option<IrcMessagePrefix>,
    pub command: String,
    pub params: String
}

/*
<message>  ::= [':' <prefix> <SPACE> ] <command> <params> <crlf>
<prefix>   ::= <servername> | <nick> [ '!' <user> ] [ '@' <host> ]
<command>  ::= <letter> { <letter> } | <number> <number> <number>
<SPACE>    ::= ' ' { ' ' }
<params>   ::= <SPACE> [ ':' <trailing> | <middle> <params> ]
<middle>   ::= <Any *non-empty* sequence of octets not including SPACE
               or NUL or CR or LF, the first of which may not be ':'>
<trailing> ::= <Any, possibly *empty*, sequence of octets not including
                 NUL or CR or LF>
<crlf>     ::= CR LF

1)  <SPACE> is consists only of SPACE character(s) (0x20).
    Specially notice that TABULATION, and all other control
    characters are considered NON-WHITE-SPACE.

2)  After extracting the parameter list, all parameters are equal,
    whether matched by <middle> or <trailing>. <Trailing> is just
    a syntactic trick to allow SPACE within parameter.

3)  The fact that CR and LF cannot appear in parameter strings is
    just artifact of the message framing. This might change later.

4)  The NUL character is not special in message framing, and
    basically could end up inside a parameter, but as it would
    cause extra complexities in normal C string handling. Therefore
    NUL is not allowed within messages.

5)  The last parameter may be an empty string.

6)  Use of the extended prefix (['!' <user> ] ['@' <host> ]) must
    not be used in server to server communications and is only
    intended for server to client messages in order to provide
    clients with more useful information about who a message is
    from without the need for additional queries.

Most protocol messages specify additional semantics and syntax for
the extracted parameter strings dictated by their position in the
list. For example, many server commands will assume that the first
parameter after the command is the list of targets, which can be
described with:

<target>     ::= <to> [ "," <target> ]
<to>         ::= <channel> | <user> '@' <servername> | <nick> | <mask>
<channel>    ::= ('#' | '&') <chstring>
<servername> ::= <host>
<host>       ::= see RFC 952 [DNS:4] for details on allowed hostnames
<nick>       ::= <letter> { <letter> | <number> | <special> }
<mask>       ::= ('#' | '$') <chstring>
<chstring>   ::= <any 8bit code except SPACE, BELL, NUL, CR, LF and
                    comma (',')>

Other parameter syntaxes are:

<user>       ::= <nonwhite> { <nonwhite> }
<letter>     ::= 'a' ... 'z' | 'A' ... 'Z'
<number>     ::= '0' ... '9'
<special>    ::= '-' | '[' | ']' | '\' | '`' | '^' | '{' | '}'
<nonwhite>   ::= <any 8bit code except SPACE (0x20), NUL (0x0), CR
                    (0xd), and LF (0xa)>
*/
impl std::str::FromStr for IrcMessage {
    type Err = (); // TODO?

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let has_prefix = s.starts_with(':');
        let words = s.split_whitespace();

        let prefix = if has_prefix {
            let raw_prefix = words.clone().next().unwrap(); // TODO
            let foo = IrcMessagePrefix::from_str(raw_prefix)?;
            Some(foo)
        } else {
            None
        };

        let message = IrcMessage {
            prefix: None,
            command: format!("FOO"),
            params: format!("BAR")
        };

        Ok(message)
    }
}

#[derive(Debug)]
pub struct IrcMessagePrefix {
    prefix: String, // TODO this should be an enum
    user: Option<String>,
    host: Option<String>
}

impl std::str::FromStr for IrcMessagePrefix {
    type Err = (); // TODO?

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefix = IrcMessagePrefix {
            prefix: format!("FOO"),
            user: None,
            host: None
        };
        
        Ok(prefix)
    }
}

#[test]
fn test_one() {
    let raw_str = "NICK vince";
    let parsed_message = IrcMessage::from_str(raw_str).expect("foo");
}
