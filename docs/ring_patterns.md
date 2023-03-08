# Ring patterns

Agents can set a custom ring pattern that affects how the ringer sounds when the agent calls the host.
They do this using a "ring pattern expression." These are strings with the following format:

## Pattern symbols

| Symbol         | Meaning                                                                     |
|----------------|-----------------------------------------------------------------------------|
| `C<h>,<l>,<t>` | Cycle high for `h` ms and low for `l` ms, for total of `t` ms |
| `R<f>,<t>` | Ring at `f` Hz for `t` milliseconds |
| `Q<t>` | Ring at 20Hz for `t` milliseconds |
| `L<t>`| Set low for `t` milliseconds |
| `H<t>` | Set high for `t` milliseconds |

## Examples

### Ring at 20Hz for 2 seconds, then rest for 4 seconds

```
Q2000 L4000
```
*or*
```
R20,2 L4
```
*or*
```
C25,25,20 L4
```

### Sets of two

```
Q300 L300 Q300 L3150
```

### Creepy 5Hz slow-ring

```
R5,2000 L4000
```

### Sinister 0.5Hz super-slow-ring

```
R.5,1000
```
*or*
```
H1 L1
```