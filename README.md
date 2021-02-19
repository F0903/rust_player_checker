# Rust Player Checker

Checks if a player is playing on a specific Rust (the game) server, then plays an audio cue.

## Commands

### **--listen** _address_ **-u** _steam-username_

Listens for the specified player, and plays a sound when found.

### **--dump** _address_

Dumps all player-info to a file called dbg_dump.txt

### **--print** _address_

Prints every player in server to console.

## Variables

### \*recent

Contains the last server address used. (stored in recent.txt)

_usage example: --print *recent_
