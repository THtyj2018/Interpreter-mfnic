# Mathematical Function Interpreter

## Build and run
```shell
cargo run --example mfnic --release
```

## Code examples
+ basic usage
```
>>> a = 4
>>> b = 1.0
>>> f : x, y = ...
... x * x / a + y * y / b
>>> b = (-4.2e-2 + a / 10.4) ^ 2
>>> f(1, 1)
1.25
>>> a > b
1.0
>>> a <= b && a != 2.0
0.0
>>> my_abs : x = x >= 0 ? x : -x
>>> my_abs(-1.1)
1.1
>>> atan2(_)
!Error: Inconsistent Variables Count: atan2
>>> atan(_)
0.8329812666744317
>>> quit
```
+ capture and recursive
```
>>> a = 1
>>> a = a + a
>>> a
2
>>> fib: x = ((1+sqrt(5)) / 2) ^ x
>>> fib(1)
1.618033988749895
>>> fib: n = n > 2 ? fib(n-1) + fib(n-2) : 1
>>> fib(1)
1
>>> fib(10)
55
>>> quit
```

## Built-in definitions
Built-in variables and functions are not allowed to be rewrite.
+ `pi`, `e`
+ `abs`, `floor`, `ceil`, `round`, `sgn`
+ `sqrt`, `cbrt`
+ `sin`, `cos`, `tan`
+ `asin`, `acos`, `atan`, `atan2`
+ `ln`, `log`

## Data Type
Only has 64-bit floating point value.

## Tokens
| Token |         Expression          |
| :---: | :-------------------------: |
| IDENT |        `[\w^\d]\w*`         |
|  NUM  | `\d+(.\d+)?([eE][+-]?\d+)?` |
|  MD   |            `*|/`            |
|  PN   |            `+|-`            |
|  CMP  |    `<=|>=|<|>|==|!=|<=>`    |
|  OR   |           `\|\|`            |
|  AND  |            `&&`             |
| WRAP  |            `...`            |
+ literals: `=()!^?:,`

## Grammer
### YACC and G(S)
```
 1) statement: assignment                                // S -> A
 2)          | expression                                // S -> E
 3) assignment: IDENT '=' expression                     // A -> i=E
 4)           | IDENT ':' variable_list '=' expression   // A -> i:V=E
 5) variable_list: variable_list ',' IDENT               // V -> V,i
 6)              | IDENT                                 // V -> i
 7) expression: '(' expression ')'                       // E -> (E)
 8)           | '!' expression                           // E -> !E
 9)           | PN expression                            // E -> pE
10)           | expression '^' expression                // E -> E^E
11)           | expression MD expression                 // E -> EmE
12)           | expression PN expression                 // E -> EpE
13)           | expression CMP expression                // E -> EcE
14)           | expression OR expression                 // E -> EoE
15)           | expression AND expression                // E -> EaE
16)           | expression '?' expression ':' expression // E -> E?E:E
17)           | IDENT '(' parameter_list ')'             // E -> i(P)
18)           | IDENT                                    // E -> i
19)           | NUM                                      // E -> n
20) parameter_list: parameter_list ',' expression        // P -> P,E
21)               | expression                           // P -> E
```

### Operator Priority
The following operators are sorted in descending order of priority:

| binary operator |                                                description                                                 |
| :-------------: | :--------------------------------------------------------------------------------------------------------: |
|       `^`       |                                               exponentiation                                               |
|      `MD`       |                                        multiplication and division                                         |
|      `PN`       |                                             sum and difference                                             |
|      `CMP`      | comparision, result is `1.0` for `true` or `0.0` for `false`<br>(`1.0`, `0.0`, `-1.0` for case of `'<=>'`) |
|      `OR`       |                                           logical 'or' operation                                           |
|      `AND`      |                                          logical 'and' operation                                           |
The ternary operator `?:` has the lowest priority.

## SLR(1)
Initial state is 0.

`{.E}` is `.(E) | .!E | .pE | .E^E | .EmE | .EpE | .EcE | .EoE | .EaE | .E?E:E | .i(P) | .i | .n`
`{E.xE}` is `E.^E | E.mE | E.pE | E.cE | E.oE | E.aE`

