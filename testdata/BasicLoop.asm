@256
D=A
@SP
M=D


@0
D=A
@SP
A=M
M=D
@SP
M=M+1

@LCL
D=M
@13
M=D
@0
D=A
@13
M=D+M
@SP
M=M-1
A=M
D=M
@13
A=M
M=D

(LOOP)

@0
D=A
@ARG
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1

@0
D=A
@LCL
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1

@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M=D+M
@SP
M=M+1

@LCL
D=M
@13
M=D
@0
D=A
@13
M=D+M
@SP
M=M-1
A=M
D=M
@13
A=M
M=D

@0
D=A
@ARG
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1

@1
D=A
@SP
A=M
M=D
@SP
M=M+1

@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M=M-D
@SP
M=M+1

@ARG
D=M
@13
M=D
@0
D=A
@13
M=D+M
@SP
M=M-1
A=M
D=M
@13
A=M
M=D

@0
D=A
@ARG
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1

@SP
M=M-1
A=M
D=M
@LOOP
D;JNE
@0
D=A
@LCL
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1

