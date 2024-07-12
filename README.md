# Generic chess engine written in rust

## Features:
- UCI
- Legal move generator
- Stalemate and checkmate detection
- Repetition detection
- Fail-soft alpha-beta pruning
- Iterative deepening
- Principal Variation Search
- Aspiration windows
- Quiescence search
- Transposition table
- Tuned piece-square-table-only evaluation
- Check extensions

### Search pruning and reductions
- Late move reduction
- Late move pruning
- Static null move pruning (also known as reverse futility pruning)
- Null move heuristic
- Internal iterative reduction

### Search move ordering
- History heuristic
- Killer move heuristic
- MVV-LVA

## TODO:
- Futility Pruning
- Opening book