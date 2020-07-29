
function! DeleteCurrentBuffer() abort
    let current_buffer = bufnr('%')
    let next_buffer = g:vem_tabline#tabline.get_replacement_buffer()
    try
        exec 'confirm ' . current_buffer . 'bdelete'
        if next_buffer != 0
            exec next_buffer . 'buffer'
        endif
    catch /E516:/
       " If the operation is cancelled, do nothing
    endtry
endfunction
nmap <leader>q :call DeleteCurrentBuffer()<CR>

nmap <leader>h <Plug>vem_prev_buffer-
nmap <leader>l <Plug>vem_next_buffer-
