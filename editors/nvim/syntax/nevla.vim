" nevla syntax highlighting: lexical classes only, mirrors src/lexer.rs.
if exists("b:current_syntax")
  finish
endif

syn keyword nevlaKeyword fn struct import return if else for range break continue check with
syn keyword nevlaType int float bool str error py map
syn keyword nevlaBoolean true false
syn keyword nevlaConstant none
syn keyword nevlaBuiltin print printf sprintf len append ord chr args input

syn match nevlaComment "//.*$" contains=@Spell
" the lexer's escapes are exactly \n \t \" \\; strings are one line
syn match nevlaEscape contained +\\[nt"\\]+
syn region nevlaString start=+"+ skip=+\\"+ end=+"+ oneline contains=nevlaEscape
syn match nevlaNumber "\<\d\+\%(\.\d\+\)\?\>"
syn match nevlaOperator ":=\|==\|!=\|<=\|>=\|&&\|||\|[-+*/%@!<>?=]"
" name in a fn declaration
syn match nevlaFuncDef "\%(\<fn\s\+\)\@<=\w\+"

hi def link nevlaKeyword Keyword
hi def link nevlaType Type
hi def link nevlaBoolean Boolean
hi def link nevlaConstant Constant
hi def link nevlaBuiltin Function
hi def link nevlaComment Comment
hi def link nevlaString String
hi def link nevlaEscape SpecialChar
hi def link nevlaNumber Number
hi def link nevlaOperator Operator
hi def link nevlaFuncDef Function

let b:current_syntax = "nevla"
