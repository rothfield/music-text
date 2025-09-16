# Technical Note: Using 2D Array Grammars for Annotation Alignment

## Introduction

In many domains (linguistics, music, translation, notation), information is arranged in **parallel lines** that must be aligned column-wise.  
Examples include:

- Interlinear glossed text in linguistics  
- Lyrics beneath notes in music notation  
- Word-by-word translation glosses  
- Parallel corpora in NLP  

Our simple example illustrates this:

```
these   are   annotations
 hello   i          eat
```

Intuitively, we want to **pair** the tokens from the top and bottom rows column by column:

```
{ (hello, these), (i, are), (eat, annotations) }
```

---

## Interpreting as a 2D Array

We can represent the two rows as a **2 × 3 array**:

```
[ these     are     annotations ]
[ hello      i            eat   ]
```

This is the natural input structure for a **2D array grammar**.

---

## 2D Array Grammar Basics

A **2D array grammar** generalizes context-free string grammars to two dimensions:

- Instead of rewriting substrings, we rewrite **sub-arrays**.
- Rules can apply to rectangular blocks (e.g. 2×1, 2×2).
- Useful for describing **grid-structured layouts**.

Formally, a 2D array grammar is a tuple:

```
G = (N, T, S, P)
```

Where:
- `N` = nonterminals
- `T` = terminals
- `S` = start symbol
- `P` = set of productions

---

## Grammar for the Example

**Nonterminals:**
- `S` → start
- `A` → annotation token
- `B` → base token
- `P` → paired token

**Terminals:**
- `"these"`, `"are"`, `"annotations"`
- `"hello"`, `"i"`, `"eat"`

**Start Array:**

```
[ A   A   A ]
[ B   B   B ]
```

**Productions:**

1. Annotation substitution:
   ```
   A → these | are | annotations
   ```

2. Base substitution:
   ```
   B → hello | i | eat
   ```

3. Pairing rule (vertical alignment):
   ```
   [ A ]
   [ B ]   →   [ P ]
   ```

4. Interpretation of P:
   ```
   P → (B, A)
   ```

---

## Derivation Walkthrough

1. Start:

   ```
   [ A   A   A ]
   [ B   B   B ]
   ```

2. Substitute terminals:

   ```
   [ these     are     annotations ]
   [ hello      i            eat   ]
   ```

3. Apply pairing rule:

   ```
   [ P   P   P ]
   ```

4. Interpret pairs:

   ```
   [ (hello, these)   (i, are)   (eat, annotations) ]
   ```

---

## Connections to Other Formalisms

- **Interlinear Glossed Text (IGT):** Linguists align morphemes and translations in multiple lines.  
- **Autosegmental Phonology:** Multiple tiers (tone, stress, segment) aligned by association rules.  
- **Tile Grammars / Picture Grammars:** Used for image and level generation in games.  

All share the principle of **parallel lines + alignment + rules of association**.

---

## Conclusion

This toy example shows how **2D array grammars** provide a natural formalism for column-wise annotation alignment:

- Treat stacked tokens as 2D sub-arrays.  
- Apply production rules to collapse them into paired units.  
- Generalize to multi-line annotation systems.  

This bridges domains such as linguistics, music, and procedural content generation, where multi-tier alignment is a central problem.
