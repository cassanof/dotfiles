" welcome to elleven11's vimrc!
" this vimrc works only when used with:
" https://github.com/amix/vimrc
" i use the awesome version,
" but the vimrc works in the basic one too

let mapleader = ","

" cursor blinkage
set guicursor=v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50,a:blinkwait1700-blinkoff400-blinkon950-Cursor/lCursor,sm:block,n:block-blinkon0

" move between splits
nmap <C-h> :wincmd h <CR>
nmap <C-j> :wincmd j <CR>
nmap <C-k> :wincmd k <CR>
nmap <C-l> :wincmd l <CR>

" remembers cursor location
set viminfo='100,\"2500,:200,%,n~/.viminfo

" syntax
syntax on

" system clipboard
set clipboard=unnamedplus

set nu

set smarttab
set cindent
set tabstop=2
set softtabstop=2
set shiftwidth=2
" always uses spaces instead of tab characters
set expandtab

" character limit column
set colorcolumn=80
highlight ColorColumn ctermbg=lightgrey guibg=lightgrey

set scrolloff=10

" don't continue to format comment tags after enter
set formatoptions-=cro

" more buffers open
set hidden 


" block cursor to | cursor in insert mode
let &t_SI = "\e[6 q"
let &t_EI = "\e[2 q"
" reset cursor on start
augroup myCmds
au!
autocmd VimEnter * silent !echo -ne "\e[2 q"
autocmd VimLeave * silent !echo -ne "\e[6 q"
augroup END
