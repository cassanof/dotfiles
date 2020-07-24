# Vim keys personal cheatsheet (related to my config)
listing only basic keys, so keystrokes like t, q, and other complex keys are missing

### Modes
Esc or jj | go back to normal mode  
i | go to insert mode  
Ctrl+v | visual block mode  
Shift+v | visual line mode  
Shift+r | go to replace mode  
/ | go to search mode  
: | command mode  
:! | sh command mode
Ctrl+n | open and go to NERDTREE split  
F5 | run/compile file  


### Navigation
h | go to char left  
j | go to char down  
k | go to char up  
l | go to char right  
w | go to next word  
b | go to previous word  
f-letter- | go to first letter specified  
-number-gg | go to line number, if no line specified go to line 1  
Shift-g | go to end of file  
$ | go to last char in line  
0 or ^ | go to first char in line  
\\w | easymotion to all words  
\\s | easymotion to selected char  
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
s | delete char and insert mode  
Shift+s | delete line and insert mode  
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
cs-letter- | removes -letter- sorrounding cursor (vim-sorround)  
cs-letter--newletter- | replaces sorrounding -letter- with -newletter-  
ysiw-letter- | puts -letter- sorrounding the word the cursor is on  
yssb | puts parenthesis sorrounding whole line  
yss-letter- | pust -letter- sorrouding whole line  
