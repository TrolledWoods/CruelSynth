# Cruel Synth
A "modular" synthesizing "programming language", or 
a node based synthesizer whose nodes are defined by a
simple programming language.

Written entirely in the rust programming language.

This synthesizer is pretty quick it seems, as it is
defined on one place on the heap with very few references
to other locations in memory, which lends itself to 
cpu caching.

The name is inspired by the fact that this synthesizer
can be used to very easily create horrifying earrape.
When I get around to implementing low pass filters the
earrapiness might be reduced by a good amount(I hope).

## The programming language
The programming language is very, very simple. The error
messages are pretty bad at the moment, and there are a lot
more things I want to add to it, like sequencing from midi files,
maybe a window you can open that allows you to play the synth
with your keyboard, polysynthesis and more!

There is only one type of command so far, and that is assignment.
The syntax for assignment is: ``[variable name]: [expression];``

```
x: 1.0;
```

Most things in the language look like function calls. The syntax
is pretty simple. It looks like this:
(also, comments use `#`, they can be placed anywhere in a line and will
comment out the rest of the line, kindof like ``//`` in C, C#, C++, rust, e.t.c)

```
# All operators take two parameters. The parameters are inside parenthesees,
# and are separated by colons.
# (Also, number can be specified in a number of ways)
x: +(1, 5.0);
y: -(4.0, .1);

# Some more complex functions only need 1 parameter. In this case,
# the paranthesees are optional, and can be omitted.
# The "osc" function is pretty important, it produces a sin
# wave with the argument giving the frequency(in hertz)
x_1: osc(310);
y_1: osc 310;

# Since we have all these variables, why not use some we
# defined previously?
# Variables are accessed by this syntax: ``$ [variable name]``.
# They don't have to be joined together, there can be whitespace in between.
# I just think it looks nicer for them to be joined together.
#
x_2: +($x, $y);
y_2: +($x_1, $y_1);

# Some functions have properties. Properties can at the moment
# only be numbers, not expressions. Also, if you set a property
# that doesn't exist.... Nothing happens. It's a bit wishy washy.
# This examples takes the "osc" function, which usually oscillates
# from -1 to 1, and maps it to oscillate between 0.1 and 1 instead.
scale_lfo: clamp[min: 0.1, max: 1.0] osc 1.0;

# In order to actually have sound come out, we
# have to tell the program what to put out into the
# speaker. This can be done 2 ways:
# STEREO: set left and right values to whatever you
# want to be exported there
left: * ($scale_lfo, $x_2);
right: * ($scale_lfo, $y_2);

# MONO: Just set the "out" value to whatever you
# want both the left and right channel to be set to
out: * ($scale_lfo, +($x_2, $y_2));
```

## All functions
``osc( freq )``, a sin wave at a set frequency.
``square( freq)``, a square wave at a set frequency.
``clamp[min, max] (val)``, clamps a value(that should range from -1 to 1)
    to the min and max properties.

## Variable accessing disclaimer
You can only use variables if they are defined above you.
Later(i.e. not implemented yet), delays will be able
to use variables defined further down. This is because
variables are instant. Using a variable uses the variables
value calculated on that iteration. Therefore, not having
this restriction could cause circular referencing, which
wouldn't be possible. Delays however can get their value
this frame before their arguments are calculated, which
would allow them to use circular referencing. Wow, that
was a mouthful, sorry

## Extra info
Why did I have the ugly syntax of using a dollar sign to
access variables? Because I didn't want to have to deal with
the annoyance of not knowing if what you used was a function
or if you accessed a variable. Therefore, I did what any lazy
person would and just made a dollar sign syntax instead. ;)
