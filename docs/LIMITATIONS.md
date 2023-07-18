# Known limitations

Since this is only an example, it is not production-ready. Therefore there are multiple limitations.

* The work with MP3 frames is cumbersome but since this is not MP3 app example, it should be fine.
* Streaming of MP3 files is slightly faster than their real-time playback time (receivers get slightly more MP3 data to play than was real broadcast time).
* The identity files are not stored in fault-tolerant fashion (if the machine crashes "at the right time", the identities may become corrupted).