; Simple G Code Example Mill
; From: http://www.helmancnc.com/simple-g-code-example-mill-g-code-programming-for-beginners/
O1000
T1 M6
(Linear / Feed - Absolute)
G0 G90 G40 G21 G17 G94 G80
G54 X-75 Y-75 S500 M3  (Position 6)
G43 Z100 H1
Z5
G1 Z-20 F100
X-40                   (Position 1)
Y40 M8                 (Position 2)
X40                    (Position 3)
Y-40                   (Position 4)
X-75                   (Position 5)
Y-75                   (Position 6)
G0 Z100
M30
