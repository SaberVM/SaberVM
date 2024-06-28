
This is a collection of the denotational semantics I'm working on.

We start with a category $T$ of the types used by SaberVM, such as `u8` and `i64`.

$T$ is bicartesian closed, so for any pair of types $A$ and $B$ there are product types $A\times B$, coproducts $A+B$, and exponentials $B^A$ that are all objects in $T$. However, the formalization is dramatically improved by using limits and colimits from arbitrary finite discrete categories, generalizing the typical product and coproduct constructions. In simpler terms, we're generalizing products and coproducts to use zero or more component types, instead of necessarily exactly two. We call these "n-products" and "n-coproducts" to communicate this change clearly throughout, inspired by the term "n-tuple" for generalizing pairs of values. This formalization is useful because we can more easily count the number of component types of a product or coproduct, without having to consider whether or not the component types are themselves products or coproducts respectively. If I have a regular product type $(A\times B)\times C$ it's a bit messy to say that this has three component types, even if that's the statement I want to make, because any of $A$, $B$, or $C$ could be a product type itself. Later we will formalize the stack as a product type, and want to talk about the $i^{th}$ component type in the stack, but with a tuple only increasing the number of types on the stack by one, instead of incrementing once for every component in the tuple. With the n-product formalization, the number of component types is a simple finite number: a 3-product has 3 component types, even if some of these are themselves n-products. The case for n-coproducts is essentially the same. As notation, we write $A_0\land A_1\land...\land A_n$ for an n-product and $A_0\lor A_1\lor...\lor A_n$. Note that these are not associative binary operators, but non-associative operators of an arbitrary number of arguments! We write $A_i$ for mentioning the $i^{th}$ component type of an n-(co)product. We use a "spread" notation for variables (representing n-(co)products) whose component types should be counted individually in a larger n-(co)product: $A\land x...\land B$.

$T$ also has limits with (countable) infinite discrete diagram categories, which just means a countable infinity of component types. We call these $\omega$-products. The stack type is always finite, so these infinite types won't have their component types counted. An example:
$$\forall a:8.\;\texttt{u8}^a\quad\rightarrow_\texttt{u8}\quad\texttt{u8}^\texttt{u8}$$
$$\forall a:8.\;\texttt{u8}^a\quad\rightarrow_{(\texttt{i64}\times\texttt{u8})}\quad\texttt{u8}^{(\texttt{i64}\times\texttt{u8})}$$
etc.

That is to say, to denote a polymorphic type, we use an $\omega$-product where we index the projections by other objects of $T$ (in particular, objects with a size of $8$ in the above example, where "size" is calculated with a function from types to natural numbers). In SaberVM, the `app` opcode is this type-indexed projection family of functions (with the caveat that `app` is actually the result of transforming this family of morphisms in $T$ to a fairly different family of morphisms in $C$, discussed below, even gaining an additional index).

Existential types are denoted with $\omega$-coproducts with the type-indexed `pack`-opcode family of functions (with the same caveat):
$$\texttt{u8}^\texttt{u8}\quad\rightarrow_\texttt{u8}\quad\exists a:8.\;\texttt{u8}^a$$$$\texttt{u8}^{(\texttt{i64}\times\texttt{u8})}\quad\rightarrow_{(\texttt{i64}\times\texttt{u8})}\quad\exists a:8.\;\texttt{u8}^a$$
$T$ has an initial and terminal object, called `void` and `unit` respectively.

There's a special object $R$ in $T$ which holds the termination states of a SaberVM program. $R$ is basically `u8` intuitively but it has very different morphisms going in and out of it, so they aren't isomorphic. Namely, there is only one morphism going out of $R$! It's the one going to `unit` to make `unit` a terminal object.

$T$ is a pretty typical category for modelling a pure functional language like Haskell. We need I/O side effects, mutable memory side effects, and exception handling. For simplicity I'm just going to stick all that into an unspecified monad $U$ for now, assuming very conventional implementations of the aforementioned effects.

