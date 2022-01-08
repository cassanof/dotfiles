let g:slime_target = "neovim"

let g:racket_hash_lang_dict = {
      \   'racket/base': 'racket',
      \   'typed/racket': 'racket',
      \   'typed/racket/base': 'racket',
      \   'br': 'racket',
      \   'plai': 'racket',
      \   'br/quicklang': 'racket',
      \   'scribble/base': 'scribble',
      \   'scribble/manual': 'scribble',
      \ }


" don't autopair single quote and quasi quote
let lispPairs = {'(':')', '[':']', '{':'}','"':'"'}
au Filetype racket let b:AutoPairs = lispPairs
au Filetype scheme let b:AutoPairs = lispPairs
au Filetype lisp let b:AutoPairs = lispPairs
au Filetype clojure let b:AutoPairs = lispPairs

au filetype racket set lisp
au filetype racket set autoindent
