" welcome to elleven11's vimrc!
" this vimrc works only when used with:
" https://github.com/amix/vimrc
" i use the awesome version,
" but the vimrc works in the basic one too

" most important! press jj to escape insert mode 
imap jj <Esc>

" remembers cursor location
set viminfo='100,\"2500,:200,%,n~/.viminfo

" line number stuff
set nu
set rnu
set cursorline
set cursorlineopt=number
autocmd ColorScheme * highlight CursorLineNr cterm=NONE term=bold gui=bold

" character limit column
set colorcolumn=80
highlight ColorColumn ctermbg=lightgrey guibg=lightgrey

" block cursor to | cursor in insert mode
let &t_SI = "\e[6 q"
let &t_EI = "\e[2 q"
" reset cursor on start
augroup myCmds
au!
autocmd VimEnter * silent !echo -ne "\e[2 q"
autocmd VimLeave * silent !echo -ne "\e[6 q"
augroup END


" 2 spaces tabs
set tabstop=2
set softtabstop=2
" when indenting with '>'
set shiftwidth=2

" Plugins in use:
" pathogen
" vim-go
" syntastic
" YouCompleteMe
execute pathogen#infect()

" syntastic stuff
set statusline+=%#warningmsg#
set statusline+=%{SyntasticStatuslineFlag()}
set statusline+=%*

let g:syntastic_always_populate_loc_list = 1
let g:syntastic_auto_loc_list = 1
let g:syntastic_check_on_open = 1
let g:syntastic_check_on_wq = 0
let g:syntastic_go_checkers = ['go', 'golint', 'errcheck']

" YCM stuff
let g:ycm_global_ycm_extra_conf = '~/.vim_runtime/.ycm_extra_conf.py'

" use goimports for formatting
let g:go_fmt_command = "goimports"

" turn highlighting on
let g:go_highlight_functions = 1
let g:go_highlight_methods = 1
let g:go_highlight_structs = 1
let g:go_highlight_operators = 1
let g:go_highlight_build_constraints = 1

"" Open go doc in vertical window, horizontal, or tab
au Filetype go nnoremap <leader>v :vsp <CR>:exe "GoDef" <CR>
au Filetype go nnoremap <leader>s :sp <CR>:exe "GoDef"<CR>
au Filetype go nnoremap <leader>t :tab split <CR>:exe "GoDef"<CR>"

