#!/bin/sh
# Change to the directory displayed in the title of the current (first) tab.
# XEMBED contains the window id for tabbed(1).
cd "$(xwininfo -children -id "$XEMBED" | sed '
    # Find the first child window description line.
    /^  *0x[0-9a-f]* "\([^"]*\)".*/!d
    # The title of the first child.
    s//\1/
    # Get the directory displayed in the title.
    s/.*://
    # Convert ~/... to $HOME/...
    s@^~/@'"$HOME"'/@
    # Print only this line.
    q
')"
exec st -w "$XEMBED"
