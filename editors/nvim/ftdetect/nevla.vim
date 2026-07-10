au BufRead,BufNewFile *.nv setfiletype nevla
" nv shebang scripts have no extension
au BufRead,BufNewFile * if getline(1) =~# '^#!.*\<nv\>$' | setfiletype nevla | endif
