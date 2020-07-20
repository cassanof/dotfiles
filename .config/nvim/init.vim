"
""" Main rc, for generic keybindings, settings and plugin loading
"

" Specify a directory for plugins call plug#begin('~/.nvim/plugged')
call plug#begin('~/.nvim/plugged')
Plug 'neoclide/coc.nvim', {'branch': 'release'}
Plug 'vim-airline/vim-airline'
Plug 'vim-airline/vim-airline-themes'
Plug 'tsony-tsonev/nerdtree-git-plugin'
Plug 'Xuyuanp/nerdtree-git-plugin'
Plug 'preservim/nerdtree'
Plug 'tiagofumo/vim-nerdtree-syntax-highlight'
Plug 'ryanoasis/vim-devicons'
Plug 'airblade/vim-gitgutter'
Plug 'scrooloose/nerdcommenter'
Plug 'fatih/vim-go', { 'do': ':GoUpdateBinaries' }

Plug 'morhetz/gruvbox'

" Initialize plugin system
call plug#end()

" colorscheme
colorscheme gruvbox

" real colors, needs patched urxvt to work right
set termguicolors

" sourcing rcs
source ~/.config/nvim/airline-rc.vim
source ~/.config/nvim/coc-rc.vim
source ~/.config/nvim/nerdtree-rc.vim
source ~/.config/nvim/vim-go-rc.vim

inoremap jj <ESC>
nmap <C-n> :NERDTreeToggle<CR>
" the _ key is actually / because nvim/vim does not recognize that!
vmap <C-_> <plug>NERDCommenterToggle
nmap <C-_> <plug>NERDCommenterToggle

" setting clipboard to system
set clipboard=unnamedplus


" move between splits
nmap <C-H> :wincmd h<CR>
nmap <C-J> :wincmd j<CR>
nmap <C-K> :wincmd k<CR>
nmap <C-L> :wincmd l<CR>

" j/k will move virtual lines (lines that wrap)
noremap <silent> <expr> j (v:count == 0 ? 'gj' : 'j')
noremap <silent> <expr> k (v:count == 0 ? 'gk' : 'k')

set relativenumber

set smarttab
set cindent
set tabstop=2
set softtabstop=2
set shiftwidth=2
" always uses spaces instead of tab characters
set expandtab

" keep undos in a file
set undofile
set undodir=~/.cache/vim/undo

" lines to keep cursor vertically centered
set scrolloff=10

" remember cursor location
set viminfo='100,\"2500,:200,%,n~/.cache/.viminfo


" Character 'limit' line
set colorcolumn=110
highlight ColorColumn ctermbg=lightgrey guibg=lightgrey


set hidden 
set updatetime=300

" don't give |ins-completion-menu| messages.
set shortmess+=c

" don't show signcolumn
set signcolumn=no

" Use tab for trigger completion with characters ahead and navigate.
" Use command ':verbose imap <tab>' to make sure tab is not mapped by other plugin.
inoremap <silent><expr> <TAB>
      \ pumvisible() ? "\<C-n>" :
      \ <SID>check_back_space() ? "\<TAB>" :
      \ coc#refresh()
inoremap <expr><S-TAB> pumvisible() ? "\<C-p>" : "\<C-h>"

function! s:check_back_space() abort
  let col = col('.') - 1
  return !col || getline('.')[col - 1]  =~# '\s'
endfunction