So then we get $U$, the Kleisli category of monad $U$ over $T$.

The final thing we need is to make the whole thing run on a stack. We take a final category $C$ where the morphisms from $U$ become families of morphisms in $C$ indexed by a stack type (some n-product object in $T$). For example, $add: \texttt{i64}\land\texttt{i64}\rightarrow U\texttt{i64}$ becomes $add_\sigma:\sigma...\land\texttt{i64}\land\texttt{i64}\rightarrow U(\sigma...\land\texttt{i64})$.

Note that $C$ and $U$ have the same set of objects as $T$.

There's a functor $\neg: C^{op}\rightarrow\mathtt{Set}$ (a presheaf) that is defined as $C(-,R)$. The entry point of a SaberVM program is a (Kleisli) morphism in $\neg\texttt{unit}$. SaberVM functions are all morphisms in images of $\neg$. Obviously, if there's a morphism in some $\neg A$, that is a morphism $A\rightarrow UR$, then a morphism $B\rightarrow UA$ can be composed with it to get a morphism $B\rightarrow UR$, and thus the composition of $B\rightarrow UA$ and $\neg A$ is a morphism in $\neg B$.

`call` is indexed both by a stack type $\sigma$ and an argument type $a$, and has the type signature $\texttt{call}_{\sigma,a}:\sigma...\land a\land\neg a\rightarrow UR$. This means that `call` can be used to create morphisms in $\neg A$ for some $A$. 

Hopefully `call` shows how `call_nz` would be denoted and how it could also be used to create morphisms of $\neg A$ for some $A$.

The final opcode that can be used to create morphisms of $\neg A$ is `halt`, which is just parameterized by a stack type: $halt_\sigma: \sigma...\land\texttt{u8}\rightarrow UR$ . As you can imagine, `halt` simply maps `u8`s into exit codes and ignores the rest of the stack.

Hopefully it's clear that functions must always end with `call`, `call_nz`, or `halt`. Putting these at the end of a composition pipeline is the only way to form a $C$-morphism into $R$. And there isn't any instruction that starts with $R$ to continue that composition pipeline, so functions must have exactly one occurrence of these opcodes.

There's a unary operator $\$_{\sigma,a,f}:\sigma\rightarrow U(\sigma...\land\neg a)$ which is indexed by a stack type $\sigma$, a type $a$, and a morphism $f:\neg a$. $\$$ is the opcode for pushing its morphism-index onto the stack. The generated opcode just pushes that morphism onto the stack. In SaberVM this is the `global_func` opcode.

The `get` opcode copies something from within the stack to the top. This reifies the environment onto the stack, so SaberVM doesn't need a separate environment. Note here $a$ is an n-product type, and $a_0$ is its first component type. Thus, we're expressing a type in the middle of a big product type (the stack). $$\texttt{get}_{\sigma,a}:\sigma...\land a...\rightarrow U(\sigma...\land a...\land a_0)$$
Note that this complexity just comes from the indexing, which is more or less just syntax. The type of each morphism in the family is trivial to write. In this formalization, `get` isn't indexed by some $i$ as you might expect; it's implicit in the number of components in the n-product $a$. In the "syntax" of the bytecode, we do use a number instead, which is easily interpreted as partitioning the stack at that position (from the top) into two n-product types to use as the denotational indices of `get`. The SaberVM type checker literally just gets the $i^{th}$ component of the stack type (from the back) and appends it to get the new stack type. 

The `app` and `pack` opcodes we mentioned above have the following formalizations:
$$\texttt{app}_{\sigma,s,a:s}:\sigma...\land\forall x:s.\;A\rightarrow\sigma...\land A\{x\mapsto a\}$$
$$\texttt{pack}_{\sigma,s,a:s}:\sigma...\land A\{x\mapsto a\}\rightarrow\sigma...\land\exists x:s.A$$
Which just comes from the transformation of $U$-morphisms into $C$-morphisms.