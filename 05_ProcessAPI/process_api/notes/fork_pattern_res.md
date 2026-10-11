
### There is no priority. There is no guaranteed order. The pattern C2 → C3 → C1 → P1 is what your machine's scheduler happened to produce on most runs, not something the kernel promises. Run it long enough and you'll eventually see a different order (I can see one in your own paste — one run has [C3] before [C2]).

> Why it usually looks like C2 → C3 → C1 → P

It's a chain of "who is blocked waiting for whom." Follow the waits:
process	does it wait?	on whom?

|p | waiting| whom|
---
|C2|no|—|
|C3|no|—|
|C1|yes|C3|
|P1|yes|C1, then C2|

The kernel can only run a process that is runnable. Blocked processes are off the runqueue. So the first process to become runnable and get picked is one of the leaves — C2 or C3. Once C3 exits, C1 becomes runnable and finishes; once C1 exits, P's waitpid(x) returns and P finishes. That gives you the order of completion, roughly:



``` bash

[C2]:224188 | x:224187,y:0 | C2 did not created anychild so no reaping.
[C3]:224189 | x:0,y:0
[C1]:224187 | x:0,y:224189 | C3 dead with waitpid: 224189
[P1]:224101 | x:224187,y:224188 |
 Parent with PID:224101 Riped both c2:224188 and then c1:224187

[C2]:224465 | x:224464,y:0 | C2 did not created anychild so no reaping.
[C3]:224466 | x:0,y:0
[C1]:224464 | x:0,y:224466 | C3 dead with waitpid: 224466
[P1]:224461 | x:224464,y:224465 |
 Parent with PID:224461 Riped both c2:224465 and then c1:224464

[C2]:224548 | x:224547,y:0 | C2 did not created anychild so no reaping.
[C3]:224549 | x:0,y:0
[C1]:224547 | x:0,y:224549 | C3 dead with waitpid: 224549
[P1]:224544 | x:224547,y:224548 |
 Parent with PID:224544 Riped both c2:224548 and then c1:224547

[C2]:224588 | x:224587,y:0 | C2 did not created anychild so no reaping.
[C3]:224589 | x:0,y:0
[C1]:224587 | x:0,y:224589 | C3 dead with waitpid: 224589
[P1]:224584 | x:224587,y:224588 |
 Parent with PID:224584 Riped both c2:224588 and then c1:224587

[C2]:224628 | x:224627,y:0 | C2 did not created anychild so no reaping.
[C3]:224629 | x:0,y:0
[C1]:224627 | x:0,y:224629 | C3 dead with waitpid: 224629
[P1]:224624 | x:224627,y:224628 |
 Parent with PID:224624 Riped both c2:224628 and then c1:224627

[C2]:224754 | x:224753,y:0 | C2 did not created anychild so no reaping.
[C3]:224755 | x:0,y:0
[C1]:224753 | x:0,y:224755 | C3 dead with waitpid: 224755
[P1]:224750 | x:224753,y:224754 |
 Parent with PID:224750 Riped both c2:224754 and then c1:224753

[C2]:224794 | x:224793,y:0 | C2 did not created anychild so no reaping.
[C3]:224795 | x:0,y:0
[C1]:224793 | x:0,y:224795 | C3 dead with waitpid: 224795
[P1]:224790 | x:224793,y:224794 |
 Parent with PID:224790 Riped both c2:224794 and then c1:224793

[C2]:224834 | x:224833,y:0 | C2 did not created anychild so no reaping.
[C3]:224835 | x:0,y:0
[C1]:224833 | x:0,y:224835 | C3 dead with waitpid: 224835
[P1]:224830 | x:224833,y:224834 |
 Parent with PID:224830 Riped both c2:224834 and then c1:224833

[C2]:224874 | x:224873,y:0 | C2 did not created anychild so no reaping.
[C3]:224875 | x:0,y:0
[C1]:224873 | x:0,y:224875 | C3 dead with waitpid: 224875
[P1]:224870 | x:224873,y:224874 |
 Parent with PID:224870 Riped both c2:224874 and then c1:224873

[C2]:224914 | x:224913,y:0 | C2 did not created anychild so no reaping.
[C3]:224915 | x:0,y:0
[C1]:224913 | x:0,y:224915 | C3 dead with waitpid: 224915
[P1]:224910 | x:224913,y:224914 |
 Parent with PID:224910 Riped both c2:224914 and then c1:224913

==================================================
|                  Different case                |
--------------------------------------------------

[C3]:224994 | x:0,y:0
[C2]:224993 | x:224992,y:0 | C2 did not created anychild so no reaping.
[C1]:224992 | x:0,y:224994 | C3 dead with waitpid: 224994
[P1]:224989 | x:224992,y:224993 |
 Parent with PID:224989 Riped both c2:224993 and then c1:224992


here c1 run prirror.
--------------------------------------------------

[C2]:225070 | x:225069,y:0 | C2 did not created anychild so no reaping.
[C3]:225071 | x:0,y:0
[C1]:225069 | x:0,y:225071 | C3 dead with waitpid: 225071
[P1]:225066 | x:225069,y:225070 |
 Parent with PID:225066 Riped both c2:225070 and then c1:225069

[C2]:225110 | x:225109,y:0 | C2 did not created anychild so no reaping.
[C3]:225111 | x:0,y:0
[C1]:225109 | x:0,y:225111 | C3 dead with waitpid: 225111
[P1]:225106 | x:225109,y:225110 |
 Parent with PID:225106 Riped both c2:225110 and then c1:225109

[C2]:225150 | x:225149,y:0 | C2 did not created anychild so no reaping.
[C3]:225151 | x:0,y:0
[C1]:225149 | x:0,y:225151 | C3 dead with waitpid: 225151
[P1]:225146 | x:225149,y:225150 |
 Parent with PID:225146 Riped both c2:225150 and then c1:225149

[C2]:225190 | x:225189,y:0 | C2 did not created anychild so no reaping.
[C3]:225191 | x:0,y:0
[C1]:225189 | x:0,y:225191 | C3 dead with waitpid: 225191
[P1]:225186 | x:225189,y:225190 |
 Parent with PID:225186 Riped both c2:225190 and then c1:225189

[C2]:225230 | x:225229,y:0 | C2 did not created anychild so no reaping.
[C3]:225231 | x:0,y:0
[C1]:225229 | x:0,y:225231 | C3 dead with waitpid: 225231
[P1]:225226 | x:225229,y:225230 |
 Parent with PID:225226 Riped both c2:225230 and then c1:225229

[C2]:225345 | x:225344,y:0 | C2 did not created anychild so no reaping.
[C3]:225346 | x:0,y:0
[C1]:225344 | x:0,y:225346 | C3 dead with waitpid: 225346
[P1]:225316 | x:225344,y:225345 |
 Parent with PID:225316 Riped both c2:225345 and then c1:225344

[C2]:225391 | x:225390,y:0 | C2 did not created anychild so no reaping.
[C3]:225392 | x:0,y:0
[C1]:225390 | x:0,y:225392 | C3 dead with waitpid: 225392
[P1]:225387 | x:225390,y:225391 |
 Parent with PID:225387 Riped both c2:225391 and then c1:225390

[C2]:225431 | x:225430,y:0 | C2 did not created anychild so no reaping.
[C3]:225432 | x:0,y:0
[C1]:225430 | x:0,y:225432 | C3 dead with waitpid: 225432
[P1]:225427 | x:225430,y:225431 |
 Parent with PID:225427 Riped both c2:225431 and then c1:225430

[C2]:225471 | x:225470,y:0 | C2 did not created anychild so no reaping.
[C3]:225472 | x:0,y:0
[C1]:225470 | x:0,y:225472 | C3 dead with waitpid: 225472
[P1]:225467 | x:225470,y:225471 |
 Parent with PID:225467 Riped both c2:225471 and then c1:225470

[C2]:225511 | x:225510,y:0 | C2 did not created anychild so no reaping.
[C3]:225512 | x:0,y:0
[C1]:225510 | x:0,y:225512 | C3 dead with waitpid: 225512
[P1]:225507 | x:225510,y:225511 |
 Parent with PID:225507 Riped both c2:225511 and then c1:225510

[C2]:225590 | x:225589,y:0 | C2 did not created anychild so no reaping.
[C3]:225591 | x:0,y:0
[C1]:225589 | x:0,y:225591 | C3 dead with waitpid: 225591
[P1]:225586 | x:225589,y:225590 |
 Parent with PID:225586 Riped both c2:225590 and then c1:225589

[C2]:225672 | x:225671,y:0 | C2 did not created anychild so no reaping.
[C3]:225673 | x:0,y:0
[C1]:225671 | x:0,y:225673 | C3 dead with waitpid: 225673
[P1]:225668 | x:225671,y:225672 |
 Parent with PID:225668 Riped both c2:225672 and then c1:225671

[C2]:225712 | x:225711,y:0 | C2 did not created anychild so no reaping.
[C3]:225713 | x:0,y:0
[C1]:225711 | x:0,y:225713 | C3 dead with waitpid: 225713
[P1]:225708 | x:225711,y:225712 |
 Parent with PID:225708 Riped both c2:225712 and then c1:225711

[C2]:225752 | x:225751,y:0 | C2 did not created anychild so no reaping.
[C3]:225753 | x:0,y:0
[C1]:225751 | x:0,y:225753 | C3 dead with waitpid: 225753
[P1]:225748 | x:225751,y:225752 |
 Parent with PID:225748 Riped both c2:225752 and then c1:225751

[C2]:225792 | x:225791,y:0 | C2 did not created anychild so no reaping.
[C3]:225793 | x:0,y:0
[C1]:225791 | x:0,y:225793 | C3 dead with waitpid: 225793
[P1]:225788 | x:225791,y:225792 |
 Parent with PID:225788 Riped both c2:225792 and then c1:225791

[C2]:225832 | x:225831,y:0 | C2 did not created anychild so no reaping.
[C3]:225833 | x:0,y:0
[C1]:225831 | x:0,y:225833 | C3 dead with waitpid: 225833
[P1]:225828 | x:225831,y:225832 |
 Parent with PID:225828 Riped both c2:225832 and then c1:225831

[C2]:225911 | x:225910,y:0 | C2 did not created anychild so no reaping.
[C3]:225912 | x:0,y:0
[C1]:225910 | x:0,y:225912 | C3 dead with waitpid: 225912
[P1]:225907 | x:225910,y:225911 |
 Parent with PID:225907 Riped both c2:225911 and then c1:225910

[C3]:225992 | x:0,y:0
[C2]:225993 | x:225991,y:0 | C2 did not created anychild so no reaping.
[C1]:225991 | x:0,y:225992 | C3 dead with waitpid: 225992
[P1]:225988 | x:225991,y:225993 |
 Parent with PID:225988 Riped both c2:225993 and then c1:225991

[C2]:226032 | x:226031,y:0 | C2 did not created anychild so no reaping.
[C3]:226033 | x:0,y:0
[C1]:226031 | x:0,y:226033 | C3 dead with waitpid: 226033
[P1]:226028 | x:226031,y:226032 |
 Parent with PID:226028 Riped both c2:226032 and then c1:226031

[C2]:226072 | x:226071,y:0 | C2 did not created anychild so no reaping.
[C3]:226073 | x:0,y:0
[C1]:226071 | x:0,y:226073 | C3 dead with waitpid: 226073
[P1]:226068 | x:226071,y:226072 |
 Parent with PID:226068 Riped both c2:226072 and then c1:226071

[C2]:226114 | x:226113,y:0 | C2 did not created anychild so no reaping.
[C3]:226115 | x:0,y:0
[C1]:226113 | x:0,y:226115 | C3 dead with waitpid: 226115
[P1]:226108 | x:226113,y:226114 |
 Parent with PID:226108 Riped both c2:226114 and then c1:226113

[C2]:226198 | x:226197,y:0 | C2 did not created anychild so no reaping.
[C3]:226199 | x:0,y:0
[C1]:226197 | x:0,y:226199 | C3 dead with waitpid: 226199
[P1]:226194 | x:226197,y:226198 |
 Parent with PID:226194 Riped both c2:226198 and then c1:226197

[C2]:226279 | x:226278,y:0 | C2 did not created anychild so no reaping.
[C3]:226280 | x:0,y:0
[C1]:226278 | x:0,y:226280 | C3 dead with waitpid: 226280
[P1]:226275 | x:226278,y:226279 |
 Parent with PID:226275 Riped both c2:226279 and then c1:226278


```
