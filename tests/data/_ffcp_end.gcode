; Source: https://github.com/DrLex0/FFCP-GCodeSnippets (GCode/End.gcode)
;- - - Custom finish printing G-code for FlashForge Creator Pro - - -
M73 P100; end build progress
M127; disable fan
; 500 is a nice Z speed for this, because it aurally signals the end of our build by resonating the enclosure.
G1 Z150 F500 ; send Z axis to bottom of machine
G162 X Y F2500; home X and Y axes
M109 S0 T0; set bed temperature to 0
M104 S0 T0; set 1st extruder temperature to 0
M104 S0 T1; set 2nd extruder temperature to 0
M18; disable all stepper motors
M70 P3; We <3 Making Things!
M72 P1; Play Ta-Da song
;- - - End finish printing G-code - - -
