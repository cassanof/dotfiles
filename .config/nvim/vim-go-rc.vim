let g:go_fold_enable = ['import']

let g:go_highlight_operators = 1
let g:go_highlight_functions = 1
let g:go_highlight_types = 1
let g:go_highlight_extra_types = 1
let g:go_highlight_function_calls = 1
let g:go_highlight_fields = 1
let g:go_highlight_build_constraints = 1
let g:go_doc_keywordprg_enabled = 0

let g:go_info_mode = 'guru'

" dont need compl from vim-go
let g:go_code_completion_enabled = 0

let g:go_fmt_autosave = 1
let g:go_fmt_options = {
  \ 'gofmt': '-s',
  \ 'goimports': '',
  \ }

let g:go_imports_autosave = 1

