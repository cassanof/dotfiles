let g:gruvbox_italic = 1
let g:gruvbox_contrast_dark = "hard"


" colorscheme
colorscheme gruvbox
syntax on

" tell vim how the background is
set background=dark


hi link Operator GruvboxOrange

" fixes CoC diagnostics window colors
hi Quote ctermbg=109 guifg=#83a598
