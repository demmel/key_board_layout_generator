# Keyboard Layout Generator

Have you ever wanted a keyboard layout optimized for your personal typing patterns? This suite of tools may be for you.

# How it works

First, you'll need to collect a log of youring typing to give the layout generator something to work with.

```
cargo run --release --bin keylogger -- --help
A simple keylogger that logs key presses and releases to a file

Usage: keylogger.exe <LOG_FILE>

Arguments:
  <LOG_FILE>  Path to the log file to store key presses and releases. If the file does not exist, it will be created, otherwise it will be appended to

Options:
  -h, --help  Print help
```

This will get you a simple log of key presses and releases that you can feed to the `explore_layouts` progran.

```
cargo run --release --bin explore_layouts -- --help
A tool to explore different keyboard layouts using a genetic algorithm and simulated annealing.

Every iteration, the program will output the max, mean, and min scores of the current population, as well as the diversity of the population.  The program will also save the best layout to a file called `best.txt`.

The program will run indefinitely, so you will need to manually stop it when you are satisfied with the results.

Usage: explore_layouts.exe <LOG_FILE> <KEYMAP_CONFIG>

Arguments:
  <LOG_FILE>
          Path to the log file created by the keylogger

  <KEYMAP_CONFIG>
          Path to a keymap configuration file describing the layout of the physical keyboard.  See the README for more information

Options:
  -h, --help
          Print help (see a summary with '-h')
```

# The Layout Format

This is special layout format file that simiplifies the definition of a keyboard layout.

They look like this:

```
Fingers
LP: 70
LR: 50
LM: 80
LI: 100
LT: 100
RT: 100
RI: 100
RM: 80
RR: 50
RP: 70

Keys
-----------------------------------------------------------------
| = | 1 | 2 | 3 | 4 | 5 |   |   |   |   | 6 | 7 | 8 | 9 | 0 | - |
|LP |LP |LR |LM |LI |LI |   |   |   |   |RI |RI |RM |RR |RP |RP |
|25 |35 |45 |50 |50 |50 |   |   |   |   |50 |50 |50 |45 |35 |25 |
-----------------------------------------------------------------
|Tab| Q | W | E | R | T |   |   |   |   | Y | U | I | O | P | \ |
|LP |LP |LR |LM |LI |LI |   |   |   |   |RI |RI |RM |RR |RP |RP |
|35 |75 |75 |75 |75 |75 |   |   |   |   |75 |75 |75 |75 |75 |35 |
-----------------------------------------------------------------
|Esc| A | S | D | F | G |   |   |   |   | H | J | K | L | ; | ' |
|LP |LP |LR |LM |LI |LI |   |   |   |   |RI |RI |RM |RR |RP |RP |
|75 |100|100|100|100|100|   |   |   |   |100|100|100|100|100|75 |
-----------------------------------------------------------------
|LSh| Z | X | C | V | B |LCt|LAt|LMt|RCt| N | M | , | . | / |RSh|
|LP |LP |LR |LM |LI |LI |LT |LT |RT |RT |RI |RI |RM |RR |RP |RP |
|65 |85 |85 |85 |85 |85 |70 |50 |50 |70 |85 |85 |85 |85 |85 |65 |
-----------------------------------------------------------------
|   | ~ |Cap|<--|-->|Bks|Del|Hom|PUp|Etr|Spc| Up| Dn| [ | ] |   |
|   |LP |LR |LM |LI |LT |LT |LT |RT |RT |RT |RI |RI |RP |RP |   |
|   |50 |50 |50 |50 |100|100|70 |70 |70 |100|100|50 |50 |50 |   |
-----------------------------------------------------------------
|   |   |   |   |   |   |   |End|PDn|   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |LT |RT |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |80 |80 |   |   |   |   |   |   |   |
-----------------------------------------------------------------
```

The first section is the finger section. It defines the score for each finger.  
The second section is the key section. It defines the layout of the keyboard.
Each key is defined by a character, a finger, and a score.
The layout is defined by a grid of keys.

This layout makes it easy to define a layout for a keyboard without having to consider the position of each key while writing something like JSON.

The project provides a layout file for the Kinesis Advantage 360 in `kinesis.layout`. Feel free to make your own for your favorite keyboard.
