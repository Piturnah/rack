format ELF64 executable 3
entry main
segment readable executable
main:
  ;; Op::PushInt(50) - ../tests/if.porth:4:1
  mov rax, 50
  push rax
  ;; Op::PushInt(8) - ../tests/if.porth:4:4
  mov rax, 8
  push rax
  ;; Op::Minus - ../tests/if.porth:4:6
  pop rbx
  pop rax
  sub rax, rbx
  push rax
  ;; Op::PushInt(42) - ../tests/if.porth:4:8
  mov rax, 42
  push rax
  ;; Op::Equals - ../tests/if.porth:4:11
  pop rax
  pop rbx
  cmp rax, rbx
  je J0
  push 0
  jmp J1
J0:
  push 1
J1:
  ;; Op::If - ../tests/if.porth:4:13
  pop rax
  cmp rax, 1
  jne F3
  ;; Op::PushInt(5) - ../tests/if.porth:5:3
  mov rax, 5
  push rax
  ;; Op::PushStrPtr(3) - ../tests/if.porth:5:3
  push str_3
  ;; Op::Puts - ../tests/if.porth:5:12 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  ;; Op::PushInt(1) - ../tests/if.porth:7:3
  mov rax, 1
  push rax
  ;; Op::If - ../tests/if.porth:7:8
  pop rax
  cmp rax, 1
  jne F1
  ;; Op::PushInt(0) - ../tests/if.porth:8:6
  mov rax, 0
  push rax
  ;; Op::If - ../tests/if.porth:8:12
  pop rax
  cmp rax, 1
  jne F0
  ;; Op::PushInt(6) - ../tests/if.porth:9:8
  mov rax, 6
  push rax
  ;; Op::PushStrPtr(4) - ../tests/if.porth:9:8
  push str_4
  ;; Op::Puts - ../tests/if.porth:9:18 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  ;; Op::End(If) - ../tests/if.porth:10:6
F0:
  ;; Op::PushInt(10) - ../tests/if.porth:12:6
  mov rax, 10
  push rax
  ;; Op::PushStrPtr(5) - ../tests/if.porth:12:6
  push str_5
  ;; Op::Puts - ../tests/if.porth:12:20 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  ;; Op::End(If) - ../tests/if.porth:13:3
F1:
  ;; Op::PushInt(8) - ../tests/if.porth:15:3
  mov rax, 8
  push rax
  ;; Op::PushInt(2) - ../tests/if.porth:15:5
  mov rax, 2
  push rax
  ;; Op::Equals - ../tests/if.porth:15:7
  pop rax
  pop rbx
  cmp rax, rbx
  je J2
  push 0
  jmp J3
J2:
  push 1
J3:
  ;; Op::If - ../tests/if.porth:15:9
  pop rax
  cmp rax, 1
  jne F2
  ;; Op::PushInt(6) - ../tests/if.porth:16:5
  mov rax, 6
  push rax
  ;; Op::PushStrPtr(6) - ../tests/if.porth:16:5
  push str_6
  ;; Op::Puts - ../tests/if.porth:16:15 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  ;; Op::End(If) - ../tests/if.porth:17:3
F2:
  ;; Op::End(If) - ../tests/if.porth:19:1
F3:
  mov rax, 60
  mov rdi, 0
  syscall
segment readable
str_0: db 116,114,117,101
str_1: db 97,108,115,111,32,116,114,117,101
str_2: db 102,97,108,115,101
str_3: db 116,114,117,101,10
str_4: db 102,97,108,115,101,10
str_5: db 97,108,115,111,32,116,114,117,101,10
str_6: db 102,97,108,115,101,10
