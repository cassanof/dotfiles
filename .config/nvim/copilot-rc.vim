" copilot disable tab, use <leader>c on insert mode
imap <silent><script><expr> <leader>c copilot#Accept("\<CR>")
let g:copilot_no_tab_map = v:true
let g:copilot_filetypes = {
   \ 'markdown': v:true,
   \ }
