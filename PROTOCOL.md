Channel list
---
REQ -> LIST >4,<10000
RES -> :<source> 321 nick Channel :Users Name
(1 for each channel)
RES -> :<source> 322 nick #channel 123 :<something?>
RES -> :<source> 323 nick :End of /LIST

Joining a channel
---
REQ -> JOIN #channel
RES -> :<source> JOIN :#channel

REQ -> MODE #channel
RES -> :<source> 332 nick #channel :<STUFF>
RES -> :<source> 333 nick #channel someone? number13432523
RES -> :<source> 353 nick = #channel :listofusers with @
RES -> :<source> 366 nick #channel :End of /NAMES list.

REQ -> :<source> WHO #channel
RES -> :<source> 324 nick #channel +mtn1 100
RES -> :<source> 329 nick #channel number

(1 of these for each user)
RES -> :<source> 352 nick #channel user-nick whattheyconnectedto nick H :0 something??
RES -> 315 nick #channel :End of /WHO list.

Leaving a channel
---
PART #channel :Leaving
:<source> PART #channel

:vince!~vince@<hostname>
