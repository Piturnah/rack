" Vim syntax file
" Language: Rack

if exists("b:current_syntax")
    finish
endif

" TODOs
syntax keyword rackTodos TODO FIXME NOTE

" Keywords
syntax keyword rackKeyword in let peek do while if end fn ret
syntax keyword rackStack swap dup drop and or not
syntax keyword rackBool true false
syntax keyword rackInclude include

" Operators
syntax keyword rackOp + - / %

" Comments
syntax region rackCommentLine start="// " end="$" contains=rackTodos

" String literals
syntax region rackString start=/\v"/ end=/\v"/

" Number literals"
" TODO: numbers shouldn't be matched when in the middle of words
syntax match rackNumber /\(0x[0-f]\+\)\|\(0o[0-7]\+\)\|\(0b[01]\+\)\|[0-9]/
syntax region rackChar start=/\v'/ end=/\v'/

highlight default link rackTodos Todo
highlight default link rackKeyword Keyword
highlight default link rackStack Special
highlight default link rackInclude Include
highlight default link rackOp Operator
highlight default link rackBool Boolean
highlight default link rackCommentLine Comment
highlight default link rackString String
highlight default link rackChar Number
highlight default link rackNumber Number

let b:current_syntax = "rack"
