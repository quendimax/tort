# TORT

_**T**est **ORT**hograpy_

A small markup language to facilitate learning of... something text-based.
Firstly it was created for testing orthography knowledge, and the first attempt
is [there](https://github.com/quendimax/orthotester).

## Tort script description

A tort script consists of lines. Every line can be a comment or a statement with
_orthograms_.

_Orthogram_ is part of a line where a mistake is expected to be able to happen.

So below you can read about every kind of lines.

### Translation line

```tort
hello -> salut
```

Translation line prints text in original "language", and propose you to
"translate" it.

```
  Translate:  hello
Your answer:  salut
       --->   Right
```

### Orthogram line

The orthogram line can contain two kinds of orthograms. Every orthogram line can
have many orthogram of different kinds.

#### Gap orthogram

```tort
a p[ie]ce of the cake
```

A gap orthogram allows you to learn which symbols should be written in difficult
for memorization words.

```
  Fill gaps:  a p_ce of the cake
Your answer:  a piece of the cake
        --->   Right
```

#### Choice orthogram

```tort
w[ee|e|ie]k
```

The choice orthogram is close to the gap one, but it shows possible variants, so
you can choose between them. In your script the first variant is meant to be
right.


```
  Fill gaps:  w[e/ie/ee]k
Your answer:  week
        --->   Right
```

### Comment

A comment starts with `#` symbol and continues to the end of the current line.
It is ignored by TORT interpreter.

### Public comment

A public comment starts with `#!` symbols and is printed during running your
script. It is used to send some help messages to user.

### Shebang

[Shebang] is a special kind of a public comment that is used by Unix systems.
You can add `#!/usr/bin/env tort` to the first line of your script, and add
executable permission to your script. This allows you to run your script just
with `./your-script.tort`, without specifying tort interpreter.

[shebang]: https://en.wikipedia.org/wiki/Shebang_(Unix)

## Bundle

To build a bundle for you OS, you can use [Cargo Packager] tool.

[Cargo Packager]: https://github.com/crabnebula-dev/cargo-packager
