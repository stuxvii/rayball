```
 _ _   |_  _ ||
| (_|\/|_)(_|||
     /         
```
rayball is a remake of the H*xBall client with biased additions, built using raylib, rust, ezsockets and webrtc.

> [!WARNING]
> This project is currently on halt until someone is able to help.
## Information about the main issue and how you can help
When connecting to a room, the official H*xBall performs these steps:
1. Create an RTC Peer Connection with an ICE STUN server (in this case Google's)
2. Gather ICE candidates into an array
3. Create data channels "ro" (ordered: true, reliable: true), "ru" (ordered: false, reliable: true), "uu" (ordered: false, reliable: false)
4. Create an offer, generate a local description and set it
5. Create a new binary encoder (for version 3f23b1af of the H*xBall game code, it is the A class)
6. Now just create the WebRTC handshake message
   - Write u8 integer for 0x00 (header)
   - Write string for the SDP of the local description created in step 4
   - Write json for the array of candidates gathered in step 2
   - Write u16 int corresponding to the game's version (most commonly, version 9)
   - Write nullable string for the password
   - Compress it with pako's deflateRaw (it's just raw deflate, available in all programming languages)
   - append a 0x01 byte and the previously compressed data
7. Open a WebSocket to "wss://p2p.haxball.com/client?id=Ur_R0omCod3"
8. Send the WebRTC handshake msg created in step 6 through it
9. Wait on the server to return a string of bytes, which also contains their description's SDP and list of available candidates (read it using the binary decoder from H*xBall, first chunk is a string with their SDP, the second is a JSON block with their ICE candidates)
10. Create a RTCSessionDescription answer containing their SDP
10. Set the remote description to that description
11. Add all of their ICE candidates to the peer connection object
(At this stage, you've successfully connected to the room and your data channels will have been opened) The host will send an array of bytes through the ro data channel.
This part is where the rayball client is stuck. The array of bytes for a challenge is never ever sent. 
13. Sign received challenge bytes with your id.key
14. Now just create the join + auth handshake message (w/binary encoder)
    - Write u8 integer for 0x00 (header)
    - Write string for the x, url encoded
    - Write string for the y, url encoded
    - Append signed challenge bytes
    - Write string for your username
    - Write string for your country
    - Write nullable string for your avatar
15. Send it back through the ro data channel

Any help investigating this in depth will be heavily appreciated.
Relevant joining code is located in [the join portion of the code](src/net/join.rs)
