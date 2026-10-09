# Game log encoding

The launcher reads the game's stdout and stderr as text. Rust treats those
bytes as UTF-8. Java 18 and newer print UTF-8. An older Java on Windows prints
the ANSI code page, which on this machine is Windows-1252.

A strict UTF-8 read stops at the first byte that is not UTF-8. On 1.5.2 that
byte is in the menu line `INFORMAÇÕES`: `Ç` is the byte `0xC7` in Windows-1252,
and that byte is not a valid UTF-8 sequence. The game had already reached the
menu. The launcher still reported `stream did not contain valid UTF-8` and
exited 1.

## What the reader does

One codec covers the whole process, both pipes. The major comes from
`javaVersion` for the Temurin copy, and from `java -version` when `--java` is
passed. Java 18 and newer are decoded as UTF-8. An older Java uses the system
code page.

On Windows that page is `GetACP()`, a single call with no arguments. Page 1252
is Windows-1252, so `0xE7 0xE3` stays `ção`. Page 65001 is UTF-8. Any other
page is read as lossy UTF-8. Linux does not ask for a code page. An older Java
there is also lossy UTF-8.

A byte that does not fit the chosen codec becomes U+FFFD. The reader does not
return an error, and the next line is still kept. Five Windows-1252 bytes in
`0x80`–`0x9F` are undefined in that page and become U+FFFD too.

The launch does not pass `-Dfile.encoding`. That flag would change the charset
inside the game. The fix only changes how Licht reads the pipes.

## What is still approximate

Java 17 and older use one codec for every line. A library that prints a UTF-8
accent into that same process can be misread, because the line is decoded as
Windows-1252. A code page other than 1252 or 65001 is not decoded as that
page. Those bytes become U+FFFD when they are not valid UTF-8.
