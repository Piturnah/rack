format ELF64 executable 3
entry main
segment readable executable
print:
  mov     r9, -3689348814741910323
  sub     rsp, 40
  mov     BYTE [rsp+31], 10
  lea     rcx, [rsp+30]
.L2:
  mov     rax, rdi
  lea     r8, [rsp+32]
  mul     r9
  mov     rax, rdi
  sub     r8, rcx
  shr     rdx, 3
  lea     rsi, [rdx+rdx*4]
  add     rsi, rsi
  sub     rax, rsi
  add     eax, 48
  mov     BYTE [rcx], al
  mov     rax, rdi
  mov     rdi, rdx
  mov     rdx, rcx
  sub     rcx, 1
  cmp     rax, 9
  ja      .L2
  lea     rax, [rsp+32]
  mov     edi, 1
  sub     rdx, rax
  xor     eax, eax
  lea     rsi, [rsp+32+rdx]
  mov     rdx, r8
  mov     rax, 1
  syscall
  add     rsp, 40
  ret
main:
  ;; Op::PushInt(10) - tests/let.porth:1:1
  mov rax, 10
  push rax
  ;; Op::PushInt(20) - tests/let.porth:1:4
  mov rax, 20
  push rax
  ;; Op::PushInt(30) - tests/let.porth:1:7
  mov rax, 30
  push rax
  ;; Op::Let - tests/let.porth:2:1
  ;; Op::In
  ;; Op::PushBinding(2) - tests/let.porth:3:3
  add rsp, 16
  pop rax
  sub rsp, 16
  sub rsp, 8
  push rax
  ;; Op::Print - tests/let.porth:3:5
  pop rdi
  call print
  ;; Op::PushBinding(1) - tests/let.porth:4:3
  add rsp, 8
  pop rax
  sub rsp, 8
  sub rsp, 8
  push rax
  ;; Op::Print - tests/let.porth:4:5
  pop rdi
  call print
  ;; Op::PushBinding(0) - tests/let.porth:5:3
  add rsp, 0
  pop rax
  sub rsp, 0
  sub rsp, 8
  push rax
  ;; Op::Print - tests/let.porth:5:5
  pop rdi
  call print
  ;; Op::Drop - tests/let.porth:6:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:6:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:6:1
  add rsp, 8
  ;; Op::PushInt(1) - tests/let.porth:7:1
  mov rax, 1
  push rax
  ;; Op::PushStrPtr(0) - tests/let.porth:7:1
  push str_0
  ;; Op::Puts - tests/let.porth:7:6 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  ;; Op::PushInt(1) - tests/let.porth:9:1
  mov rax, 1
  push rax
  ;; Op::PushStrPtr(1) - tests/let.porth:9:1
  push str_1
  ;; Op::PushInt(1) - tests/let.porth:9:5
  mov rax, 1
  push rax
  ;; Op::PushStrPtr(2) - tests/let.porth:9:5
  push str_2
  ;; Op::PushInt(1) - tests/let.porth:9:9
  mov rax, 1
  push rax
  ;; Op::PushStrPtr(3) - tests/let.porth:9:9
  push str_3
  ;; Op::Let - tests/let.porth:10:1
  ;; Op::In
  ;; Op::PushBinding(4) - tests/let.porth:11:3
  add rsp, 32
  pop rax
  sub rsp, 32
  sub rsp, 8
  push rax
  ;; Op::PushBinding(5) - tests/let.porth:11:6
  add rsp, 40
  pop rax
  sub rsp, 40
  sub rsp, 8
  push rax
  ;; Op::Puts - tests/let.porth:11:9 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  ;; Op::Drop - tests/let.porth:14:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:14:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:14:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:14:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:14:1
  add rsp, 8
  ;; Op::Drop - tests/let.porth:14:1
  add rsp, 8
  ;; Op::PushInt(1) - tests/let.porth:15:1
  mov rax, 1
  push rax
  ;; Op::PushStrPtr(4) - tests/let.porth:15:1
  push str_4
  ;; Op::Puts - tests/let.porth:15:6 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
  mov rax, 60
  mov rdi, 0
  syscall
segment readable
str_0: db 10
str_1: db 97
str_2: db 98
str_3: db 99
str_4: db 10
