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
  ;; Op::PushInt(4) - ../tests/test.porth:1:1
  mov rax, 4
  push rax
  ;; Op::PushStrPtr(0) - ../tests/test.porth:1:1
	push str_0
	add rsp, 8
	add rsp, 8
	pop rdi
	call print
  mov rax, 60
  mov rdi, 0
  syscall
segment readable
str_0: db 116,101,115,116
