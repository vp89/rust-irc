# TODO

## Working on

## Functionality/Correctness
- Pass PINGPONG tests in irctest suite and add missing coverage

## Testing
- Look into how to build automated "end to end" tests to test things like connection shutdown behavior

## Connection handling
- Should non-graceful disconnect send QUIT anyway?
- Verify that threads/sockets are not leaking, ie. connections are fully shutdown even when QUIT not received

## Efficiency

## Error handling

## Code organization
- Read up on best practices RE: structuring modules and refactor accordingly 

## Configurability