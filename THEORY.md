
This is a collection of the denotational semantics I'm working on.

We start with a category $T$ of the types used by SaberVM, such as `u8` and `i64`.

$T$ is bicartesian closed, so for any pair of types $A$ and $B$ there are product types $A\times B$, coproducts $A+B$, and exponentials $B^A$ that are all objects in $T$. 

$T$ also has infinitary (co)products:
$$\forall a:8.\texttt{u8}^a\quad\rightarrow_\texttt{i64}\quad\texttt{u8}^\texttt{i64}$$
$$\forall a:8.\texttt{u8}^a\quad\rightarrow_{(\texttt{i64}\times\texttt{u8})@r}\quad\texttt{u8}^{(\texttt{i64}\times\texttt{u8})@r}$$
etc.

That is to say, to denote a polymorphic type, we use an infinitary product where we index the projections by other objects of $T$ (in particular, objects with a size of $8$ in the above example, where "size" is calculated with a function from types to natural numbers). In SaberVM, the `app` opcode is this type-indexed projection family of functions (with the caveat that `app` is actually the result of transforming this family of morphisms in $T$ to a fairly different family of morphisms in $C$, discussed below).

Existential types are denoted with infinitary coproducts with the type-indexed `pack`-opcode family of functions (with the same caveat):
$$\texttt{u8}^\texttt{i64}\quad\rightarrow_\texttt{i64}\quad\exists a:8.\texttt{u8}^a$$
$$\texttt{u8}^{(\texttt{i64}\times\texttt{u8})@r}\quad\rightarrow_{(\texttt{i64}\times\texttt{u8})@r}\quad\exists a:8.\texttt{u8}^a$$

$T$ has an initial and terminal object, called `void` and `unit` respectively.

There's a special object $R$ in $T$ which holds the termination states of a SaberVM program. $R$ is basically `u8` intuitively but it has very different morphisms going in and out of it, so they aren't isomorphic. Namely, there is only one morphism going out of $R$! It's the one going to `unit` to make `unit` a terminal object.

$T$ is a pretty typical category for modelling a pure functional language like Haskell. We need I/O side effects, mutable memory side effects, and exception handling. For simplicity I'm just going to stick all that into an unspecified monad $M$ for now, assuming very conventional implementations of the aforementioned effects.

The final thing we need is to make the whole thing run on a stack. We take a final category $C$ where the morphisms from the Kleisli category of $M$ become families of morphisms in $C$ indexed by a stack type (a list of types). We write the list in square brackets like $[A, B, C]$, concatenate lists with the $\oplus$ operator, and extract elements from a list with subscript notation like $L_i$. Note that $(\mathtt{ob}(T),\oplus,[])$ is the free monoid over the set of types $\mathtt{ob}(T)$. Note that we're using the objects of $T$, not $C$, ensuring type-lists don't nest. As an example of the conversion from $T$ to $C$, $\texttt{add}: \texttt{i64}\times\texttt{i64}\rightarrow\texttt{i64}$ becomes $\texttt{add}_\sigma:\sigma\oplus[\texttt{i64},\texttt{i64}]\rightarrow M(\sigma\oplus[\texttt{i64}])$.

Note that $C$ has all the objects of $T$. $C$ only adds the aforementioned type-list objects. Also note that $C$ is still a Kleisli category; we extend the definition of $M$ to type-lists in the obvious way (a type-list is a lot like a product type). Type-lists can technically show up in (co)products but never will since we only use them to interpret the stack type. For this reason we often talk about the objects of $T$ as the set of types, not the objects of $C$. Type lists *do* appear in exponentials however, as the arguments list.

There's a functor $\neg: C^{op}\rightarrow\mathtt{Set}$ (a presheaf) that is defined as $C(-,R)$, a set of Kleisli arrows. The entry point of a SaberVM program is a (Kleisli) morphism in $\neg\texttt{unit}$. SaberVM functions are all morphisms in images of $\neg$. Obviously, if there's a morphism in some $\neg A$, that is a morphism $A\rightarrow MR$, then a morphism $B\rightarrow MA$ can be composed with it to get a morphism $B\rightarrow MR$, and thus the composition of $B\rightarrow MA$ and $\neg A$ is a morphism in $\neg B$.

`call` is indexed both by a stack type $\sigma$ and an argument type $a$, and has the type signature $\texttt{call}_{\sigma,a}:\sigma\oplus a\oplus[\neg a]\rightarrow MR$. This means that `call` can be used to create morphisms in $\neg A$ for some $A$. 

`call_nz` is very similar, with the type signature $\texttt{callnz}_{\sigma,a}:\sigma\oplus a\oplus[\neg a,\neg a,\texttt{u8}]\rightarrow MR$, which of course means `call_nz` can also be used to create morphisms in images of $\neg$.

The final opcode that can be used to create morphisms of $\neg A$ is `halt`, which is just parameterized by a stack type: $halt_\sigma: \sigma\oplus[\texttt{u8}]\rightarrow MR$ . `halt` simply maps `u8`s into exit codes and ignores the rest of the stack.

Hopefully it's clear that functions must always end with `call`, `call_nz`, or `halt`, when there are no other opcodes representing (families of) Kleisli morphisms into $R$. Putting these at the end of a composition pipeline is the only way to form a $C$-morphism into $R$. And there isn't any instruction that starts with $R$ to continue that composition pipeline, so functions must have exactly one occurrence of these opcodes.

There's an opcode $\texttt{globalfunc}_{\sigma,a,f:\neg a}:\sigma\rightarrow M(\sigma\oplus[\neg a])$ which is indexed by a stack type $\sigma$, a type $a$, and a morphism $f:\neg a$. `global_func` is the opcode for pushing its morphism-index onto the stack.

The `get` opcode copies something from within the stack to the top. This reifies the environment onto the stack, so SaberVM doesn't need a separate environment. Note here $a$ is a type list, and $a_0$ is its first component type. Thus, we're expressing a type in the middle of a big type list (the stack). $$\texttt{get}_{\sigma,a}:\sigma\oplus a\rightarrow M(\sigma\oplus a\oplus a_0)$$
In this formalization, `get` isn't indexed by some $i$ as you might expect; it's implicit in the number of components in the type list $a$. In the "syntax" of the bytecode, we do use a number instead, which is easily interpreted as partitioning the stack at that position (from the top) into two type lists to use as the denotational indices of `get`. The SaberVM type checker literally just gets the $i^{th}$ component of the stack type (from the top) and appends it to get the new stack type. 

The `app` and `pack` opcodes we mentioned above have denotations with the following type signatures:
$$\texttt{app}_{\sigma,s,a:s}:\sigma\oplus[\forall x:s.A]\rightarrow M(\sigma\oplus[A\{x\mapsto a\}])$$
$$\texttt{pack}_{\sigma,s,a:s}:\sigma\oplus [A\{x\mapsto a\}]\rightarrow M(\sigma\oplus[\exists x:s.A])$$

Which just comes from the transformation of $T$-morphisms into $C$-morphisms.

Note that morphisms a user might want in $T$ have to be CPS-converted to get the corresponding morphisms in $C$, if that morphism isn't accessible via the denotations of the opcodes. This CPS conversion is very different from the stackification transformation to produce the opcodes of $C$ from morphisms in $T$, and affects the type signature and way of calling the morphism a lot.

TODO: regions! Or more generally, a better formalization of the monad $M$, that can lead to a memory safety proof for SaberVM regions.