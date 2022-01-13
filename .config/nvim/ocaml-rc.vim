
" don't pair single quote
let ocamlPairs = {'(':')', '[':']', '{':'}','"':'"','`':'`'}

au Filetype ocaml let b:AutoPairs = ocamlPairs
