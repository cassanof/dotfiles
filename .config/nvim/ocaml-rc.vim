
" don't pair single quote
let ocamlPairs = {'(':')', '[':']', '{':'}','"':'"','`':'`'}

let g:opamshare = substitute(system('opam var share'),'\n$','','''')
execute "set rtp+=" . g:opamshare . "/merlin/vim"


execute "helptags " . g:opamshare . "/merlin/vim/doc"

au Filetype ocaml let b:AutoPairs = ocamlPairs