| State |      S       |         A          |          V           |                E                 |       P        |
| :---: | :----------: | :----------------: | :------------------: | :------------------------------: | :------------: |
|   0   | `.A`<br>`.E` | `.i=E`<br>`.i:V=E` |                      |              `{.E}`              |
|   1   |     `A.`     |
|   2   |     `E.`     |                    |                      |       `{E.xE}`<br>`E.?E:E`       |
|   3   |              | `i.=E`<br>`i.:V=E` |                      |         `i.(P)`<br>`i.`          |
|   4   |              |                    |                      |               `n.`               |
|   5   |              |                    |                      |         `(.E)`<br>`{.E}`         |
|   6   |              |                    |                      |         `!.E`<br>`{.E}`          |
|   7   |              |                    |                      |         `p.E`<br>`{.E}`          |
|   8   |              |                    |                      |         `E^.E`<br>`{.E}`         |
|   9   |              |                    |                      |         `Em.E`<br>`{.E}`         |
|  10   |              |                    |                      |         `Ep.E`<br>`{.E}`         |
|  11   |              |                    |                      |         `Ec.E`<br>`{.E}`         |
|  12   |              |                    |                      |         `Eo.E`<br>`{.E}`         |
|  13   |              |                    |                      |         `Ea.E`<br>`{.E}`         |
|  14   |              |                    |                      |        `E?.E:E`<br>`{.E}`        |
|  15   |              |       `i=.E`       |                      |              `{.E}`              |
|  16   |              |                    |                      |        `i(.P)`<br>`{.E}`         | `.P,E`<br>`.E` |
|  17   |              |      `i:.V=E`      |    `.V,i`<br>`.i`    |
|  18   |              |                    |                      |  `(E.)`<br>`{E.xE}`<br>`E.?E:E`  |
|  19   |              |                    |                      |         `i.(P)`<br>`i.`          |
|  20   |              |                    |                      |  `!E.`<br>`{E.xE}`<br>`E.?E:E`   |
|  21   |              |                    |                      |  `pE.`<br>`{E.xE}`<br>`E.?E:E`   |
|  22   |              |                    |                      |  `E^E.`<br>`{E.xE}`<br>`E.?E:E`  |
|  23   |              |                    |                      |  `EmE.`<br>`{E.xE}`<br>`E.?E:E`  |
|  24   |              |                    |                      |  `EpE.`<br>`{E.xE}`<br>`E.?E:E`  |
|  25   |              |                    |                      |  `EcE.`<br>`{E.xE}`<br>`E.?E:E`  |
|  26   |              |                    |                      |  `EoE.`<br>`{E.xE}`<br>`E.?E:E`  |
|  27   |              |                    |                      |  `EaE.`<br>`{E.xE}`<br>`E.?E:E`  |
|  28   |              |                    |                      | `E?E.:E`<br>`{E.xE}`<br>`E.?E:E` |
|  29   |              |                    |                      |  `i=E.`<br>`{E.xE}`<br>`E.?E:E`  |
|  30   |              |                    |                      |       `{E.xE}`<br>`E.?E:E`       |      `E.`      |
|  31   |              |                    |                      |             `i(P.)`              |     `P.,E`     |
|  32   |              |      `i:V.=E`      |        `V.,i`        |
|  33   |              |                    |         `i.`         |
|  34   |              |                    |                      |              `(E).`              |
|  35   |              |                    |                      |        `E?E:.E`<br>`{.E}`        |
|  36   |              |                    |                      |             `i(P).`              |
|  37   |              |                    |                      |              `{.E}`              |     `P,.E`     |
|  38   |              |      `i:V=.E`      |                      |              `{.E}`              |
|  39   |              |                    |        `V,.i`        |
|  40   |              |                    |                      | `E?E:E.`<br>`{E.xE}`<br>`E.?E:E` |
|  41   |              |                    |                      |       `{E.xE}`<br>`E.?E:E`       |     `P,E.`     |
|  42   |              |      `i:V=E.`      | `{E.xE}`<br>`E.?E:E` |
|  43   |              |                    |        `V,i.`        |

| State |   A   |   V   |   E   |   P   |
| :---: | :---: | :---: | :---: | :---: |
|   0   |   1   |       |   2   |
|   5   |       |       |  18   |
|   6   |       |       |  20   |
|   7   |       |       |  21   |
|   8   |       |       |  22   |
|   9   |       |       |  23   |
|  10   |       |       |  24   |
|  11   |       |       |  25   |
|  12   |       |       |  26   |
|  13   |       |       |  27   |
|  14   |       |       |  28   |
|  15   |       |       |  29   |
|  16   |       |       |  30   |  31   |
|  17   |       |  32   |
|  35   |       |       |  40   |
|  37   |       |       |  41   |
|  38   |       |       |  42   |

