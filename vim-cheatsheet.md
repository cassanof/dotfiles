# Vim keys personal cheatsheet (related to my config)
listing only basic keys, so keystrokes like t, q, and other complex keys are missing

### Modes
Esc or jj | go back to normal mode  
i | go to insert mode  
v | visual mode  
Ctrl+v | visual block mode  
Shift+v | visual line mode  
Shift+r | go to replace mode (like the INS key in a normal text editor)  
/ | go to search mode  
:s/-word-/-new word-/-options- | replace word with this word  
: | command mode  
:! | sh command mode
Ctrl+n | open and go to NERDTREE split  
Ctrl+b | open and go to tagbar split  
Ctrl+s | save file  
F5 | run/compile file  
. | repeat last action  


### Navigation
h | go to char left  
j | go to char down  
k | go to char up  
l | go to char right  
w | go to next word  
e | go to last char of the word  
b | go to previous word  
f-letter- | go to first letter specified  
-number-gg | go to line number, if no line specified go to line 1  
Shift-g | go to end of file  
$ | go to last char in line  
0 or ^ | go to first char in line  
Shift+h | go to high section of the screen  
Shift+m | go to mid section of the screen  
Shift+l | go to low section of the screen  
-leader-leader-w | easymotion to all words  
-leader-leader-s | easymotion to selected char  
Ctrl+Shift+h | go to split left  
Ctrl+Shift+j | go to split down  
Ctrl+Shift+k | go to split up  
Ctrl+Shift+l | go to split right  

### Modification
a | insert mode after cursor  
Shift+a | insert mode at last word of line  
Shift+i | insert mode at first word of the line  
o | insert mode in new line after down  
Shift+o | insert mode in new line up  
x | delete char under cursor  
Shift+x | delete char before cursor  
s | delete char and insert mode  
Shift+s | delete line and insert mode  
r | replaces char under cursor  
c-mod- | changes something, if -mod- is iw it changes the word, etc...  
Shift+c or cc | deletes line and insert  
dd | delete whole line  
dw | delete word forward  
db | delete word backward  
Shift+d | delete every char in line after cursor  
y | copy selection  
yy | copy whole line  
Shift+y | copy every char in line after cursor  
p | paste  
Shift+p | paste backward  
"-key-y | copy into registry -key-  
"-key-p | paste whats into the registry -key-  
u | undo  
Ctrl+r | redo  
cs-letter- | removes -letter- sorrounding cursor (vim-sorround)  
cs-letter--newletter- | replaces sorrounding -letter- with -newletter-  
ysiw-letter- | puts -letter- sorrounding the word the cursor is on  
yssb | puts parenthesis sorrounding whole line  
yss-letter- | pust -letter- sorrouding whole line  
