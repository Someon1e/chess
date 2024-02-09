## Generic chess engine written in rust

### Features:
- UCI
- Legal move generator
- Alpha-beta pruning
- Move ordering
- Late move reduction
- Transposition table
- Quiescence search
- Iterative deepening
- PeSTO evaluation
- Stalemate and checkmate detection
- Repetition detection

### TODO:
- Magic numbers for perfect hashing in sliding move lookup
- Killer move heuristic
- Null move heuristic
- Remove fnv dependency
- Multiple lines, so that it does not always play the same moves.
