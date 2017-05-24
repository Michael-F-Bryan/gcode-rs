; CNC Milling programming example code with drawing, which shows how G41 
; Cutter Radius Compensation Left is used in a cnc mill program
;
; From: http://www.helmancnc.com/cnc-mill-program-g41-cutter-radius-compensation-left/
N10 T2 M3 S447 F80
N20 G0 X112 Y-2
N30 Z-5
N40 G41
N50 G1 X95 Y8 M8
N60 X32
N70 X5 Y15
N80 Y52
N90 G2 X15 Y62 I10 J0
N100 G1 X83
N110 G3 X95 Y50 I12 J0
N120 G1 Y-12
N130 G40
N140 G0 Z100 M9
N150 X150 Y150
N160 M30
