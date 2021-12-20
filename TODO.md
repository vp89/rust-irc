# TODO

## Working on

## Functionality/Correctness
- Pass PINGPONG tests in irctest suite and add missing coverage
- All missing parameter errors should be observed in the message handlers not in the message parsing
    - PRIVMSG

## Testing
- Look into how to build automated "end to end" tests to test things like connection shutdown behavior

## Connection handling
- Should non-graceful disconnect send QUIT anyway?
- Verify that threads/sockets are not leaking, ie. connections are fully shutdown even when QUIT not received

## Efficiency
- Look at async to keep thread usage down (currently 2+2N where N is # of connections)
- Can we reduce file descriptor usage per connection from 3 -> 2 or even 1?

## Error handling

## Code organization
- Read up on best practices RE: structuring modules and refactor accordingly 

## Configurability
- Make current server params configurable via file?
