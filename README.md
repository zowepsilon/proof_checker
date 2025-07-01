## Proof checker

A natural deduction proof checker.

This proof checker features hypothesis inference using a little bit of unification.

For instance in the rule: 
```math
\frac{Gamma ⊦ F}{Gamma ⊦ F \lor G} \text{(or left introduction)}
```

```
    Γ ⊦ F
------------ (or left introduction)
 Γ ⊦ F ∨ G
```
the program can infer that the hypothesis must be `F` from the conclusion and the shape of the rule only.

### Examples

```rust

// proof of the commutativity of the disjunction

Prop AndComm(infer p, infer q):
  q.p => p.q
Proof:
  // we start the proof by using the and introduction rule
  // every hypothesis (p, q) can be infered here
  AndIntro:
    // we can infer the right side of the hypothesis q ∧ p 
    // as the conclusion is p
    // but not the left side (q) so it is passed as an argument
    AndElimRight(q): 
      // the axiom rule does not have any hypothesis
      Ax
    AndElimLeft(p):
      Ax

```
