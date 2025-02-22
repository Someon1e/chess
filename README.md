# Generic chess engine written in Rust

## Features:
- UCI
- Bit boards
- Legal move generator
- Stalemate and checkmate detection
- Repetition detection
- Profile-guided optimisation
- Pondering

### Search
- SPSA-tuned search parameters
- Fail-soft alpha-beta pruning
- Iterative deepening
- Principal variation search
- Aspiration windows
- Quiescence search
- Transposition table
- Check extensions
- `improving` heuristic

### Search pruning and reductions
- Late move reduction
- Late move pruning
- Futility pruning
- Static null move pruning (also known as reverse futility pruning)
- Null move heuristic
- Internal iterative reduction

### Evaluation
- Tuned piece-square-table-only evaluation
- Pawn correction history

### Search move ordering
- Butterfly history heuristic
- Killer move heuristic
- MVV-LVA

## TODO:
- Static exchange evaluation
- Tablebases
- Opening book