| State |   i   |   n   |   =   |   (   |   )   |   !   |   ^   |   m   |   p   |   c   |   o   |   a   |   ?   |   :   |   ,   |   #   |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
|   0   |  s3   |  s4   |       |  s5   |       |  s6   |       |       |  s7   |
|   1   |       |       |       |       |       |       |       |       |       |       |       |       |       |       |       |  acc  |
|   2   |       |       |       |       |       |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |       |       |  acc  |
|   3   |       |       |  s15  |  s16  |       |       |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |  s17  |       |  r18  |
|   4   |       |       |       |       |  r19  |       |  r19  |  r19  |  r19  |  r19  |  r19  |  r19  |  r19  |  r19  |  r19  |  r19  |
| 5~16  |  s19  |  s4   |       |  s5   |       |  s6   |       |       |  s7   |
|  17   |  s33  |
|  18   |       |       |       |       |  s34  |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |
|  19   |       |       |       |  s16  |  r18  |       |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |  r18  |
|  20   |       |       |       |       |  r8   |       |  r8   |  r8   |  r8   |  r8   |  r8   |  r8   |  r8   |  r8   |  r8   |  r8   |
|  21   |       |       |       |       |  r9   |       |  r9   |  r9   |  r9   |  r9   |  r9   |  r9   |  r9   |  r9   |  r9   |  r9   |
|  22   |       |       |       |       |  r10  |       |  r10  |  r10  |  r10  |  r10  |  r10  |  r10  |  r10  |  r10  |  r10  |  r10  |
|  23   |       |       |       |       |  r11  |       |  s8   |  r11  |  r11  |  r11  |  r11  |  r11  |  r11  |  r11  |  r11  |  r11  |
|  24   |       |       |       |       |  r12  |       |  s8   |  s9   |  r12  |  r12  |  r12  |  r12  |  r12  |  r12  |  r12  |  r12  |
|  25   |       |       |       |       |  r13  |       |  s8   |  s9   |  s10  |  r13  |  r13  |  r13  |  r13  |  r13  |  r13  |  r13  |
|  26   |       |       |       |       |  r14  |       |  s8   |  s9   |  s10  |  s11  |  r14  |  r14  |  r14  |  r14  |  r14  |  r14  |
|  27   |       |       |       |       |  r15  |       |  s8   |  s9   |  s10  |  s11  |  s12  |  r15  |  r15  |  r15  |  r15  |  r15  |
|  28   |       |       |       |       |       |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |  s35  |
|  29   |       |       |       |       |       |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |       |       |  r3   |
|  30   |       |       |       |       |  r21  |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |       |  r21  |  r21  |
|  31   |       |       |       |       |  s36  |       |       |       |       |       |       |       |       |       |  s37  |
|  32   |       |       |  s38  |       |       |       |       |       |       |       |       |       |       |       |  s39  |
|  33   |       |       |  r6   |       |       |       |       |       |       |       |       |       |       |       |  r6   |
|  34   |       |       |       |       |  r7   |       |  r7   |  r7   |  r7   |  r7   |  r7   |  r7   |  r7   |  r7   |  r7   |  r7   |
|  35   |  s19  |  s4   |       |  s5   |       |  s6   |       |       |  s7   |
|  36   |       |       |       |       |  r17  |       |  r17  |  r17  |  r17  |  r17  |  r17  |  r17  |  r17  |  r17  |  r17  |  r17  |
|  37   |  s19  |  s4   |       |  s5   |       |  s6   |       |       |  s7   |
|  38   |  s19  |  s4   |       |  s5   |       |  s6   |       |       |  s7   |
|  39   |  s43  |
|  40   |       |       |       |       |  r16  |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |  r16  |  r16  |  r16  |
|  41   |       |       |       |       |  r20  |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |       |  r20  |
|  42   |       |       |       |       |       |       |  s8   |  s9   |  s10  |  s11  |  s12  |  s13  |  s14  |       |       |  r4   |
|  43   |       |       |  r5   |       |       |       |       |       |       |       |       |       |       |       |  r5   |
