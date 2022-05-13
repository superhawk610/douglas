# Changelog

## 0.1.1 - May 13, 2022

Improved renderer!

The renderer now performs line-by-line diffing (if a line hasn't changed since
the previous render, it's not updated), and also writes updates to an internal
buffer that's flushed 60 times per second, instead of always flushing updates
directly to the screen.

This also fixes a bug where the topmost line of text wouldn't be properly
cleared between renders.

## 0.1.0 - May 13, 2022

Initial release.
