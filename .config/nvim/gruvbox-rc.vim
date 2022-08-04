let g:gruvbox_italic = 1
let g:gruvbox_contrast_dark = "hard"


" colorscheme
colorscheme gruvbox
syntax on

" tell vim how the background is
set background=dark


hi link Operator GruvboxOrange

" coc notification highlights
hi link CocNotificationProgress GruvboxGreen
hi link CocNotificationButton GruvboxOrange

" fixes CoC diagnostics window colors
hi Quote ctermbg=109 guifg=#83a598
  
" coc autocomplete
hi link CocMenuSel GruvboxGreen
hi link CocPumSearch GruvboxGreenBold
hi link CocPumDeprecated GruvboxRedBold
